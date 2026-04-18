use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::sync::mpsc;

use crate::config::AppConfig;
use crate::meteorological::{cloudbase, dew_point, frost_point, heat_index, wind_chill};
use crate::sanity::sanity_check;
use crate::types::*;
use crate::units::{c_to_k, f_to_c, mps_to_mph};

const TOPIC_PREFIX: &str = "mqtt-wx";
const RAIN_WINDOW_SECONDS: f64 = 24.0 * 60.0 * 60.0;

fn topic_lightning_count() -> String {
    format!("{TOPIC_PREFIX}/lightning_count")
}

fn topic_rain_24h() -> String {
    format!("{TOPIC_PREFIX}/rain_24h")
}

/// Internal state for weather data processing, decoupled from MQTT transport.
pub struct Processor {
    pub output_data: OutputData,
    pub previous_output_data: OutputData,
    pub total_lightning_strikes: i64,
    pub rain: f64,
    pub rain_events: VecDeque<(f64, f64)>,
    pub sensor_height_m: f64,
    pub elevation_m: f64,
    // Tracks whether retained topics have been consumed
    lightning_count_loaded: bool,
    rain_24h_loaded: bool,
}

/// Actions the processor wants the MQTT layer to perform after processing a message.
pub enum ProcessorAction {
    /// Publish data to a topic (topic, payload, retain)
    Publish(String, String, bool),
    /// Unsubscribe from a topic
    Unsubscribe(String),
}

impl Processor {
    pub fn new(sensor_height_m: f64, elevation_m: f64) -> Self {
        Self {
            output_data: OutputData::default(),
            previous_output_data: OutputData::default(),
            total_lightning_strikes: -1,
            rain: -1.0,
            rain_events: VecDeque::new(),
            sensor_height_m,
            elevation_m,
            lightning_count_loaded: false,
            rain_24h_loaded: false,
        }
    }

    /// Process an incoming MQTT message and return actions for the MQTT layer.
    /// Returns None if the message should not trigger an output publish.
    pub fn process_message(
        &mut self,
        topic: &str,
        payload: &str,
        config: &AppConfig,
    ) -> Vec<ProcessorAction> {
        let mut actions = Vec::new();
        self.previous_output_data = self.output_data.clone();

        if topic == topic_lightning_count() {
            if !self.lightning_count_loaded {
                if let Ok(count) = payload.parse::<i64>()
                    && self.total_lightning_strikes == -1 {
                        self.total_lightning_strikes = count;
                        actions.push(ProcessorAction::Publish(
                            topic_lightning_count(),
                            self.total_lightning_strikes.to_string(),
                            true,
                        ));
                    }
                self.lightning_count_loaded = true;
            }
            actions.push(ProcessorAction::Unsubscribe(topic_lightning_count()));
            return actions;
        }

        if topic == topic_rain_24h() {
            if !self.rain_24h_loaded {
                if let Ok(events) = serde_json::from_str::<Vec<(f64, f64)>>(payload) {
                    let now = now_secs();
                    self.rain_events = events
                        .into_iter()
                        .filter(|(ts, _)| now - ts < RAIN_WINDOW_SECONDS)
                        .collect();
                    let total: f64 = self.rain_events.iter().map(|(_, d)| d).sum();
                    self.output_data.rain = Some(round2(total));
                }
                self.rain_24h_loaded = true;
            }
            actions.push(ProcessorAction::Unsubscribe(topic_rain_24h()));
            return actions;
        }

        if topic == config.input_topic.weather {
            if let Ok(data) = serde_json::from_str::<WeatherData>(payload) {
                self.process_weather(&data, &mut actions);
            } else {
                log::warn!("Failed to parse weather data: {payload}");
                return actions;
            }
        } else if topic == config.input_topic.indoor {
            if let Ok(data) = serde_json::from_str::<IndoorData>(payload) {
                self.process_indoor(&data);
            } else {
                log::warn!("Failed to parse indoor data: {payload}");
                return actions;
            }
        } else if topic == config.input_topic.particle_sensor {
            if let Ok(data) = serde_json::from_str::<ParticleSensorData>(payload) {
                self.process_particle_sensor(&data);
            } else {
                log::warn!("Failed to parse particle sensor data: {payload}");
                return actions;
            }
        } else if topic == config.input_topic.lightning {
            if let Ok(data) = serde_json::from_str::<LightningData>(payload) {
                if !self.process_lightning(&data, &mut actions) {
                    return actions;
                }
            } else {
                log::warn!("Failed to parse lightning data: {payload}");
                return actions;
            }
        } else if topic == config.input_topic.light {
            if let Ok(data) = serde_json::from_str::<LightData>(payload) {
                self.process_light(&data);
            } else {
                log::warn!("Failed to parse light data: {payload}");
                return actions;
            }
        } else if topic == config.input_topic.pressure {
            if let Ok(data) = serde_json::from_str::<PressureData>(payload) {
                self.process_pressure(&data);
            } else {
                log::warn!("Failed to parse pressure data: {payload}");
                return actions;
            }
        } else if topic == config.input_topic.co2 {
            if let Ok(data) = serde_json::from_str::<Co2Data>(payload) {
                self.process_co2(&data);
            } else {
                log::warn!("Failed to parse CO2 data: {payload}");
                return actions;
            }
        } else {
            return actions;
        }

        if sanity_check(&self.output_data, &self.previous_output_data) {
            self.output_data.date_time = Some(now_secs());
            if let Ok(json) = serde_json::to_string(&self.output_data) {
                actions.push(ProcessorAction::Publish(
                    config.output_topic.clone(),
                    json,
                    false,
                ));
            }
        }

        actions
    }

