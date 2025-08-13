use rust_mc_proto_tokio::{DataWriter, MCConnTcp, Packet};
use uuid::Uuid;

// Send handshake packet to initiate connection
pub async fn send_handshake(
    conn: &mut MCConnTcp,
    proto: u16,
    srv_addr: &str,
    srv_port: u16,
    next_state: u8,
) {
    let mut packet = Packet::empty(0x00);

    _ = packet.write_u16_varint(proto).await;
    _ = packet.write_string(srv_addr).await;
    _ = packet.write_unsigned_short(srv_port).await;
    _ = packet.write_u8_varint(next_state).await;
    _ = conn.write_packet(&packet).await;
}

// Send login start packet
pub async fn send_login_start(conn: &mut MCConnTcp, username: &str) {
    let mut packet = Packet::empty(0x00);
    _ = packet.write_string(username).await;
    _ = packet.write_uuid(&Uuid::default()).await; // No UUID for offline mode
    _ = conn.write_packet(&packet).await;
}
pub async fn send_mc_packet(conn: &mut MCConnTcp, port: &u16, hostname: &String) {
    let protocol_version = 770;

    // Switch to login state (2)
    send_handshake(conn, protocol_version, &hostname.to_string(), *port, 2).await;

    // Send login start packet
    send_login_start(conn, "test").await;

    //_ = conn.write_packet(&Packet::empty(0x03)).await;
}
