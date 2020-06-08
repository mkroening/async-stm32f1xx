//! [`Stream`]/[`Sink`]-based abstractions for DMA-based Serial Communication (USART).

use as_slice::{AsMutSlice, AsSlice};
use core::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use futures::{
    sink::{Sink, SinkExt},
    stream::{FusedStream, Stream},
};
use stm32f1xx_hal::{
    dma::{self, CircBuffer, CircReadDma, Event, Half, Transfer, WriteDma, R},
    serial::{RxDma1, RxDma2, RxDma3, TxDma1, TxDma2, TxDma3},
};

/// A [`Future`] driving a [`Transfer`].
///
/// You can not use this directly.
/// Use [`TxSink`] instead.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct TransferFuture<T>(Option<T>);

impl<T> TransferFuture<T> {
    /// Creates a TransferFuture, the DMA channel of which must be listen to [`Event::TransferComplete`].
    fn from_listening(transfer: T) -> Self {
        Self(Some(transfer))
    }
}

macro_rules! transfer_future {
    ($(
        $USARTX:ident: ($INT:ident, $TxDmaX:ty),
    )+) => {
        $(
            impl<BUF> Future for TransferFuture<Transfer<R, BUF, $TxDmaX>>
            where
                BUF: Unpin,
            {
                type Output = (BUF, $TxDmaX);

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let transfer = self.0.as_mut().expect("polled after completion");
                    if transfer.is_done() {
                        Poll::Ready(self.0.take().unwrap().wait())
                    } else {
                        waker_interrupt!($INT, cx.waker().clone());
                        Poll::Pending
                    }
                }
            }
        )+
    }
}

transfer_future!(
    USART1: (DMA1_CHANNEL4, TxDma1),
    USART2: (DMA1_CHANNEL7, TxDma2),
    USART3: (DMA1_CHANNEL2, TxDma3),
);

/// A [`Sink`]-based asynchronous abstraction over a DMA transmitter.
///
/// # Examples
///
/// ```
/// let mut tx_sink = TxSink3::new(tx_buf, tx.with_dma(channels.2));
/// // Spams "01234567"
/// loop {
///     tx_sink.send(*b"01234567").await.unwrap();
/// }
/// ```
#[must_use = "sinks do nothing unless polled"]
pub struct TxSink<'a, BUF, PAYLOAD>(Option<TxSinkState<'a, BUF, PAYLOAD>>);

enum TxSinkState<'a, BUF, PAYLOAD> {
    Ready {
        buf: &'a mut BUF,
        tx: PAYLOAD,
    },
    Sending {
        transfer: TransferFuture<Transfer<R, &'a mut BUF, PAYLOAD>>,
    },
}

impl<'a, BUF, PAYLOAD> TxSink<'a, BUF, PAYLOAD>
where
    TxSink<'a, BUF, PAYLOAD>: Sink<BUF, Error = Infallible>,
    PAYLOAD: Unpin,
{
    /// Releases the buffer and payload peripheral.
    pub async fn release(mut self) -> (&'a mut BUF, PAYLOAD) {
        // Unwrapping: TxSink is infallible
        self.close().await.unwrap();
        match self.0.unwrap() {
            TxSinkState::Ready { buf, tx } => (buf, tx),
            _ => unreachable!("invalid state after closing"),
        }
    }
}