    fn process_weather(&mut self, data: &WeatherData, actions: &mut Vec<ProcessorAction>) {
        if let Some(battery_ok) = data.battery_ok {
            self.output_data.out_temp_battery_status = Some(if battery_ok { 0 } else { 1 });
        }
        if let Some(temp_f) = data.temperature_f {
            self.output_data.out_temp = Some(round1(f_to_c(temp_f)));
        }
        if let Some(humidity) = data.humidity {
            self.output_data.out_humidity = Some(humidity);
        }
        if let Some(wind_dir) = data.wind_dir_deg {
            self.output_data.wind_dir = Some(wind_dir);
        }
        if let Some(wind_avg) = data.wind_avg_m_s {
            self.output_data.wind_speed = Some(wind_avg);
        }
        if let Some(wind_max) = data.wind_max_m_s {
            self.output_data.wind_gust = Some(wind_max);
        }
        if let Some(uv) = data.uv {
            self.output_data.uv = Some(uv / 10.0);
        }
        if let Some(rssi) = data.rssi {
            self.output_data.out_rssi = Some(rssi);
        }
        if let Some(snr) = data.snr {
            self.output_data.out_snr = Some(snr);
        }
        if let Some(noise) = data.noise {
            self.output_data.out_noise = Some(noise);
        }
        if let Some(light_lux) = data.light_lux {
            self.output_data.radiation = Some(light_lux / 126.7);
        }

        // Derived calculations
        if let (Some(temp_f), Some(humidity)) = (data.temperature_f, data.humidity) {
            self.output_data.heatindex = Some(round1(f_to_c(heat_index(temp_f, humidity))));

            self.output_data.dewpoint =
                Some(round1(f_to_c(dew_point(temp_f, humidity))));
        }

        if let (Some(temp_f), Some(wind_avg)) = (data.temperature_f, data.wind_avg_m_s) {
            let (wc, applicable) = wind_chill(temp_f, mps_to_mph(wind_avg));
            if applicable {
                self.output_data.windchill = Some(round1(f_to_c(wc)));
            } else {
                self.output_data.windchill = None;
            }
        }

        if let (Some(out_temp), Some(dewpoint)) = (self.output_data.out_temp, self.output_data.dewpoint) {
            self.output_data.frostpoint = Some(round1(f_to_c(frost_point(
                c_to_k(out_temp),
                c_to_k(dewpoint),
            ))));

            self.output_data.cloudbase = Some(round1(
                cloudbase(out_temp, dewpoint) + self.sensor_height_m + self.elevation_m,
            ));
        }

        // Rain accumulation
        if let Some(rain_mm) = data.rain_mm {
            let now = now_secs();
            if self.rain < 0.0 {
                // First reading — establish baseline
                self.rain = rain_mm;
            } else if self.rain < rain_mm {
                // Rain increased — record the delta
                let delta = rain_mm - self.rain;
                self.rain = rain_mm;
                self.rain_events.push_back((now, delta));
            } else if self.rain > rain_mm {
                // Rain counter reset — re-establish baseline
                self.rain = rain_mm;
            }

            // Prune events older than 24h
            while self
                .rain_events
                .front()
                .is_some_and(|(ts, _)| now - ts >= RAIN_WINDOW_SECONDS)
            {
                self.rain_events.pop_front();
            }

            let total: f64 = self.rain_events.iter().map(|(_, d)| d).sum();
            self.output_data.rain = Some(round2(total));

            // Persist rain events
            if let Ok(json) = serde_json::to_string(&self.rain_events.iter().collect::<Vec<_>>()) {
                actions.push(ProcessorAction::Publish(topic_rain_24h(), json, true));
            }
        }
    }

