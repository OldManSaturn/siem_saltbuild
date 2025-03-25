use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub syslog_tcp_port: u16,
    pub syslog_udp_port: u16,
    pub log_file_path: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?;

        cfg.try_deserialize()
    }
}
