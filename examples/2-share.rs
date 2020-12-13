//! Sharing state between tasks using [`Cell`] and [`RefCell`].
//!
//! # Expected output
//!
//! ```
//! Task `b` will set cell now.
//! Task `b` will set ref_cell now.
//! Task `b` will yield now.
//! Task `a`: cell = Cell { value: 42 }
//! Task `a`: ref_cell = RefCell { value: Some(42) }
//! Task `a` will yield now.
//! Task `b` will not yield again.
//! ```

#![no_main]
#![no_std]

use async_embedded::task;
use core::cell::{Cell, RefCell};
use cortex_m_rt::entry;
use defmt::{consts, info, Debug2Format};
use defmt_rtt as _; // global logger
use panic_probe as _;
use stm32f1xx_hal as _; // memory layout // panic handler

#[entry]
fn main() -> ! {
    static mut CELL: Cell<u32> = Cell::new(0);
    static mut REF_CELL: RefCell<Option<u32>> = RefCell::new(None);

    // only references with `'static` lifetimes can be sent to `spawn`-ed tasks
    // NOTE we coerce these to a shared (`&-`) reference to avoid the `move` blocks taking ownership
    // of the owning static (`&'static mut`) reference
    let cell: &'static Cell<_> = CELL;
    let ref_cell: &'static RefCell<_> = REF_CELL;

    let a = async move {
        info!("Task `a`: cell = {:?}", Debug2Format::<consts::U64>(cell));
        info!(
            "Task `a`: ref_cell = {:?}",
            Debug2Format::<consts::U64>(ref_cell)
        );

        loop {
            info!("Task `a` will yield now.");
            task::r#yield().await;
        }
    };
    task::spawn(a);

    let b = async {
        info!("Task `b` will set cell now.");
        cell.set(42);

        info!("Task `b` will set ref_cell now.");
        *ref_cell.borrow_mut() = Some(42);

        info!("Task `b` will yield now.");
        task::r#yield().await;

        info!("Task `b` will not yield again.");
        loop {}
    };
    task::block_on(b)
}