    fn process_indoor(&mut self, data: &IndoorData) {
        // Ignore values outside reasonable indoor ranges
        if data.temperature < 0.0 || data.temperature > 50.0 || data.humidity < 0.0 || data.humidity > 100.0
        {
            return;
        }
        self.output_data.in_temp = Some(round1(data.temperature));
        self.output_data.in_humidity = Some(data.humidity);
        if let Some(tvoc) = data.tvoc {
            self.output_data.tvoc = Some(round4(tvoc * 0.001));
        }
    }

    fn process_particle_sensor(&mut self, data: &ParticleSensorData) {
        if let Some(pm10) = data.pm10 {
            self.output_data.pm1_0 = Some(round2(pm10));
        }
        if let Some(pm25) = data.pm25 {
            self.output_data.pm2_5 = Some(round2(pm25));
        }
    }

    /// Returns false if the caller should skip the output publish (presence=false case).
    fn process_lightning(
        &mut self,
        data: &LightningData,
        actions: &mut Vec<ProcessorAction>,
    ) -> bool {
        if data.presence == Some(false) {
            self.output_data.lightning_energy = None;
            self.output_data.lightning_distance = None;
            return false;
        }

        if self.total_lightning_strikes == -1 {
            self.total_lightning_strikes = 0;
        }
        self.total_lightning_strikes += 1;
        self.output_data.lightning_strike_count = Some(self.total_lightning_strikes);
        actions.push(ProcessorAction::Publish(
            topic_lightning_count(),
            self.total_lightning_strikes.to_string(),
            true,
        ));

        if let Some(energy) = data.energy {
            self.output_data.lightning_energy = Some(energy);
            self.output_data.lightning_distance = Some(round1(2100.0 / energy.sqrt()));
        }

        true
    }

    fn process_light(&mut self, data: &LightData) {
        if let Some(lux) = data.lux {
            self.output_data.luminosity = Some(lux);
        }
    }

    fn process_pressure(&mut self, data: &PressureData) {
        if let Some(pressure) = data.pressure {
            self.output_data.barometer = Some(round2(pressure));
        }
    }

