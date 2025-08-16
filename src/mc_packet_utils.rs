use rust_mc_proto_tokio::{DataWriter, MCConnTcp, Packet};

// Send handshake packet to initiate connection
pub async fn send_handshake(conn: &mut MCConnTcp, proto: u16, hostname: &str, srv_port: u16, next_state: u8) {
    let mut packet = Packet::empty(0x00);

    _ = packet.write_u16_varint(proto).await;
    _ = packet.write_string(hostname).await;
    _ = packet.write_unsigned_short(srv_port).await;
    _ = packet.write_u8_varint(next_state).await;
    _ = conn.write_packet(&packet).await;
}
