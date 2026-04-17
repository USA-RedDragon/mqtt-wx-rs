
use clap::Parser;

#[derive(Parser)]
#[command(name = "mqtt-wx")]
#[command(about = "Bridge my weather station to MQTT", long_about = None)]
#[command(version = concat!(env!("PKG_VERSION", "dev"), " (", env!("GIT_COMMIT", "unknown"), ")"))]
struct Cli {
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    log::info!(
        "mqtt-wx v{} ({})",
        env!("PKG_VERSION", "dev"),
        env!("GIT_COMMIT", "unknown"),
    );
}
