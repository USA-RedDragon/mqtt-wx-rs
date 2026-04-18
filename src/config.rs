use configulator::{
    CLIFlagOptions, Config, Configulator, EnvironmentVariableOptions, FileOptions, Validate,
    serde_loader,
};

#[derive(Config, Default, Debug)]
pub struct MqttConfig {
    #[configulator(name = "host", default = "localhost", description = "MQTT broker hostname")]
    pub host: String,

    #[configulator(name = "port", default = "1883", description = "MQTT broker port")]
    pub port: u16,

    #[configulator(name = "username", default = "", description = "MQTT username")]
    pub username: String,

    #[configulator(name = "password", default = "", description = "MQTT password")]
    pub password: String,
}

#[derive(Config, Default, Debug)]
pub struct InputTopicConfig {
    #[configulator(
        name = "weather",
        default = "weather",
        description = "Input topic for weather station data"
    )]
    pub weather: String,

    #[configulator(
        name = "indoor",
        default = "indoor",
        description = "Input topic for indoor sensor data"
    )]
    pub indoor: String,

    #[configulator(
        name = "lightning",
        default = "lightning",
        description = "Input topic for lightning data"
    )]
    pub lightning: String,

    #[configulator(
        name = "light",
        default = "light",
        description = "Input topic for light data"
    )]
    pub light: String,

    #[configulator(
        name = "pressure",
        default = "pressure",
        description = "Input topic for pressure data"
    )]
    pub pressure: String,

    #[configulator(
        name = "particle-sensor",
        default = "particle_sensor",
        description = "Input topic for particle sensor data"
    )]
    pub particle_sensor: String,

    #[configulator(
        name = "co2",
        default = "co2",
        description = "Input topic for CO2 sensor data"
    )]
    pub co2: String,
}

#[derive(Config, Default, Debug)]
pub struct AppConfig {
    #[configulator(name = "mqtt")]
    pub mqtt: MqttConfig,

    #[configulator(name = "input-topic")]
    pub input_topic: InputTopicConfig,

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
        if self.mqtt.host.is_empty() {
            return Err("mqtt.host must not be empty".into());
        }
        if self.mqtt.port == 0 {
            return Err("mqtt.port must be non-zero".into());
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
                "config.yml".into(),
                "/etc/mqtt-wx/config.yaml".into(),
                "/etc/mqtt-wx/config.yml".into(),
                "/config.yaml".into(),
                "/config.yml".into(),
                "/mqtt-wx.yaml".into(),
                "/mqtt-wx.yml".into(),
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
