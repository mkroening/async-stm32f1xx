//! Mutex shared between tasks
//!
//! "When to use `Mutex` instead of a `RefCell`?" Both abstractions will give you an exclusive
//! (`&mut-`) reference to the data and that reference can survive across `yield`s (either explicit
//! , i.e. `task::yield`, or implicit, `.await`).
//!
//! The difference between the two is clear when contention occurs. If two or more tasks contend for
//! a `RefCell`, as in they both call `borrow_mut` on it, you'll get a panic. On the other hand, if
//! you use a `Mutex` in a similar scenario, i.e. both tasks call `lock` on it, then one of them
//! will asynchronously wait for (i.e. not resume until) the other task to release (releases) the
//! lock.
//!
//! # Expected output
//!
//! ```
//! Task `b` will asynchronously lock the mutex now.
//! Task `a` will write now.
//! Task `a` has dropped the lock.
//! Task `a` will yield now.
//! Task `b`: *lock = 42
//! Task `b` will not yield again.
//! ```

#![no_main]
#![no_std]

use async_embedded::{task, unsync::Mutex};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _; // panic handler
use stm32f1xx_hal as _; // memory layout

#[entry]
fn main() -> ! {
    static mut X: Mutex<u32> = Mutex::new(0);
    // Locking the Mutex immediately forces contention.
    let mut lock = X.try_lock().unwrap();

    let a = async {
        hprintln!("Task `a` will write now.").ok();
        *lock = 42;

        drop(lock);
        hprintln!("Task `a` has dropped the lock.").ok();

        loop {
            hprintln!("Task `a` will yield now.").ok();
            task::r#yield().await;
        }
    };
    task::spawn(a);

    let b = async {
        hprintln!("Task `b` will asynchronously lock the mutex now.").ok();
        // If the mutex can not be locked immediately, this yields.
        let lock = X.lock().await;
        hprintln!("Task `b`: *lock = {:?}", *lock).ok();

        hprintln!("Task `b` will not yield again.").ok();
        loop {}
    };
    task::block_on(b)
}
