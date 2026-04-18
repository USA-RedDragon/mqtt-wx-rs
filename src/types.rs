use serde::{Deserialize, Serialize};

/// Output data published to the output MQTT topic.
/// All field names match the Python version's JSON output for backward compatibility.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OutputData {
    #[serde(rename = "dateTime", skip_serializing_if = "Option::is_none")]
    pub date_time: Option<f64>,

    #[serde(rename = "outTemp", skip_serializing_if = "Option::is_none")]
    pub out_temp: Option<f64>,

    #[serde(rename = "outHumidity", skip_serializing_if = "Option::is_none")]
    pub out_humidity: Option<f64>,

    #[serde(rename = "outTempBatteryStatus", skip_serializing_if = "Option::is_none")]
    pub out_temp_battery_status: Option<i32>,

    #[serde(rename = "windDir", skip_serializing_if = "Option::is_none")]
    pub wind_dir: Option<f64>,

    #[serde(rename = "windSpeed", skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f64>,

    #[serde(rename = "windGust", skip_serializing_if = "Option::is_none")]
    pub wind_gust: Option<f64>,

    #[serde(rename = "UV", skip_serializing_if = "Option::is_none")]
    pub uv: Option<f64>,

    #[serde(rename = "outRSSI", skip_serializing_if = "Option::is_none")]
    pub out_rssi: Option<f64>,

    #[serde(rename = "outSNR", skip_serializing_if = "Option::is_none")]
    pub out_snr: Option<f64>,

    #[serde(rename = "outNoise", skip_serializing_if = "Option::is_none")]
    pub out_noise: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub radiation: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub heatindex: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub windchill: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dewpoint: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frostpoint: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloudbase: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain: Option<f64>,

    #[serde(rename = "inTemp", skip_serializing_if = "Option::is_none")]
    pub in_temp: Option<f64>,

    #[serde(rename = "inHumidity", skip_serializing_if = "Option::is_none")]
    pub in_humidity: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tvoc: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pm1_0: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pm2_5: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightning_strike_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightning_energy: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightning_distance: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub luminosity: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub barometer: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub co2: Option<f64>,
}

/// Input data from the weather station topic.
#[derive(Debug, Deserialize)]
pub struct WeatherData {
    pub battery_ok: Option<bool>,
    #[serde(rename = "temperature_F")]
    pub temperature_f: Option<f64>,
    pub humidity: Option<f64>,
    pub wind_dir_deg: Option<f64>,
    pub wind_avg_m_s: Option<f64>,
    pub wind_max_m_s: Option<f64>,
    pub uv: Option<f64>,
    pub rssi: Option<f64>,
    pub snr: Option<f64>,
    pub noise: Option<f64>,
    pub light_lux: Option<f64>,
    pub rain_mm: Option<f64>,
}

/// Input data from the indoor sensor topic.
#[derive(Debug, Deserialize)]
pub struct IndoorData {
    pub temperature: f64,
    pub humidity: f64,
    pub tvoc: Option<f64>,
}

/// Input data from the lightning detector topic.
#[derive(Debug, Deserialize)]
pub struct LightningData {
    pub presence: Option<bool>,
    pub energy: Option<f64>,
}

/// Input data from the light sensor topic.
#[derive(Debug, Deserialize)]
pub struct LightData {
    pub lux: Option<f64>,
}

/// Input data from the pressure sensor topic.
#[derive(Debug, Deserialize)]
pub struct PressureData {
    pub pressure: Option<f64>,
}

/// Input data from the particle sensor topic.
#[derive(Debug, Deserialize)]
pub struct ParticleSensorData {
    pub pm10: Option<f64>,
    pub pm25: Option<f64>,
}

/// Input data from the CO2 sensor topic.
#[derive(Debug, Deserialize)]
pub struct Co2Data {
    pub co2: Option<f64>,
}
