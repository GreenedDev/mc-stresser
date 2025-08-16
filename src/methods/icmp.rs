use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

pub async fn send_icmp_ping(target: &Ipv4Addr, cps: Arc<AtomicU64>, failures: Arc<AtomicU64>) {
    // Using a RAW socket (may require privileges)
    let ip = IpAddr::V4(*target);

    match ping::new(ip).socket_type(ping::RAW).send() {
        Ok(_) => cps.fetch_add(1, Ordering::Relaxed),
        Err(_) => failures.fetch_add(1, Ordering::Relaxed),
    };
}
