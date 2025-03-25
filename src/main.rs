mod db;
mod syslog_ingestion;
mod log_parser;
mod file_based_ingestion;

use db::init_db;
use syslog_ingestion::start_syslog_server;
use file_based_ingestion::ingest_log_file;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_pool = init_db().await?;

    let db1 = db_pool.clone();
    let db2 = db_pool.clone();

    let syslog_task = tokio::spawn(async move {
        run_syslog_ingestor(&db1).await
    });

    let file_task = tokio::spawn(async move {
        run_file_ingestor(&db2).await
    });

    let _ = tokio::try_join!(syslog_task, file_task)?;

    Ok(())
}


async fn run_syslog_ingestor(db_pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    start_syslog_server(514, 514, db_pool.clone()).await?;
    Ok(())
}

async fn run_file_ingestor(db_pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ingest_log_file("./yourlogfile.log", db_pool.clone()).await?;
    Ok(())
}
// ─────────────────────────────────────────