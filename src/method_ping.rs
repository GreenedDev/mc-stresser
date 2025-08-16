use tokio::net::TcpStream;

use crate::mc_packet_utils::send_handshake;

pub async fn send_ping(stream: &mut TcpStream, port: &u16, hostname: &str) {
    let protocol_version = 770;

    // Switch to login state (1)
    send_handshake(stream, protocol_version, hostname, *port, 1).await;
}
