use std::{fs, net::SocketAddrV4, sync::atomic::Ordering, sync::Arc};
use std::{net::Ipv4Addr, sync::atomic::AtomicU64, time::Duration};

use clap::Parser;
use packet_utils::{send_handshake, send_login_start};
use rust_mc_proto_tokio::{MCConnTcp, Packet};
use tokio::time::sleep;
mod packet_utils;

#[derive(Debug, Parser)]
struct Args {
    server_address: Ipv4Addr,
    server_port: u16,
    tasks: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let target = (args.server_address, args.server_port);
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
}

fn load_proxies() -> Vec<SocketAddrV4> {
    fs::read_to_string("proxies.txt")
        .unwrap()
        .lines()
        .map(|line| {
            let mut parts = line.split(":");
            let addr = parts.next().expect("missing server address");
            let port = parts.next().expect("missing port");

            let addr = addr
                .parse::<Ipv4Addr>()
                .expect("couldn't parse target as ipv4 addr");
            let port = port.parse::<u16>().expect("couldn't parse port as an u16");
            SocketAddrV4::new(addr, port)
        })
        .collect()
}
