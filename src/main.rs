use std::sync::atomic::AtomicUsize;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, net::SocketAddrV4, sync::atomic::Ordering, sync::Arc};
use std::{net::Ipv4Addr, sync::atomic::AtomicU64, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Context, OptionExt};
use dashmap::DashMap;
use packet_utils::{send_handshake, send_login_start};
use rust_mc_proto_tokio::{MCConnTcp, Packet};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
use tokio_socks::tcp::Socks4Stream;
mod packet_utils;

#[derive(Debug, Parser)]
struct Args {
    server_address: Ipv4Addr,
    server_port: u16,
    tasks: u32,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

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
    tokio::signal::ctrl_c().await?;
    println!("1111");
    return Ok(());
}

async fn worker_loop(
    connections: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    target: (Ipv4Addr, u16),
) {
    loop {
        /*// get next index atomically (wrap-around)
        let idx = proxy_index.fetch_add(1, Ordering::Relaxed) % proxies.len();
        let proxy = proxies[idx];

        // quick disabled check (no await)
        if let Some(ts) = disabled_proxies.get(&proxy) {
            if *ts >= get_current_time_millis() {
                // proxy still disabled — try next immediately
                continue;
            } else {
                disabled_proxies.remove(&proxy);
            }
        }*/
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
        /*
                // Only the connect attempt is timed out:
                match tokio::time::timeout(Duration::from_secs(2), Socks4Stream::connect(proxy, target))
                    .await
                {
                    Ok(Ok(socks_stream)) => {
                        connections.fetch_add(1, Ordering::Relaxed);
                        // create a real String for IP and pass by ref to avoid borrow of temp
                        let ip_str = target.0.to_string();
                        let port = target.1;
                        send_mc_packet(socks_stream.into_inner(), &ip_str, port).await;
                    }
                    Ok(Err(_)) | Err(_) => {
                        // Err from connect OR timeout expired
                        disabled_proxies.insert(proxy, get_current_time_millis() + 2000);
                        failures.fetch_add(1, Ordering::Relaxed);
                        // small backoff to avoid hot loop if all proxies failing:
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
        */
    }
}

fn get_current_time_millis() -> u128 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
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

fn load_proxies() -> color_eyre::Result<Vec<SocketAddrV4>> {
    fs::read_to_string("proxies.txt")
        .wrap_err("couldn't find proxies.txt")?
        .lines()
        .map(|line| {
            let mut parts = line.split(":");
            let addr = parts.next().ok_or_eyre("missing server address")?;
            let port = parts.next().ok_or_eyre("missing port")?;

            let addr = addr
                .parse::<Ipv4Addr>()
                .wrap_err("couldn't parse target as ipv4 addr")?;
            let port = port
                .parse::<u16>()
                .wrap_err("couldn't parse port as an u16")?;
            Ok(SocketAddrV4::new(addr, port))
        })
        .collect()
}
