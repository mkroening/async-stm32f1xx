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
use cortex_m_semihosting::hprintln;
use panic_semihosting as _; // panic handler
use stm32f1xx_hal as _; // memory layout

#[entry]
fn main() -> ! {
    static mut CHANNEL: Channel<u32> = Channel::new();

    // coerce to a shared (`&-`) reference to avoid _one_ of the `move` blocks taking ownership of
    // the owning static (`&'static mut`) reference
    let channel: &'static _ = CHANNEL;

    let a = async move {
        hprintln!("Task `a` will send a message now.").ok();
        channel.send(42).await;
        hprintln!("Task `a` has sent the message.").ok();

        loop {
            hprintln!("Task `a` will yield now.").ok();
            task::r#yield().await;
        }
    };
    task::spawn(a);

    let b = async move {
        hprintln!("Task `b` will start asynchronously receiving a message now.").ok();
        // If no message can be received immediately, this yields.
        let msg = channel.recv().await;
        hprintln!("Task `b`: msg = {:?}", msg).ok();

        hprintln!("Task `b` will not yield again.").ok();
        loop {}
    };
    task::block_on(b)
}
