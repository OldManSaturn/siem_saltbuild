use tokio::net::{TcpListener, UdpSocket};
use tokio::io::AsyncReadExt;
use sqlx::SqlitePool;
use std::error::Error;
use crate::log_parser::parse_log;
use tokio::sync::broadcast;

pub async fn start_syslog_server(
    tcp_port: u16,
    udp_port: u16,
    db_pool: SqlitePool,
    shutdown: broadcast::Receiver<()>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tcp_addr = format!("0.0.0.0:{}", tcp_port);
    let udp_addr = format!("0.0.0.0:{}", udp_port);

    let shutdown_tcp = shutdown.resubscribe();
    let shutdown_udp = shutdown.resubscribe();

    let tcp_task = tokio::spawn(start_tcp_syslog_server(tcp_addr, db_pool.clone(), shutdown_tcp));
    let udp_task = tokio::spawn(start_udp_syslog_server(udp_addr, db_pool, shutdown_udp));

    let _ = tokio::try_join!(tcp_task, udp_task)?;

    Ok(())
}

async fn start_tcp_syslog_server(
    addr: String,
    db_pool: SqlitePool,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind(&addr).await?;
    println!("TCP Syslog server listening on {}", addr);

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                println!("Shutting down TCP syslog server on {}", addr);
                break;
            }
            accept_result = listener.accept() => {
                let (mut socket, _) = accept_result?;
                let pool = db_pool.clone();

                tokio::spawn(async move {
                    let mut buffer = [0u8; 1024];
                    loop {
                        match socket.read(&mut buffer).await {
                            Ok(0) => break,
                            Ok(n) => {
                                let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                                println!("Received TCP log: {}", message);

                                let parsed = parse_log("TCP", "tcp connection", &message);

                                let _ = sqlx::query(
                                    "INSERT INTO logs (protocol, source, message, parsed_timestamp, hostname, process)
                                     VALUES (?, ?, ?, ?, ?, ?)"
                                )
                                .bind(&parsed.protocol)
                                .bind(&parsed.source)
                                .bind(&parsed.message)
                                .bind(&parsed.timestamp)
                                .bind(&parsed.hostname)
                                .bind(&parsed.process)
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
    }

    Ok(())
}

async fn start_udp_syslog_server(
    addr: String,
    db_pool: SqlitePool,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let socket = UdpSocket::bind(&addr).await?;
    println!("UDP Syslog server listening on {}", addr);

    let mut buffer = [0u8; 1024];

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                println!("Shutting down UDP syslog server on {}", addr);
                break;
            }

            result = socket.recv_from(&mut buffer) => {
                match result {
                    Ok((n, src)) => {
                        let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                        println!("Received UDP log from {}: {}", src, message);

                        let parsed = parse_log("UDP", &src.to_string(), &message);

                        let _ = sqlx::query(
                            "INSERT INTO logs (protocol, source, message, parsed_timestamp, hostname, process)
                             VALUES (?, ?, ?, ?, ?, ?)"
                        )
                        .bind(&parsed.protocol)
                        .bind(&parsed.source)
                        .bind(&parsed.message)
                        .bind(&parsed.timestamp)
                        .bind(&parsed.hostname)
                        .bind(&parsed.process)
                        .execute(&db_pool)
                        .await;
                    }
                    Err(e) => eprintln!("UDP receive error: {}", e),
                }
            }
        }
    }

    Ok(())
}
