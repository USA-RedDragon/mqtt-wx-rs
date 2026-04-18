mod config;
mod meteorological;
mod mqtt_client;
mod sanity;
mod types;
mod units;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!(
        "mqtt-wx v{} ({})",
        option_env!("PKG_VERSION").unwrap_or("dev"),
        option_env!("GIT_COMMIT").unwrap_or("unknown"),
    );

    let config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load config: {e}");
            std::process::exit(1);
        }
    };

    log::info!("Connecting to MQTT broker at {}:{}", config.mqtt_host, config.mqtt_port);

    if let Err(e) = mqtt_client::run(config).await {
        log::error!("Fatal error: {e}");
        std::process::exit(1);
    }
}
