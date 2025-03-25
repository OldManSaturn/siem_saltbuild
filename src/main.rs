mod db;
mod syslog_ingestion;
mod log_parser;
mod file_based_ingestion;
mod cli;
mod config; 

use db::init_db;
use file_based_ingestion::ingest_log_file;
use tokio::sync::broadcast;
use config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = AppConfig::load()?;
    println!("Loaded config: {:?}", settings);
    let db_pool = init_db().await?;
    let config = AppConfig::load()?;
    cli::launch_cli(db_pool, config).await?;

    Ok(())
}

// Optional runners (not used directly in CLI)
async fn run_syslog_ingestor(
    db_pool: &sqlx::SqlitePool,
    shutdown: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    syslog_ingestion::start_syslog_server(514, 514, db_pool.clone(), shutdown).await?;
    Ok(())
}

async fn run_file_ingestor(
    db_pool: &sqlx::SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ingest_log_file("./yourlogfile.log", db_pool.clone()).await?;
    Ok(())
}
