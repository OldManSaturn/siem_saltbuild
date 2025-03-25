// this module starts a simple syslog server that listens on TCP and UDP ports
// it spawns separate threads to handle TCP and UDP connections, incoming log messages are printed to the console
use std::net::{TcpListener, UdpSocket};
use std::io::{Read, Write};
use std::thread; // needed to run the tcp and udp servers in separate threads, concurrently

pub mod log_ingestion {
    use super::*;

    pub fn start_syslog_server(tcp_port: u16, udp_port: u16) {
        let tcp_handle = thread::spawn(move || {
            if let Err(e) = start_tcp_syslog_server(tcp_port) {
                eprintln!("TCP Syslog server error: {}", e);
            }
        });

        let udp_handle = thread::spawn(move || {
            if let Err(e) = start_udp_syslog_server(udp_port) {
                eprintln!("UDP Syslog server error: {}", e);
            }
        });

        tcp_handle.join().unwrap();
        udp_handle.join().unwrap();
    }

    fn start_tcp_syslog_server(port: u16) -> std::io::Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        println!("TCP Syslog server listening on port {}", port);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        loop {
                            match stream.read(&mut buffer) {
                                Ok(0) => break, // Connection closed
                                Ok(size) => {
                                    let log_message = String::from_utf8_lossy(&buffer[..size]);
                                    println!("Received TCP log: {}", log_message);
                                    // Process the log message here
                                }
                                Err(e) => {
                                    eprintln!("Failed to read from TCP stream: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept TCP connection: {}", e),
            }
        }
        Ok(())
    }

    fn start_udp_syslog_server(port: u16) -> std::io::Result<()> {
        let socket = UdpSocket::bind(("0.0.0.0", port))?;
        println!("UDP Syslog server listening on port {}", port);

        let mut buffer = [0; 1024];
        loop {
            match socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    let log_message = String::from_utf8_lossy(&buffer[..size]);
                    println!("Received UDP log from {}: {}", src, log_message);
                    // Process the log message here
                }
                Err(e) => eprintln!("Failed to receive UDP packet: {}", e),
            }
        }
    }
}