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
static mut MAX_CPS: u64 = 0;
static mut AVERAGE_CPS: u64 = 0;
static mut AVERAGE_DIVISOR: u64 = 0;
#[expect(static_mut_refs)]
pub async unsafe fn write_stats(cps: Arc<AtomicU64>, fails: Arc<AtomicU64>) {
    unsafe {
        loop {
            sleep(Duration::from_secs(1)).await;

            AVERAGE_DIVISOR += 1;

            let prev_cps = cps.swap(0, Ordering::Relaxed);
            AVERAGE_CPS = ((AVERAGE_DIVISOR - 1) * AVERAGE_CPS + prev_cps) / AVERAGE_DIVISOR;
            if prev_cps > MAX_CPS {
                MAX_CPS = prev_cps;
            }
            print!("\x1b[4A");
            print!("\r");
            println!("{ANALYTICS} Traffic statistics:\x1B[K");
            println!(
                "{ANALYTICS} MAX CPS: {MAX_CPS:?}{} AVERAGE CPS: {AVERAGE_CPS:?}{}      \x1B[K",
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
            //print!("{ANALYTICS} Bits: {}\x1B[K", "0 mbit/s".to_string().cyan());
            std::io::stdout().flush().unwrap();
        }
    }
}
