use std::{fmt::Debug, fs, io, path::Path};

use clap::Parser;
use streamserver::app;
use tracing::{info, span, Level};
use tracing_appender::rolling::Rotation;
use tracing_subscriber::{
    fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt, Layer, Registry,
};

#[derive(Debug, Clone, clap::Parser)]
struct Opts {
    #[clap(short, long)]
    config: Option<String>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let opt = Opts::parse();
    let app_config =
        streamserver::config::AppConfig::load_from_file(opt.config.as_deref(), "STREAMSERVER")
            .expect("Failed to load config");
    // println!("config: {:?}", app_config);

    let log_level = streamserver::config::log::str_to_level(&app_config.log.level);
    let log_file = if app_config.log.file.is_empty() {
        None
    } else {
        Some(app_config.log.file.clone())
    };

    // init console logger
    let mut layers: Box<dyn Layer<Registry> + Send + Sync> = if app_config.debug {
        tracing_subscriber::fmt::layer()
            .pretty()
            .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
            .with_writer(io::stdout.with_max_level(Level::DEBUG))
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
            .with_writer(io::stdout.with_max_level(log_level))
            .boxed()
    };

    // init file json logger
    if let Some(f) = log_file {
        let path = std::path::Path::new(&f);
        let directory = path.parent().expect("failed to parse log file");
        let base_filename = path.file_name().expect("failed to parse log file");

        if !Path::exists(directory) {
            fs::create_dir_all(directory).expect("failed to create log directory");
        }

        let layer = if app_config.log.rotate {
            let wrt = tracing_appender::rolling::RollingFileAppender::new(
                Rotation::DAILY,
                directory,
                base_filename,
            )
            .with_max_level(log_level);
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(wrt)
                .boxed()
        } else {
            let wrt = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)
                .expect("failed to open log file");
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(wrt)
                .boxed()
        };
        layers = layers.and_then(layer).boxed();
    }

    let log_config = std::env::var("STREAM_LOG_EXTRA").unwrap_or("xwebrtc=info".to_string());
    let layers = layers.with_filter(
        tracing_subscriber::filter::EnvFilter::builder()
            .with_default_directive(log_level.into())
            .parse_lossy("tracing,".to_string() + &log_config),
    );
    Registry::default().with(layers).init();

    let span = span!(Level::TRACE, "bootstrap");
    let _enter = span.enter();

    // register api server
    let api_cfg = app_config.server.clone();
    streamserver::services::state::streamserver::register_to_apiserver(&api_cfg)
            .await
            .expect("can't connect to apiserver");

    tokio::spawn(async move {
        app::start_app(&app_config).await;
    });

    // graceful shutdown
    tokio::signal::ctrl_c().await.expect("failed to install signal handler");
    info!("shutting down");

    streamserver::services::state::streamserver::unregister_to_apiserver(&api_cfg)
        .await
        .expect("can't connect to apiserver");

}
