//! Yielding from a task
//!
//! # Expected output
//!
//! ```
//! Task `b` will yield now.
//! Task `a` will yield now.
//! Task `b` will yield again now.
//! Task `a` will yield now.
//! Task `b` will not yield again.
//! ```

#![no_main]
#![no_std]

use async_embedded::task;
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler
use stm32f1xx_hal as _; // memory layout

#[entry]
fn main() -> ! {
    let a = async {
        loop {
            info!("Task `a` will yield now.");
            task::r#yield().await;
        }
    };
    task::spawn(a);

    let b = async {
        info!("Task `b` will yield now.");
        task::r#yield().await;

        info!("Task `b` will yield again now.");
        task::r#yield().await;

        info!("Task `b` will not yield again.");
        loop {}
    };
    task::block_on(b)
}
