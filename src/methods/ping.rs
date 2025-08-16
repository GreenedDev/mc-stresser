use rust_mc_proto_tokio::{MCConnTcp, Packet};

use crate::mc_packet_utils::send_handshake;

pub async fn send_ping(conn: &mut MCConnTcp, port: &u16, hostname: &str) {
    let protocol_version = 770;

    // Switch to login state (1)
    send_handshake(conn, protocol_version, hostname, *port, 1).await;

    _ = conn.write_packet(&Packet::empty(0x00)).await;
}
