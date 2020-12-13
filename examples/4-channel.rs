//! Message passing between tasks using a MPMC channel
//!
//! # Expected output
//!
//! ```
//! Task `b` will start asynchronously receiving a message now.
//! Task `a` will send a message now.
//! Task `a` has sent the message.
//! Task `a` will yield now.
//! Task `b`: msg = 42
//! Task `b` will not yield again.
//! ```

#![no_main]
#![no_std]

use async_embedded::{task, unsync::Channel};
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler
use stm32f1xx_hal as _; // memory layout

#[entry]
fn main() -> ! {
    static mut CHANNEL: Channel<u32> = Channel::new();

    // coerce to a shared (`&-`) reference to avoid _one_ of the `move` blocks taking ownership of
    // the owning static (`&'static mut`) reference
    let channel: &'static _ = CHANNEL;

    let a = async move {
        info!("Task `a` will send a message now.");
        channel.send(42).await;
        info!("Task `a` has sent the message.");

        loop {
            info!("Task `a` will yield now.");
            task::r#yield().await;
        }
    };
    task::spawn(a);

    let b = async move {
        info!("Task `b` will start asynchronously receiving a message now.");
        // If no message can be received immediately, this yields.
        let msg = channel.recv().await;
        info!("Task `b`: msg = {:?}", msg);

        info!("Task `b` will not yield again.");
        loop {}
    };
    task::block_on(b)
}
