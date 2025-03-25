mod db;
mod log_ingestion;

use db::init_db;
use log_ingestion::start_syslog_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_pool = init_db().await?;

    start_syslog_server(514, 514, db_pool).await?;

    Ok(())
}
