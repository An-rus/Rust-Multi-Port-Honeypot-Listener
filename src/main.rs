use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use base64::{Engine, engine::general_purpose};
use std::fs::OpenOptions;
use std::io::Write;
use uuid::Uuid;

mod services;
use services::{ssh, http};

#[derive(Serialize, Deserialize, Debug)]
struct ForensicEvent {
    pub timestamp: DateTime<Utc>,
    pub source_ip: String,
    pub dest_ip: String,
    pub source_port: u16,
    pub dest_port: u16,
    pub payload_b64: String,
    pub detected_protocol: String,
    pub session_id: String,
}

impl ForensicEvent {
    pub fn new(
        source_ip: String,
        dest_ip: String,
        source_port: u16,
        dest_port: u16,
        bytes: &[u8]
    ) -> Self {
        ForensicEvent {
            timestamp: Utc::now(),
            source_ip,
            dest_ip,
            source_port,
            dest_port,
            payload_b64: general_purpose::STANDARD.encode(bytes),
            detected_protocol: detect_protocol(dest_port),
            session_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn save(&self) {
        let json = match serde_json::to_string(self) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[!] Failed to serialize forensic event: {}", e);
                return;
            }
        };

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("events.json");

        match file {
            Ok(mut f) => {
                if let Err(e) = writeln!(f, "{}", json) {
                    eprintln!("[!] Failed to write forensic event: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[!] Failed to open events.json: {}", e);
            }
        }
    }
}

fn detect_protocol(port: u16) -> String {
    match port {
        22 => "SSH".to_string(),
        80 => "HTTP".to_string(),
        21 => "FTP".to_string(),
        3306 => "MySQL".to_string(),
        _ => "Unknown".to_string(),
    }
}

async fn run_listener(port: u16) {
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[!] Failed to bind port {}: {}", port, e);
            return;
        }
    };

    println!("[+] Listening on port {}", port);

    loop {
        let (socket, addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("[!] Failed to accept connection on port {}: {}", port, e);
                continue;
            }
        };

        tokio::spawn(async move {
            match port {
                22 => ssh::handle_ssh(socket, addr, port).await,
                80 => http::handle_http(socket, addr, port).await,
                _ => {
                    // Generic ports
                    let mut buf = vec![0u8; 4096];
                    let mut s = socket;
                    let n = match s.read(&mut buf).await {
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("[!] Read error on port {} from {}: {}", port, addr, e);
                            return;
                        }
                    };

                    ForensicEvent::new(
                        addr.ip().to_string(),
                        format!("0.0.0.0:{}", port),
                        addr.port(),
                        port,
                        &buf[..n]
                    ).save();
                }
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let ports = vec![22, 80, 21, 3306];

    for port in ports {
        tokio::spawn(run_listener(port));
    }

    tokio::signal::ctrl_c().await.unwrap();
    println!("[-] Honeypot stopped");
}