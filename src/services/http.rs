use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::ForensicEvent;

pub async fn handle_http(mut socket: TcpStream, addr: std::net::SocketAddr, port: u16) {
    let mut buf = vec![0u8; 4096];
    let n = socket.read(&mut buf).await.unwrap_or(0);
    let response = b"HTTP/1.1 200 OK\r\n\
        Server: Apache/2.4.57 (Ubuntu)\r\n\
        Content-Type: text/html; charset=UTF-8\r\n\
        X-Powered-By: PHP/8.1.2\r\n\
        Connection: close\r\n\
        \r\n\
        <html><body><h1>It works!</h1></body></html>\r\n";

        let _ = socket.write_all(response).await;

        let evento = ForensicEvent::new(
            addr.ip().to_string(),
            format!("0.0.0.0:{}",port),
            addr.port(),
            port,
            &buf[..n],
        );
        evento.save();
        println!("[HTTP] {} | {} bytes | request capturado", addr.ip(), n);
}