use std::net::SocketAddrV4;
use std::{net::Ipv4Addr, sync::atomic::AtomicU64, time::Duration};
use std::{sync::atomic::Ordering, sync::Arc};

use clap::Parser;
use tokio::time::sleep;

use rust_mc_proto_tokio::{DataWriter, MCConnTcp, Packet};
use uuid::Uuid;

use crate::resolver::resolve_mc;
mod resolver;
#[derive(Debug, Parser)]
struct Args {
    addr_port: String,
    tasks: u32,
}
fn parse_target(target: String, default_port: u16) -> (String, u16) {
    let mut split = target.split(":");
    let addr = split.next().expect("no ip provided").to_string();
    let port = match split.next() {
        Some(value) => value.parse::<u16>().expect("can't parse port as u16"),
        None => default_port,
    };
    return (addr, port);
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let default_port = 25565_u16;
    let target = args.addr_port;
    let (addr, port) = parse_target(target, default_port);
    let socket_addr = match addr.parse::<Ipv4Addr>() {
        Ok(value) => SocketAddrV4::new(value, port),
        Err(_) => resolve_mc(addr, port, default_port).await,
    };

    println!("sssssss {socket_addr:?}");
    /*
    let connections = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));

    tokio::spawn(write_stats(connections.clone(), failures.clone()));

    let tasks = args.tasks as usize;

    for _ in 0..tasks {
        let connections = connections.clone();
        let failures = failures.clone();
        tokio::spawn(async move {
            worker_loop(connections, failures, target).await;
        });
    }
    */
    println!("0000");
    tokio::signal::ctrl_c()
        .await
        .expect("couldn't wait for ctrl c");
    println!("1111");
}

async fn worker_loop(
    connections: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    target: (Ipv4Addr, u16),
) {
    loop {
        let stream = match tokio::net::TcpStream::connect(target).await {
            Ok(value) => value,
            Err(_) => {
                failures.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        let mut conn = MCConnTcp::new(stream);
        send_mc_packet(&mut conn, &target.0.to_string(), target.1).await;
        connections.fetch_add(1, Ordering::Relaxed);
    }
}

async fn send_mc_packet(conn: &mut MCConnTcp, ip: &str, port: u16) {
    let protocol_version = 770;

    // Switch to login state (2)
    send_handshake(conn, protocol_version, ip, port, 2).await;

    // Send login start packet
    send_login_start(conn, "test").await;

    conn.write_packet(&Packet::empty(0x03)).await.ok();
}

async fn write_stats(cps: Arc<AtomicU64>, fails: Arc<AtomicU64>) {
    loop {
        sleep(Duration::from_secs(1)).await;
        println!(
            "cps: {cps} fails: {fails}",
            cps = cps.swap(0, Ordering::Relaxed),
            fails = fails.swap(0, Ordering::Relaxed),
        );
    }
} // Send handshake packet to initiate connection
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
