use tokio::net::{TcpListener, UdpSocket};
use tokio::io::AsyncReadExt;
use sqlx::SqlitePool;
use regex::Regex;
use std::error::Error;

pub async fn start_syslog_server(
    tcp_port: u16,
    udp_port: u16,
    db_pool: SqlitePool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tcp_addr = format!("0.0.0.0:{}", tcp_port);
    let udp_addr = format!("0.0.0.0:{}", udp_port);

    let tcp_task = tokio::spawn(start_tcp_syslog_server(tcp_addr, db_pool.clone()));
    let udp_task = tokio::spawn(start_udp_syslog_server(udp_addr, db_pool));

    let _ = tokio::try_join!(tcp_task, udp_task)?;

    Ok(())
}

async fn start_tcp_syslog_server(
    addr: String,
    db_pool: SqlitePool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind(&addr).await?;
    println!("TCP Syslog server listening on {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let pool = db_pool.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                        println!("Received TCP log: {}", message);

                        let parsed = parse_syslog_line(&message);

                        let _ = sqlx::query(
                            "INSERT INTO logs (protocol, source, message, parsed_timestamp, hostname, process)
                             VALUES (?, ?, ?, ?, ?, ?)"
                        )
                        .bind("TCP")
                        .bind("tcp connection")
                        .bind(&parsed.message)
                        .bind(parsed.timestamp)
                        .bind(parsed.hostname)
                        .bind(parsed.process)
                        .execute(&pool)
                        .await;
                    }
                    Err(e) => {
                        eprintln!("TCP read error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

async fn start_udp_syslog_server(
    addr: String,
    db_pool: SqlitePool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let socket = UdpSocket::bind(&addr).await?;
    println!("UDP Syslog server listening on {}", addr);

    let mut buffer = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buffer).await {
            Ok((n, src)) => {
                let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                println!("Received UDP log from {}: {}", src, message);

                let parsed = parse_syslog_line(&message);

                let _ = sqlx::query(
                    "INSERT INTO logs (protocol, source, message, parsed_timestamp, hostname, process)
                     VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind("UDP")
                .bind(src.to_string())
                .bind(&parsed.message)
                .bind(parsed.timestamp)
                .bind(parsed.hostname)
                .bind(parsed.process)
                .execute(&db_pool)
                .await;
            }
            Err(e) => eprintln!("UDP receive error: {}", e),
        }
    }
}

// ─────────────────────────────────────────
// Basic syslog parser
// ─────────────────────────────────────────

#[derive(Debug)]
struct ParsedLog {
    timestamp: Option<String>,
    hostname: Option<String>,
    process: Option<String>,
    message: String,
}

fn parse_syslog_line(line: &str) -> ParsedLog {
    let re = Regex::new(
        r"^(?P<timestamp>\w{3}\s+\d+\s+\d{2}:\d{2}:\d{2})\s+(?P<host>\S+)\s+(?P<proc>\S+?):\s*(?P<msg>.+)$"
    ).unwrap();

    if let Some(caps) = re.captures(line) {
        ParsedLog {
            timestamp: Some(caps["timestamp"].to_string()),
            hostname: Some(caps["host"].to_string()),
            process: Some(caps["proc"].to_string()),
            message: caps["msg"].to_string(),
        }
    } else {
        ParsedLog {
            timestamp: None,
            hostname: None,
            process: None,
            message: line.trim().to_string(),
        }
    }
}
