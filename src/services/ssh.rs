use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::ForensicEvent;

pub async fn handle_ssh(mut socket: TcpStream, addr: std::net::SocketAddr, port: u16) {
    let banner = b"SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.6\r\n";
    if  socket.write_all(banner).await.is_err() { return; }

    let mut buf =  vec![0u8; 4096];
    let n= socket.read(&mut buf).await.unwrap_or(0);

    let event = ForensicEvent::new(
        addr.ip().to_string(),
        format!("0.0.0.0:{}", port),
        addr.port(),
        port,
        &buf[..n]
    );
    event.save();
    println!("[SSH] {} intento autenticarse | {} bytes", addr.ip(), n);
}