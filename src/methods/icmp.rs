use std::{
    net::Ipv4Addr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use fastping_rs::Pinger;

pub async fn send_icmp_ping(target: &Ipv4Addr, cps: Arc<AtomicU64>, failures: Arc<AtomicU64>) {
    let (pinger, _) = match Pinger::new(None, Some(56)) {
        Ok((pinger, results)) => (pinger, results),
        Err(_) => {
            failures.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };

    pinger.add_ipaddr(target.to_string().as_str());
    pinger.run_pinger();
    cps.fetch_add(1, Ordering::Relaxed);
}
