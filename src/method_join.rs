use tokio::{io::AsyncWriteExt, net::TcpStream};
use uuid::Uuid;

use crate::mc_packet_utils::{send_handshake, write_string, write_u8_varint};

pub async fn send_join(stream: &mut TcpStream, port: &u16, hostname: &str) {
    let protocol_version = 770;

    // Switch to login state (2)
    send_handshake(stream, protocol_version, hostname, *port, 2).await;

    // Send login start packet
    send_login_start(stream, "test").await;

    //_ = conn.write_packet(&Packet::empty(0x03)).await;
}

// Send login start packet
pub async fn send_login_start(stream: &mut TcpStream, username: &str) {
    _ = write_u8_varint(stream, 0x00).await;

    _ = write_string(stream, username).await;
    _ = stream.write_all(Uuid::default().as_bytes()).await; // No UUID for offline mode
}
