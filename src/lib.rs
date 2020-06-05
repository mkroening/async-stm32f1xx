//! Abstractions for asynchronous programming on the STM32F1xx family of microcontrollers.
//!
//! This crate provides [`futures`]-based abstractions for asynchronous programming with peripherals from [`stm32f1xx_hal`]:
//!
//! - [`AsyncTimer`](crate::timer::AsyncTimer) allows delaying the current task, wrapping [`Timer`](stm32f1xx_hal::timer::Timer).
//! - [`TxSink`](crate::serial::TxSink) allows [`Sink`](futures::sink::Sink)-based USART transmissions, wrapping [`TxDma`](stm32f1xx_hal::dma::TxDma).
//! - [`RxStream`](crate::serial::RxStream) allows [`Stream`](futures::stream::Stream)-based USART receives, wrapping [`RxDma`](stm32f1xx_hal::dma::RxDma).
//!
//! To properly schedule wakeups, this crate implements the following interrupts:
//!
//! - [`TIM2`](stm32f1xx_hal::pac::Interrupt::TIM2), [`TIM3`](stm32f1xx_hal::pac::Interrupt::TIM3)
//! - [`DMA1_CHANNEL4`](stm32f1xx_hal::pac::Interrupt::DMA1_CHANNEL4), [`DMA1_CHANNEL7`](stm32f1xx_hal::pac::Interrupt::DMA1_CHANNEL7), [`DMA1_CHANNEL2`](stm32f1xx_hal::pac::Interrupt::DMA1_CHANNEL2)
//! - [`DMA1_CHANNEL5`](stm32f1xx_hal::pac::Interrupt::DMA1_CHANNEL5), [`DMA1_CHANNEL6`](stm32f1xx_hal::pac::Interrupt::DMA1_CHANNEL6), [`DMA1_CHANNEL3`](stm32f1xx_hal::pac::Interrupt::DMA1_CHANNEL3)

#![no_std]
#![deny(clippy::all, rust_2018_idioms)]
#![warn(missing_docs)]

/// Creates a new interrupt waking a [`Waker`].
///
/// As this interrupt will be declared in this macro, it can't be used for anything else.
///
/// # Examples
///
/// This macro is useful for implementing [`Future::poll`]:
///
/// ```
/// fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
///     if self.is_ready() {
///         Poll::Ready(())
///     } else {
///         waker_interrupt!(TIM2, cx.waker().clone());
///         Poll::Pending
///     }
/// }
/// ```
///
/// [`Waker`]: core::task::Waker
/// [`Future::poll`]: core::future::Future::poll
macro_rules! waker_interrupt {
    ($INT:ident, $waker:expr) => {{
        use core::sync::atomic::{self, Ordering};
        use stm32f1xx_hal::pac::{interrupt, Interrupt, NVIC};

        static mut WAKER: Option<Waker> = None;

        #[interrupt]
        fn $INT() {
            // Safety: This context is disabled while the lower priority context accesses WAKER
            if let Some(waker) = unsafe { WAKER.as_ref() } {
                waker.wake_by_ref();

                NVIC::mask(Interrupt::$INT);
            }
        }

        NVIC::mask(Interrupt::$INT);
        atomic::compiler_fence(Ordering::Acquire);
        // Safety: The other relevant context, the interrupt, is disabled
        unsafe { WAKER = Some($waker) }
        NVIC::unpend(Interrupt::$INT);
        atomic::compiler_fence(Ordering::Release);
        // Safety: This is the end of a mask-based critical section
        unsafe { NVIC::unmask(Interrupt::$INT) }
    }};
}

pub mod serial;
pub mod timer;
