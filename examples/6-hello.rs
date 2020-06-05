//! Spam "01234567" over the serial line (@ 9_600 bauds) while running the heartbeat task from `5-heartbeat`.
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.

#![no_main]
#![no_std]

use async_embedded::task;
use async_stm32f1xx::{serial::TxSink3, timer::AsyncTimer};
use cortex_m_rt::entry;
use futures::sink::SinkExt;
use panic_semihosting as _; // panic handler
use stm32f1xx_hal::{
    gpio::State,
    pac::Peripherals,
    prelude::*,
    serial::{Config, Serial},
    timer::Timer,
};

#[entry]
fn main() -> ! {
    // Extract needed peripherals
    let dp = Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let mut apb1 = rcc.apb1;
    let mut acr = dp.FLASH.constrain().acr;
    let clocks = rcc.cfgr.freeze(&mut acr);

    // Create Timer
    let mut timer: AsyncTimer<_> = Timer::tim2(dp.TIM2, &clocks, &mut apb1).into();

    // Create Led
    let mut apb2 = rcc.apb2;
    let gpioc = dp.GPIOC.split(&mut apb2);
    let mut cr = gpioc.crh;
    let mut led = gpioc
        .pc13
        .into_push_pull_output_with_state(&mut cr, State::High);

    // heartbeat task
    task::spawn(async move {
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
    });

    // Create TxSink
    let mut gpiob = dp.GPIOB.split(&mut apb2);
    let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let rx = gpiob.pb11;
    let afio = dp.AFIO.constrain(&mut apb2);
    let mut mapr = afio.mapr;
    let serial = Serial::usart3(
        dp.USART3,
        (tx, rx),
        &mut mapr,
        Config::default().baudrate(9_600.bps()),
        clocks,
        &mut apb1,
    );
    let mut ahb = rcc.ahb;
    let channels = dp.DMA1.split(&mut ahb);
    let (tx, _rx) = serial.split();
    let tx_buf = {
        static mut BUF: [u8; 8] = [0; 8];
        // Safety: We only create one mutable reference
        unsafe { &mut BUF }
    };
    let mut tx_sink = TxSink3::new(tx_buf, tx.with_dma(channels.2));

    task::block_on(async {
        loop {
            tx_sink.send(*b"01234567").await.unwrap();
        }
    })
}
