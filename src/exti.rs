use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use stm32f1xx_hal::{
    afio,
    gpio::{gpioa, gpiob, gpioc, gpiod, gpioe, ExtiPin, Input},
    pac::{interrupt, Interrupt, EXTI, NVIC},
};

const WAKER_NONE: Option<Waker> = None;

macro_rules! multi_interrupt {
    ($INT:ident, $waker_count:expr) => {
        static mut $INT: [Option<Waker>; $waker_count] = [WAKER_NONE; $waker_count];

        #[interrupt]
        fn $INT() {
            // Safety: This context is disabled while the lower priority context accesses WAKER
            unsafe {
                $INT.iter()
                    .filter_map(Option::as_ref)
                    .for_each(Waker::wake_by_ref);
            }

            NVIC::mask(Interrupt::$INT);
        }
    };
}

multi_interrupt!(EXTI9_5, 5);
multi_interrupt!(EXTI15_10, 6);

macro_rules! install_multi_interrupt_waker {
    ($INT:expr, $WAKER:expr, $waker:expr) => {{
        use core::sync::atomic::{self, Ordering};
        use stm32f1xx_hal::pac::{Interrupt, NVIC};

        NVIC::mask($INT);
        atomic::compiler_fence(Ordering::Acquire);
        // Safety: The other relevant context, the interrupt, is disabled
        unsafe { $WAKER = Some($waker) }
        NVIC::unpend($INT);
        atomic::compiler_fence(Ordering::Release);
        // Safety: This is the end of a mask-based critical section
        unsafe { NVIC::unmask($INT) }
    }};
}

pub struct AsyncPin<P>(P);

impl<P> AsRef<P> for AsyncPin<P> {
    fn as_ref(&self) -> &P {
        &self.0
    }
}

impl<P> AsMut<P> for AsyncPin<P> {
    fn as_mut(&mut self) -> &mut P {
        &mut self.0
    }
}

impl<P: ExtiPin> AsyncPin<P> {
    pub fn new(mut pin: P, afio: &mut afio::Parts, exti: &EXTI) -> Self {
        pin.make_interrupt_source(afio);
        pin.enable_interrupt(exti);
        Self(pin)
    }

    pub fn trigger(&mut self) -> AsyncTrigger<'_, P> {
        AsyncTrigger(&mut self.0)
    }
}

pub struct AsyncTrigger<'a, P>(&'a mut P);

impl<P> AsRef<P> for AsyncTrigger<'_, P> {
    fn as_ref(&self) -> &P {
        &self.0
    }
}

impl<P> AsMut<P> for AsyncTrigger<'_, P> {
    fn as_mut(&mut self) -> &mut P {
        self.0
    }
}

macro_rules! implement_trigger_future {
    ($(
        $INT:expr => {$(
            $WAKER:expr => {$(
                $PXx:ty,
            )+},
        )+},
    )+) => {
        $($($(
            impl<MODE> Future for AsyncTrigger<'_, $PXx>
            where
                MODE: Unpin,
            {
                type Output = ();

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    if !self.0.check_interrupt() {
                        install_multi_interrupt_waker!($INT, $WAKER, cx.waker().clone());
                        Poll::Pending
                    } else {
                        self.0.clear_interrupt_pending_bit();
                        Poll::Ready(())
                    }
                }
            }
        )+)+)+
    };
}

implement_trigger_future!(
    Interrupt::EXTI9_5 => {
        EXTI9_5[0] => {
            gpioa::PA5<Input<MODE>>,
            gpiob::PB5<Input<MODE>>,
            gpioc::PC5<Input<MODE>>,
            gpiod::PD5<Input<MODE>>,
            gpioe::PE5<Input<MODE>>,
        },
        EXTI9_5[1] => {
            gpioa::PA6<Input<MODE>>,
            gpiob::PB6<Input<MODE>>,
            gpioc::PC6<Input<MODE>>,
            gpiod::PD6<Input<MODE>>,
            gpioe::PE6<Input<MODE>>,
        },
        EXTI9_5[2] => {
            gpioa::PA7<Input<MODE>>,
            gpiob::PB7<Input<MODE>>,
            gpioc::PC7<Input<MODE>>,
            gpiod::PD7<Input<MODE>>,
            gpioe::PE7<Input<MODE>>,
        },
        EXTI9_5[3] => {
            gpioa::PA8<Input<MODE>>,
            gpiob::PB8<Input<MODE>>,
            gpioc::PC8<Input<MODE>>,
            gpiod::PD8<Input<MODE>>,
            gpioe::PE8<Input<MODE>>,
        },
        EXTI9_5[4] => {
            gpioa::PA9<Input<MODE>>,
            gpiob::PB9<Input<MODE>>,
            gpioc::PC9<Input<MODE>>,
            gpiod::PD9<Input<MODE>>,
            gpioe::PE9<Input<MODE>>,
        },
    },
    Interrupt::EXTI15_10 => {
        EXTI15_10[0] => {
            gpioa::PA10<Input<MODE>>,
            gpiob::PB10<Input<MODE>>,
            gpioc::PC10<Input<MODE>>,
            gpiod::PD10<Input<MODE>>,
            gpioe::PE10<Input<MODE>>,
        },
        EXTI15_10[1] => {
            gpioa::PA11<Input<MODE>>,
            gpiob::PB11<Input<MODE>>,
            gpioc::PC11<Input<MODE>>,
            gpiod::PD11<Input<MODE>>,
            gpioe::PE11<Input<MODE>>,
        },
        EXTI15_10[2] => {
            gpioa::PA12<Input<MODE>>,
            gpiob::PB12<Input<MODE>>,
            gpioc::PC12<Input<MODE>>,
            gpiod::PD12<Input<MODE>>,
            gpioe::PE12<Input<MODE>>,
        },
        EXTI15_10[3] => {
            gpioa::PA13<Input<MODE>>,
            gpiob::PB13<Input<MODE>>,
            gpioc::PC13<Input<MODE>>,
            gpiod::PD13<Input<MODE>>,
            gpioe::PE13<Input<MODE>>,
        },
        EXTI15_10[4] => {
            gpioa::PA14<Input<MODE>>,
            gpiob::PB14<Input<MODE>>,
            gpioc::PC14<Input<MODE>>,
            gpiod::PD14<Input<MODE>>,
            gpioe::PE14<Input<MODE>>,
        },
        EXTI15_10[5] => {
            gpioa::PA15<Input<MODE>>,
            gpiob::PB15<Input<MODE>>,
            gpioc::PC15<Input<MODE>>,
            gpiod::PD15<Input<MODE>>,
            gpioe::PE15<Input<MODE>>,
        },
    },
);
