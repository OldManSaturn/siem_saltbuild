// ─────────────────────────────────────────
// Basic syslog parser
// Parsing logic: normalize timestamps, extract IPs, etc.
// ─────────────────────────────────────────

use regex::Regex;

// Public struct so other modules can use it
#[derive(Debug)]
pub struct ParsedLog {
    pub protocol: String,
    pub source: String,
    pub timestamp: Option<String>,
    pub hostname: Option<String>,
    pub process: Option<String>,
    pub message: String,
}

// Public function to use from other files
pub fn parse_log(protocol: &str, source: &str, line: &str) -> ParsedLog {
    let mut parsed = parse_syslog_line(line);
    parsed.protocol = protocol.to_string();
    parsed.source = source.to_string();
    parsed
    

}

// Internal syslog line parser
fn parse_syslog_line(line: &str) -> ParsedLog {
    let re = Regex::new(
        r"^(?P<timestamp>\w{3}\s+\d+\s+\d{2}:\d{2}:\d{2})\s+(?P<host>\S+)\s+(?P<proc>\S+?):\s*(?P<msg>.+)$"
    ).unwrap();

    if let Some(caps) = re.captures(line) {
        ParsedLog {
            protocol: String::new(),
            source: String::new(),
            timestamp: Some(caps["timestamp"].to_string()),
            hostname: Some(caps["host"].to_string()),
            process: Some(caps["proc"].to_string()),
            message: caps["msg"].to_string(),
        }
    } else {
        ParsedLog {
            protocol: String::new(),
            source: String::new(),
            timestamp: None,
            hostname: None,
            process: None,
            message: line.trim().to_string(),
        }
    }
}
