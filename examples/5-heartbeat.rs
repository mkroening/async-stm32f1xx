//! Blinking an LED in a heartbeat rhythm
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.

#![no_main]
#![no_std]

use async_embedded::task;
use async_stm32f1xx::timer::AsyncTimer;
use cortex_m_rt::entry;
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler
use stm32f1xx_hal::{gpio::State, pac::Peripherals, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    // Extract needed peripherals
    let dp = Peripherals::take().expect("Peripherals have been taken before");

    // Avoid AHB going into low-power mode causing RTT to stop working
    dp.RCC.ahbenr.modify(|_, w| w.dma1en().enabled());

    let rcc = dp.RCC.constrain();

    // Create Timer
    let mut apb1 = rcc.apb1;
    let mut acr = dp.FLASH.constrain().acr;
    let clocks = rcc.cfgr.freeze(&mut acr);
    let mut timer: AsyncTimer<_> = Timer::tim2(dp.TIM2, &clocks, &mut apb1).into();

    // Create Led
    let mut apb2 = rcc.apb2;
    let gpioc = dp.GPIOC.split(&mut apb2);
    let mut cr = gpioc.crh;
    let mut led = gpioc
        .pc13
        .into_push_pull_output_with_state(&mut cr, State::High);

    task::block_on(async {
        loop {
            led.toggle().unwrap();
            timer.delay_for(10.hz()).await;
            led.toggle().unwrap();
            timer.delay_for(4.hz()).await;
            led.toggle().unwrap();
            timer.delay_for(10.hz()).await;
            led.toggle().unwrap();
            timer.delay_for(2.hz()).await;
        }
    })
}
