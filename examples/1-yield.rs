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
use cortex_m_semihosting::hprintln;
use panic_semihosting as _; // panic handler
use stm32f1xx_hal as _; // memory layout

#[entry]
fn main() -> ! {
    let a = async {
        loop {
            hprintln!("Task `a` will yield now.").ok();
            task::r#yield().await;
        }
    };
    task::spawn(a);

    let b = async {
        hprintln!("Task `b` will yield now.").ok();
        task::r#yield().await;

        hprintln!("Task `b` will yield again now.").ok();
        task::r#yield().await;

        hprintln!("Task `b` will not yield again.").ok();
        loop {}
    };
    task::block_on(b)
}
