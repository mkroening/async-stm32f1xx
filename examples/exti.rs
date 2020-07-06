//! This toggles an LED after an Edge has been detected on PA7.
//!
//! This assumes that a LED is connected to PC13 as is the case on the blue pill board.

#![no_main]
#![no_std]

use async_embedded::task;
use async_stm32f1xx::exti::AsyncPin;
use cortex_m_rt::entry;
use panic_semihosting as _; // panic handler
use stm32f1xx_hal::{
    gpio::{Edge, ExtiPin, State},
    pac::Peripherals,
    prelude::*,
};

#[entry]
fn main() -> ! {
    // Extract needed peripherals
    let dp = Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // Create, configure AsyncPin
    let exti_pin = gpioa.pa7.into_floating_input(&mut gpioa.crl);
    let mut exti_pin = AsyncPin::new(exti_pin, &mut afio, &dp.EXTI);
    exti_pin
        .as_mut()
        .trigger_on_edge(&dp.EXTI, Edge::RISING_FALLING);

    // Create LED
    let mut led = gpioc
        .pc13
        .into_push_pull_output_with_state(&mut gpioc.crh, State::High);

    task::block_on(async move {
        loop {
            exti_pin.trigger().await;
            led.toggle().unwrap();
        }
    })
}
