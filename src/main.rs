use std::net::SocketAddrV4;
use std::process::exit;
use std::time::Instant;
use std::{sync::atomic::AtomicU64, time::Duration};
use std::{sync::atomic::Ordering, sync::Arc};

use clap::Parser;
use tokio::time::sleep;

use rust_mc_proto_tokio::MCConnTcp;

use crate::mc_packet_utils::send_mc_packet;
use crate::resolver::{parse_hostname, parse_target};
mod mc_packet_utils;
mod resolver;
#[derive(Debug, Parser)]
struct Args {
    target: String,
    workers: u32,
    duration_secs: u64,
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let start_time = Instant::now();
    let target = parse_target(args.target.as_str(), 25565).await.unwrap();
    let hostname = Arc::new(parse_hostname(args.target.as_str()));
    println!("resolved target {target:?}");
    let connections = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));

    tokio::spawn(write_stats(connections.clone(), failures.clone()));

    let workers = args.workers;

    for _ in 0..workers {
        let hostname = hostname.clone();
        let connections = connections.clone();
        let failures = failures.clone();
        tokio::spawn(async move {
            worker_loop(connections, failures, target, hostname.as_str()).await;
        });
    }
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            if start_time.elapsed().as_secs() > args.duration_secs {
                exit(0);
            }
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("couldn't wait for ctrl c");
}

async fn worker_loop(
    connections: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    target: SocketAddrV4,
    hostname: &str,
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
        send_mc_packet(&mut conn, &target.port(), hostname).await;
        connections.fetch_add(1, Ordering::Relaxed);
    }
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
