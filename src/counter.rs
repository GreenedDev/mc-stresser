use std::{
    io::Write,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use ez_colorize::ColorizeDisplay;
use tokio::time::sleep;
pub const ANALYTICS: &str = "\x1b[33m[ANALYTICS]\x1b[0m";
pub async fn write_stats(cps: Arc<AtomicU64>, fails: Arc<AtomicU64>) {
    let mut max_cps = 0_u64;
    let mut average_cps = 0_u64;
    let mut average_divisor = 0_u64;

    let mut is_started = false;
    loop {
        if !is_started && cps.load(Ordering::Relaxed) == 0 {
            sleep(Duration::from_millis(1)).await;
            continue;
        }
        is_started = true;

        sleep(Duration::from_secs(1)).await;

        average_divisor += 1;

        let prev_cps = cps.swap(0, Ordering::Relaxed);
        average_cps = ((average_divisor - 1) * average_cps + prev_cps) / average_divisor;
        if prev_cps > max_cps {
            max_cps = prev_cps;
        }
        print!("\x1b[4A");
        print!("\r");
        println!("{ANALYTICS} Traffic statistics:\x1B[K");
        println!(
            "{ANALYTICS} MAX CPS: {max_cps}{} AVERAGE CPS: {average_cps}{}      \x1B[K",
            "/s".to_string().yellow(),
            "/s".to_string().yellow()
        );
        println!(
            "{ANALYTICS} CPS: {prev_cps:?}{}      \x1B[K",
            "/s".to_string().yellow()
        );
        println!(
            "{ANALYTICS} FAILS: {fails:?}{}      \x1B[K",
            "/s".to_string().yellow()
        );
        std::io::stdout().flush().unwrap();
    }
}