    fn process_co2(&mut self, data: &Co2Data) {
        if let Some(co2) = data.co2 {
            self.output_data.co2 = Some(co2);
        }
    }
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

pub async fn run(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut mqtt_options = MqttOptions::new("mqtt-wx", &config.mqtt.host, config.mqtt.port);
    mqtt_options.set_keep_alive(std::time::Duration::from_secs(60));

    if !config.mqtt.username.is_empty() {
        mqtt_options.set_credentials(&config.mqtt.username, &config.mqtt.password);
    }

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    // Subscribe to all topics on connect
    let topics = vec![
        topic_lightning_count(),
        topic_rain_24h(),
        config.input_topic.weather.clone(),
        config.input_topic.indoor.clone(),
        config.input_topic.lightning.clone(),
        config.input_topic.light.clone(),
        config.input_topic.pressure.clone(),
        config.input_topic.particle_sensor.clone(),
        config.input_topic.co2.clone(),
    ];

    let mut processor = Processor::new(config.sensor_height_m, config.elevation_m);

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            log::info!("Received SIGINT, shutting down...");
            let _ = shutdown_tx.send(()).await;
        }
        drop(shutdown_tx);
    });

    let mut subscribed = false;

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                log::info!("Shutting down MQTT client...");
                client.disconnect().await?;
                break;
            }
            event = eventloop.poll() => {
                match event {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        log::info!("Connected to MQTT broker");
                        if !subscribed {
                            for topic in &topics {
                                client.subscribe(topic, QoS::AtMostOnce).await?;
                            }
                            subscribed = true;
                        }
                    }
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        let topic = &publish.topic;
                        let payload = match std::str::from_utf8(&publish.payload) {
                            Ok(s) => s,
                            Err(_) => {
                                log::warn!("Non-UTF8 payload on topic {topic}");
                                continue;
                            }
                        };

                        let actions = processor.process_message(topic, payload, &config);

                        for action in actions {
                            match action {
                                ProcessorAction::Publish(t, p, retain) => {
                                    if let Err(e) = client.publish(&t, QoS::AtMostOnce, retain, p.as_bytes()).await {
                                        log::error!("Failed to publish to {t}: {e}");
                                    }
                                }
                                ProcessorAction::Unsubscribe(t) => {
                                    if let Err(e) = client.unsubscribe(&t).await {
                                        log::error!("Failed to unsubscribe from {t}: {e}");
                                    }
                                }
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("MQTT error: {e}");
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig {
            mqtt: crate::config::MqttConfig {
                host: "localhost".into(),
                port: 1883,
                username: String::new(),
                password: String::new(),
            },
            input_topic: crate::config::InputTopicConfig {
                weather: "weather".into(),
                indoor: "indoor".into(),
                lightning: "lightning".into(),
                light: "light".into(),
                pressure: "pressure".into(),
                particle_sensor: "particle_sensor".into(),
                co2: "co2".into(),
            },
            output_topic: "processed".into(),
            sensor_height_m: 2.7432,
            elevation_m: 363.2,
        }
    }

    #[test]
    fn test_weather_basic_fields() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{
            "battery_ok": true,
            "temperature_F": 72.5,
            "humidity": 65,
            "wind_dir_deg": 180,
            "wind_avg_m_s": 3.5,
            "wind_max_m_s": 5.2,
            "uv": 450,
            "rssi": -110,
            "snr": 8.5,
            "noise": -120,
            "light_lux": 25000,
            "rain_mm": 0
        }"#;

        let actions = proc.process_message("weather", payload, &config);

        assert_eq!(proc.output_data.out_temp_battery_status, Some(0));
        assert!(proc.output_data.out_temp.is_some());
        assert_eq!(proc.output_data.out_humidity, Some(65.0));
        assert_eq!(proc.output_data.wind_dir, Some(180.0));
        assert_eq!(proc.output_data.wind_speed, Some(3.5));
        assert_eq!(proc.output_data.wind_gust, Some(5.2));
        assert_eq!(proc.output_data.uv, Some(45.0));
        assert_eq!(proc.output_data.out_rssi, Some(-110.0));
        assert_eq!(proc.output_data.out_snr, Some(8.5));
        assert_eq!(proc.output_data.out_noise, Some(-120.0));
        assert!(proc.output_data.radiation.is_some());
        assert!(proc.output_data.heatindex.is_some());
        assert!(proc.output_data.dewpoint.is_some());
        assert!(proc.output_data.frostpoint.is_some());
        assert!(proc.output_data.cloudbase.is_some());
        assert!(proc.output_data.date_time.is_some());

        // Should have a publish action for the output topic and rain persistence
        assert!(actions
            .iter()
            .any(|a| matches!(a, ProcessorAction::Publish(t, _, false) if t == "processed")));
    }

    #[test]
    fn test_weather_temperature_conversion() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"temperature_F": 32.0, "humidity": 50}"#;

        proc.process_message("weather", payload, &config);

        // 32°F = 0°C
        assert_eq!(proc.output_data.out_temp, Some(0.0));
    }

    #[test]
    fn test_weather_battery_not_ok() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"battery_ok": false}"#;

        proc.process_message("weather", payload, &config);

        assert_eq!(proc.output_data.out_temp_battery_status, Some(1));
    }

    #[test]
    fn test_weather_uv_scaling() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"uv": 100}"#;

        proc.process_message("weather", payload, &config);

        assert_eq!(proc.output_data.uv, Some(10.0));
    }

    #[test]
    fn test_weather_radiation_conversion() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"light_lux": 1267.0}"#;

        proc.process_message("weather", payload, &config);

        assert!((proc.output_data.radiation.unwrap() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_weather_wind_chill_applicable() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        // Cold temp + wind → wind chill should apply
        let payload = r#"{"temperature_F": 30.0, "wind_avg_m_s": 5.0, "humidity": 50}"#;

        proc.process_message("weather", payload, &config);

        assert!(proc.output_data.windchill.is_some());
    }

    #[test]
    fn test_weather_wind_chill_not_applicable() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        // Warm temp → no wind chill
        let payload = r#"{"temperature_F": 72.0, "wind_avg_m_s": 5.0, "humidity": 50}"#;

        proc.process_message("weather", payload, &config);

        assert!(proc.output_data.windchill.is_none());
    }

    #[test]
    fn test_rain_accumulation() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        // First reading — baseline
        proc.process_message("weather", r#"{"rain_mm": 10.0}"#, &config);
        assert_eq!(proc.output_data.rain, Some(0.0));

        // Rain increases
        proc.process_message("weather", r#"{"rain_mm": 12.5}"#, &config);
        assert_eq!(proc.output_data.rain, Some(2.5));

        // More rain
        proc.process_message("weather", r#"{"rain_mm": 13.0}"#, &config);
        assert_eq!(proc.output_data.rain, Some(3.0));
    }

    #[test]
    fn test_rain_counter_reset() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        // Establish baseline
        proc.process_message("weather", r#"{"rain_mm": 100.0}"#, &config);
        // Some rain
        proc.process_message("weather", r#"{"rain_mm": 102.0}"#, &config);
        assert_eq!(proc.output_data.rain, Some(2.0));

        // Counter resets to 0
        proc.process_message("weather", r#"{"rain_mm": 0.0}"#, &config);
        // Rain total should be preserved since we just re-established baseline
        assert_eq!(proc.output_data.rain, Some(2.0));

        // New rain after reset
        proc.process_message("weather", r#"{"rain_mm": 1.5}"#, &config);
        assert_eq!(proc.output_data.rain, Some(3.5));
    }

    #[test]
    fn test_rain_no_change() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        proc.process_message("weather", r#"{"rain_mm": 10.0}"#, &config);
        proc.process_message("weather", r#"{"rain_mm": 10.0}"#, &config);
        assert_eq!(proc.output_data.rain, Some(0.0));
    }

    #[test]
    fn test_indoor_valid() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"temperature": 21.5, "humidity": 55, "tvoc": 500}"#;

        proc.process_message("indoor", payload, &config);

        assert_eq!(proc.output_data.in_temp, Some(21.5));
        assert_eq!(proc.output_data.in_humidity, Some(55.0));
        assert_eq!(proc.output_data.tvoc, Some(0.5));
    }

    #[test]
    fn test_indoor_negative_temp_rejected() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"temperature": -5.0, "humidity": 55}"#;

        proc.process_message("indoor", payload, &config);

        assert!(proc.output_data.in_temp.is_none());
    }

    #[test]
    fn test_indoor_high_temp_rejected() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"temperature": 55.0, "humidity": 55}"#;

        proc.process_message("indoor", payload, &config);

        assert!(proc.output_data.in_temp.is_none());
    }

    #[test]
    fn test_indoor_bad_humidity_rejected() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"temperature": 21.0, "humidity": 110}"#;

        proc.process_message("indoor", payload, &config);

        assert!(proc.output_data.in_temp.is_none());
    }

    #[test]
    fn test_particle_sensor() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"pm10": 12.5, "pm25": 8.3}"#;

        proc.process_message("particle_sensor", payload, &config);

        assert_eq!(proc.output_data.pm1_0, Some(12.5));
        assert_eq!(proc.output_data.pm2_5, Some(8.3));
    }

    #[test]
    fn test_lightning_with_presence() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"presence": true, "energy": 5000}"#;

        let actions = proc.process_message("lightning", payload, &config);

        assert_eq!(proc.output_data.lightning_strike_count, Some(1));
        assert_eq!(proc.output_data.lightning_energy, Some(5000.0));
        assert!(proc.output_data.lightning_distance.is_some());
        // Should publish lightning count
        assert!(actions.iter().any(
            |a| matches!(a, ProcessorAction::Publish(t, _, true) if t == &topic_lightning_count())
        ));
    }

    #[test]
    fn test_lightning_no_presence() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        // First set some lightning data
        proc.process_message("lightning", r#"{"presence": true, "energy": 5000}"#, &config);
        assert!(proc.output_data.lightning_energy.is_some());

        // Now presence=false should clear energy and distance
        proc.process_message("lightning", r#"{"presence": false}"#, &config);
        assert!(proc.output_data.lightning_energy.is_none());
        assert!(proc.output_data.lightning_distance.is_none());
    }

    #[test]
    fn test_lightning_strike_counting() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        proc.process_message("lightning", r#"{"presence": true, "energy": 1000}"#, &config);
        assert_eq!(proc.output_data.lightning_strike_count, Some(1));

        proc.process_message("lightning", r#"{"presence": true, "energy": 2000}"#, &config);
        assert_eq!(proc.output_data.lightning_strike_count, Some(2));

        proc.process_message("lightning", r#"{"presence": true, "energy": 3000}"#, &config);
        assert_eq!(proc.output_data.lightning_strike_count, Some(3));
    }

    #[test]
    fn test_lightning_distance_calculation() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        // energy=2100^2 = 4410000 → distance = 2100/2100 = 1.0
        let payload = r#"{"presence": true, "energy": 4410000}"#;

        proc.process_message("lightning", payload, &config);

        assert_eq!(proc.output_data.lightning_distance, Some(1.0));
    }

    #[test]
    fn test_light_sensor() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"lux": 45000}"#;

        proc.process_message("light", payload, &config);

        assert_eq!(proc.output_data.luminosity, Some(45000.0));
    }

    #[test]
    fn test_pressure_sensor() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"pressure": 1013.258}"#;

        proc.process_message("pressure", payload, &config);

        assert_eq!(proc.output_data.barometer, Some(1013.26));
    }

    #[test]
    fn test_co2_sensor() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{"co2": 420}"#;

        proc.process_message("co2", payload, &config);

        assert_eq!(proc.output_data.co2, Some(420.0));
    }

    #[test]
    fn test_retained_lightning_count() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        let actions = proc.process_message("mqtt-wx/lightning_count", "42", &config);

        assert_eq!(proc.total_lightning_strikes, 42);
        // Should unsubscribe
        assert!(actions.iter().any(
            |a| matches!(a, ProcessorAction::Unsubscribe(t) if t == &topic_lightning_count())
        ));
    }

    #[test]
    fn test_retained_rain_24h() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let now = now_secs();
        let events = vec![(now - 100.0, 1.5), (now - 200.0, 2.0)];
        let payload = serde_json::to_string(&events).unwrap();

        let actions = proc.process_message("mqtt-wx/rain_24h", &payload, &config);

        assert_eq!(proc.output_data.rain, Some(3.5));
        assert_eq!(proc.rain_events.len(), 2);
        // Should unsubscribe
        assert!(actions
            .iter()
            .any(|a| matches!(a, ProcessorAction::Unsubscribe(t) if t == &topic_rain_24h())));
    }

    #[test]
    fn test_retained_rain_24h_prunes_old() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let now = now_secs();
        let events = vec![
            (now - 100000.0, 1.5), // older than 24h, should be pruned
            (now - 100.0, 2.0),    // recent, should be kept
        ];
        let payload = serde_json::to_string(&events).unwrap();

        proc.process_message("mqtt-wx/rain_24h", &payload, &config);

        assert_eq!(proc.output_data.rain, Some(2.0));
        assert_eq!(proc.rain_events.len(), 1);
    }

    #[test]
    fn test_sanity_check_blocks_publish() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        // Set initial reasonable values
        proc.process_message(
            "weather",
            r#"{"temperature_F": 72.0, "humidity": 50}"#,
            &config,
        );

        // Now send wildly different temp → should fail sanity check
        let actions = proc.process_message(
            "weather",
            r#"{"temperature_F": 212.0, "humidity": 50}"#,
            &config,
        );

        // No publish to output topic expected
        assert!(!actions
            .iter()
            .any(|a| matches!(a, ProcessorAction::Publish(t, _, false) if t == "processed")));
    }

    #[test]
    fn test_output_json_field_names() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        let payload = r#"{
            "battery_ok": true,
            "temperature_F": 72.5,
            "humidity": 65
        }"#;

        let actions = proc.process_message("weather", payload, &config);

        let publish_action = actions
            .iter()
            .find(|a| matches!(a, ProcessorAction::Publish(t, _, false) if t == "processed"));
        assert!(publish_action.is_some());

        if let Some(ProcessorAction::Publish(_, json, _)) = publish_action {
            // Verify Python-compatible field names
            assert!(json.contains("\"outTemp\""));
            assert!(json.contains("\"outHumidity\""));
            assert!(json.contains("\"outTempBatteryStatus\""));
            assert!(json.contains("\"dateTime\""));
            assert!(json.contains("\"heatindex\""));
            assert!(json.contains("\"dewpoint\""));
        }
    }

    #[test]
    fn test_unknown_topic_ignored() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        let actions = proc.process_message("unknown/topic", r#"{"foo": "bar"}"#, &config);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_invalid_json_handled() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);

        let actions = proc.process_message("weather", "not json", &config);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_partial_weather_data() {
        let config = test_config();
        let mut proc = Processor::new(2.7432, 363.2);
        // Only temperature, no humidity
        let payload = r#"{"temperature_F": 72.0}"#;

        proc.process_message("weather", payload, &config);

        assert!(proc.output_data.out_temp.is_some());
        assert!(proc.output_data.out_humidity.is_none());
        // No heat index/dewpoint without humidity
        assert!(proc.output_data.heatindex.is_none());
        assert!(proc.output_data.dewpoint.is_none());
    }
}
