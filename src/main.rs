mod log_ingestion;
use crate::log_ingestion::start_syslog_server;

#[tokio::main]
async fn main() {
    if let Err(e) = start_syslog_server(5140, 5140).await {
        eprintln!("Syslog server error: {}", e);
    }
}