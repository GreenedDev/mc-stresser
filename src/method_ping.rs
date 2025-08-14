use rust_mc_proto_tokio::MCConnTcp;

use crate::mc_packet_utils::send_handshake;

pub async fn send_ping(conn: &mut MCConnTcp, port: &u16, hostname: &str) {
    let protocol_version = 770;

    // Switch to status state (1)
    send_handshake(conn, protocol_version, hostname, *port, 1).await;
}
