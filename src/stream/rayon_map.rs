use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::sync::Arc;

use futures::channel::oneshot::*;
use futures::ready;
use futures::Future;
use futures::{stream::FusedStream, Stream};
use pin_project::pin_project;

/// Stream for the [`map`](super::RayonStreamExt::cpu_intensive_map) method.
#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub struct RayonMap<St, F, T> {
    #[pin]
    stream: St,
    f: Arc<F>,
    #[pin]
    receiver: Option<Receiver<T>>,
}

impl<St, F, T> fmt::Debug for RayonMap<St, F, T>
where
    St: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CPUIntensiveMap")
            .field("stream", &self.stream)
            .finish()
    }
}

impl<St, F, T> RayonMap<St, F, T> {
    pub(crate) fn new(stream: St, f: F) -> Self {
        Self {
            stream,
            f: Arc::new(f),
            receiver: None,
        }
    }
}

impl<St, F, T> FusedStream for RayonMap<St, F, T>
where
    St: FusedStream,
    Self: Stream, // avoid repeating the bounds of the Stream implementation
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated() && self.receiver.is_none()
    }
}

impl<St, F, T> Stream for RayonMap<St, F, T>
where
    St: Stream,
    St::Item: Send + 'static,
    F: Fn(St::Item) -> T + Send + Sync + 'static,
    T: Send + 'static,
{
    type Item = Result<F::Output, Canceled>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Some(fut) = this.receiver.as_mut().as_pin_mut() {
                let item = ready!(fut.poll(cx));
                this.receiver.set(None);
                break Some(item);
            } else if let Some(item) =
                ready!(this.stream.as_mut().poll_next(cx))
            {
                let (sender, receiver) = channel::<T>();
                let f = (this.f).clone();
                rayon::spawn(move || {
                    let val = f(item);
                    sender
                        .send(val)
                        .unwrap_or_else(|_| panic!("Receiver dropped"));
                });

                //     this.receiver.replace(receiver);
                this.receiver.set(Some(receiver));
            } else {
                // there is no channel waiting for any event, and the stream is
                // empty
                break None;
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let future_len = if self.receiver.is_some() { 1 } else { 0 };
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_add(future_len);
        let upper = match upper {
            Some(x) => x.checked_add(future_len),
            None => None,
        };
        (lower, upper)
    }
}
