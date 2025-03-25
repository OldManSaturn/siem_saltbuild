mod log_ingestion;

fn main() {
    log_ingestion::start_syslog_server();
}