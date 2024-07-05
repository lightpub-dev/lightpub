use std::{io::Read, path::PathBuf};

use clap::Parser;
use lapin::ConnectionProperties;
use lightpub_backend::apub::ApubReqwester;
use lightpub_config::Config;
use lightpub_worker::{ApubDirector, WorkerType};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[arg(long)]
    post_worker: u32,
    #[arg(long)]
    fetch_worker: u32,
    #[arg(long, default_value = "false")]
    generate_run_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let cli = Cli::parse();
    let config = cli.config.unwrap_or("lightpub.yml.sample".into());

    let mut file = std::fs::File::open(config).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let config: Config = serde_yaml::from_str(&contents).expect("Unable to deserialize YAML");

    // connect to db
    let conn_str = format!("sqlite:{}", config.database.path);
    let pool = SqlitePoolOptions::new()
        .connect(&conn_str)
        .await
        .expect("connect to database");
    tracing::info!("Connected to database");

    // reqwester
    let requester = || ApubReqwester::new(&config);

    // director
    let director = ApubDirector::new(pool, requester);
    director.start_workers().await;

    if let Some(run) = cli.generate_run_file {
        // generate a empty file
        std::fs::write(run, "").expect("Unable to write file");
    }

    // wait until SIGTERM
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}
