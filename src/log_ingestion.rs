use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt};
use std::error::Error;

pub async fn start_syslog_server(tcp_port: u16, udp_port: u16) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tcp_addr = format!("0.0.0.0:{}", tcp_port);
    let udp_addr = format!("0.0.0.0:{}", udp_port);

    let tcp_task = tokio::spawn(start_tcp_syslog_server(tcp_addr));
    let udp_task = tokio::spawn(start_udp_syslog_server(udp_addr));

    let _ = tokio::try_join!(tcp_task, udp_task)?;

    Ok(())
}

async fn start_tcp_syslog_server(addr: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind(&addr).await?;
    println!("TCP Syslog server listening on {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let log_message = String::from_utf8_lossy(&buffer[..n]);
                        println!("Received TCP log: {}", log_message);
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

async fn start_udp_syslog_server(addr: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let socket = UdpSocket::bind(&addr).await?;
    println!("UDP Syslog server listening on {}", addr);

    let mut buffer = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buffer).await {
            Ok((n, src)) => {
                let log_message = String::from_utf8_lossy(&buffer[..n]);
                println!("Received UDP log from {}: {}", src, log_message);
            }
            Err(e) => eprintln!("UDP receive error: {}", e),
        }
    }
}