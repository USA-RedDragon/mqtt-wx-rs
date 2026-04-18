use configulator::{
    CLIFlagOptions, Config, Configulator, EnvironmentVariableOptions, FileOptions, Validate,
    serde_loader,
};

#[derive(Config, Default, Debug)]
pub struct AppConfig {
    #[configulator(name = "mqtt-host", default = "localhost", description = "MQTT broker hostname")]
    pub mqtt_host: String,

    #[configulator(name = "mqtt-port", default = "1883", description = "MQTT broker port")]
    pub mqtt_port: u16,

    #[configulator(name = "mqtt-username", default = "", description = "MQTT username")]
    pub mqtt_username: String,

    #[configulator(name = "mqtt-password", default = "", description = "MQTT password")]
    pub mqtt_password: String,

    #[configulator(
        name = "input-topic-weather",
        default = "weather",
        description = "Input topic for weather station data"
    )]
    pub input_topic_weather: String,

    #[configulator(
        name = "input-topic-indoor",
        default = "indoor",
        description = "Input topic for indoor sensor data"
    )]
    pub input_topic_indoor: String,

    #[configulator(
        name = "input-topic-lightning",
        default = "lightning",
        description = "Input topic for lightning data"
    )]
    pub input_topic_lightning: String,

    #[configulator(
        name = "input-topic-light",
        default = "light",
        description = "Input topic for light data"
    )]
    pub input_topic_light: String,

    #[configulator(
        name = "input-topic-pressure",
        default = "pressure",
        description = "Input topic for pressure data"
    )]
    pub input_topic_pressure: String,

    #[configulator(
        name = "input-topic-particle-sensor",
        default = "particle_sensor",
        description = "Input topic for particle sensor data"
    )]
    pub input_topic_particle_sensor: String,

    #[configulator(
        name = "input-topic-co2",
        default = "co2",
        description = "Input topic for CO2 sensor data"
    )]
    pub input_topic_co2: String,

    #[configulator(
        name = "output-topic",
        default = "processed",
        description = "Output topic for processed data"
    )]
    pub output_topic: String,

    #[configulator(
        name = "sensor-height-m",
        default = "2.7432",
        description = "Sensor height above ground in meters"
    )]
    pub sensor_height_m: f64,

    #[configulator(
        name = "elevation-m",
        default = "363.2",
        description = "Field elevation in meters above sea level"
    )]
    pub elevation_m: f64,
}

impl Validate for AppConfig {
    fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.mqtt_host.is_empty() {
            return Err("mqtt-host must not be empty".into());
        }
        if self.mqtt_port == 0 {
            return Err("mqtt-port must be non-zero".into());
        }
        Ok(())
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let version = format!(
        "{} ({})",
        option_env!("PKG_VERSION").unwrap_or("dev"),
        option_env!("GIT_COMMIT").unwrap_or("unknown"),
    );

    let config = Configulator::<AppConfig>::new()
        .with_file(FileOptions {
            paths: vec![
                "config.yaml".into(),
                "/etc/mqtt-wx/config.yaml".into(),
                "/config.yaml".into(),
                "/mqtt-wx.yaml".into(),
            ],
            error_if_not_found: false,
            loader: serde_loader(|s| serde_yaml_ng::from_str(s)),
        })
        .with_environment_variables(EnvironmentVariableOptions {
            prefix: "MQTT_WX".into(),
            separator: "__".into(),
        })
        .with_cli_command(clap::Command::new("mqtt-wx").version(version))
        .with_cli_flags(CLIFlagOptions {
            separator: ".".into(),
        })
        .load()?;

    Ok(config)
}
