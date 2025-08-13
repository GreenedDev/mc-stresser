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
    let mut packet = Packet::empty(0x00); //packet id for handshake

    packet.write_i32_varint(proto as i32).await.ok();
    packet.write_string(srv_addr).await.ok();
    packet.write_unsigned_short(srv_port).await.ok();
    packet.write_i32_varint(next_state as i32).await.ok();
    conn.write_packet(&packet).await.ok();
}

// Send login start packet
pub async fn send_login_start(conn: &mut MCConnTcp, username: &str) {
    let mut packet = Packet::empty(0x00);
    packet.write_string(username).await.ok();
    packet.write_uuid(&Uuid::default()).await.ok(); // No UUID for offline mode
    conn.write_packet(&packet).await.ok();
}

pub async fn send_mc_packet(conn: &mut MCConnTcp, port: &u16, hostname: &String) {
    let protocol_version = 770;

    // Switch to login state (2)
    send_handshake(conn, protocol_version, &hostname.to_string(), *port, 2).await;

    // Send login start packet
    send_login_start(conn, "test").await;

    conn.write_packet(&Packet::empty(0x03)).await.ok();
}
