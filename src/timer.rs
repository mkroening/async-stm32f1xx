//! [`Future`]-based abstractions for timers.

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use embedded_hal::timer::CountDown;
use stm32f1xx_hal::{
    pac::{TIM2, TIM3},
    time::U32Ext,
    timer::{CountDownTimer, Event, Timer},
};

/// An asynchronous abstraction over a timer.
///
/// # Examples
///
/// ```
/// let mut timer: AsyncTimer<_> = Timer::tim2(dp.TIM2, &clocks, &mut apb1).into();
/// loop {
///     led.toggle();
///     timer.delay_for(2.hz()).await;
/// }
/// ```
pub struct AsyncTimer<T>(T);

impl<T> AsRef<T> for AsyncTimer<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for AsyncTimer<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> AsyncTimer<T>
where
    T: CountDown,
{
    /// Creates a [`Future`] that resolves after the given time has been count down.
    pub fn delay_for<C>(&mut self, count: C) -> Delay<'_, T>
    where
        C: Into<T::Time>,
    {
        self.as_mut().start(count);
        Delay(&mut self.0)
    }
}

/// [`Future`] returned by [`delay_for`].
///
/// [`delay_for`]: AsyncTimer::delay_for
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Delay<'a, T>(&'a mut T);

impl<T> AsRef<T> for Delay<'_, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Delay<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        self.0
    }
}

macro_rules! timer {
    ($(
        $TIMX:ident
    ),+) => {
        $(
            impl AsyncTimer<CountDownTimer<$TIMX>> {
                /// Releases the TIM peripheral
                pub fn release(self) -> $TIMX {
                    self.0.release()
                }
            }

            impl From<Timer<$TIMX>> for AsyncTimer<CountDownTimer<$TIMX>> {
                fn from(timer: Timer<$TIMX>) -> Self {
                    let mut count_down_timer = timer.start_count_down(1.hz());
                    count_down_timer.listen(Event::Update);
                    Self(count_down_timer)
                }
            }

            impl Future for Delay<'_, CountDownTimer<$TIMX>> {
                type Output = ();

                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    use nb::{Error, Result};

                    match self.get_mut().as_mut().wait() {
                        Result::Ok(ok) => Poll::Ready(ok),
                        Result::Err(Error::Other(err)) => void::unreachable(err),
                        Result::Err(Error::WouldBlock) => {
                            waker_interrupt!($TIMX, cx.waker().clone());
                            Poll::Pending
                        }
                    }
                }
            }
        )+
    }
}

timer!(TIM2, TIM3);