impl<BUF, PAYLOAD> Sink<BUF> for TxSink<'static, BUF, PAYLOAD>
where
    BUF: AsSlice<Element = u8>,
    PAYLOAD: WriteDma<BUF, &'static mut BUF, u8> + Unpin,
    TransferFuture<Transfer<R, &'static mut BUF, PAYLOAD>>:
        Future<Output = (&'static mut BUF, PAYLOAD)>,
{
    type Error = Infallible;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: BUF) -> Result<(), Self::Error> {
        let this = self.0.take().unwrap();
        match this {
            TxSinkState::Ready { tx, buf } => {
                *buf = item;
                let transfer = TransferFuture::from_listening(tx.write(buf));
                self.0 = Some(TxSinkState::Sending { transfer });
                Ok(())
            }
            TxSinkState::Sending { .. } => panic!("started sending before polled ready"),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut self.0.as_mut().unwrap() {
            TxSinkState::Ready { .. } => Poll::Ready(Ok(())),
            TxSinkState::Sending { transfer } => match Pin::new(transfer).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready((buf, tx)) => {
                    self.0 = Some(TxSinkState::Ready { tx, buf });
                    Poll::Ready(Ok(()))
                }
            },
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}

macro_rules! tx_sink {
    ($(
        $TxSinkX:ident: ($TxDmaX:ty),
    )+) => {
        $(
            /// A type shorthand for specifying different DMA channels easily.
            pub type $TxSinkX<'a, BUF> = TxSink<'a, BUF, $TxDmaX>;

            impl<'a, BUF> $TxSinkX<'a, BUF> {
                /// Creates a new [`TxSink`] from the specified buffer and DMA transmitter.
                pub fn new(buf: &'a mut BUF, mut tx: $TxDmaX) -> Self {
                    tx.channel.listen(Event::TransferComplete);
                    Self(Some(TxSinkState::Ready {
                        buf,
                        tx,
                    }))
                }
            }
        )+
    }
}

tx_sink!(TxSink1: (TxDma1), TxSink2: (TxDma2), TxSink3: (TxDma3),);

/// A [`Stream`]-based asynchronous abstraction over a DMA receiver.
///
/// # Examples
///
/// ```
/// let mut tx_sink = TxSink3::new(tx_buf, tx.with_dma(channels.2));
/// let mut rx_stream = RxStream3::new(rx_buf, rx.with_dma(channels.3));
/// // Echoes USART3, by sending all items from the infinite RxStream
/// tx_sink.send_all(&mut rx_stream).await?;
/// unreachable!("rx_stream is empty");
/// ```
#[must_use = "streams do nothing unless polled"]
pub struct RxStream<BUF, PAYLOAD>
where
    BUF: 'static,
{
    circ_buffer: CircBuffer<BUF, PAYLOAD>,
    last_read_half: Half,
}

macro_rules! rx_stream {
    ($(
        $RxStreamX:ident: ($INT:ident, $rxdma:ty),
    )+) => {
        $(
            /// A type shorthand for specifying different DMA channels easily.
            pub type $RxStreamX<BUF> = RxStream<BUF, $rxdma>;

            impl<BUF> $RxStreamX<BUF> {
                /// Creates a new [`RxStream`] from the specified buffers and DMA transmitter.
                pub fn new(buf: &'static mut [BUF; 2], mut rx: $rxdma) -> Self
                where
                    BUF: AsMutSlice<Element = u8>,
                {
                    rx.channel.listen(Event::HalfTransfer);
                    rx.channel.listen(Event::TransferComplete);
                    Self {
                        circ_buffer: rx.circ_read(buf),
                        last_read_half: Half::Second,
                    }
                }

                /// Releases the buffers and DMA transmitter.
                pub fn release(self) -> (&'static mut [BUF; 2], $rxdma) {
                    self.circ_buffer.stop()
                }
            }

            impl<BUF> Stream for $RxStreamX<BUF>
            where
                BUF: Clone,
            {
                type Item = Result<BUF, dma::Error>;

                fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    let last_read_half = self.last_read_half;
                    let res = self.circ_buffer.peek(|buf, half| {
                        if half == last_read_half {
                            None
                        } else {
                            Some((buf.clone(), half))
                        }
                    });

                    match res {
                        Ok(Some((buf, half))) => {
                            self.last_read_half = half;
                            Poll::Ready(Some(Ok(buf)))
                        }
                        Ok(None) => {
                            waker_interrupt!($INT, cx.waker().clone());
                            Poll::Pending
                        }
                        Err(err) => Poll::Ready(Some(Err(err))),
                    }
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    (usize::MAX, None)
                }
            }


            impl<BUF> FusedStream for $RxStreamX<BUF>
            where
                BUF: Clone,
            {
                fn is_terminated(&self) -> bool {
                    false
                }
            }
        )+
    }
}

rx_stream!(
    RxStream1: (DMA1_CHANNEL5, RxDma1),
    RxStream2: (DMA1_CHANNEL6, RxDma2),
    RxStream3: (DMA1_CHANNEL3, RxDma3),
);
