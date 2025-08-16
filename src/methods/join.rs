use rust_mc_proto_tokio::{DataWriter, MCConnTcp, Packet};
use uuid::Uuid;

use crate::mc_packet_utils::send_handshake;

pub async fn send_join(conn: &mut MCConnTcp, port: &u16, hostname: &str) {
	let protocol_version = 770;

	// Switch to login state (2)
	send_handshake(conn, protocol_version, hostname, *port, 2).await;

	// Send login start packet
	send_login_start(conn, "test").await;

	//_ = conn.write_packet(&Packet::empty(0x03)).await;
}

// Send login start packet
pub async fn send_login_start(conn: &mut MCConnTcp, username: &str) {
	let mut packet = Packet::empty(0x00);
	_ = packet.write_string(username).await;
	_ = packet.write_uuid(&Uuid::default()).await; // No UUID for offline mode
	_ = conn.write_packet(&packet).await;
}
