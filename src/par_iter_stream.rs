//! Stream adapter for Rayon's parallel iterators.

#![allow(unused_imports)]
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::Arc;

use pin_project::pin_project;

use futures::pin_mut;
use futures::{Stream, StreamExt};
use rayon::iter::ParallelIterator;
use rayon::Scope;

/// Conversion into a `Stream` whose elements are generated in a Rayon thread
/// pool.
///
/// By implementing `IntoRayonStream` for a type, you define how it will be
/// converted to a stream (a.k.a. async iterator), with the particularity that
/// each element of the stream will be generated on a rayon thread pool.
///
/// # Output order
/// There is not guarantee on the order in which the elements generated by the
/// stream will be generated. As an example, the implementation of
/// `IntoRayonStream` for Rayon's `ParallelIterator` might return elements in
/// any order (compatible with the way Rayon manages parallel iterators).
pub trait IntoRayonStream {
    /// Which kind of stream are we turning this into?
    type IntoStream: Stream<Item = Self::Item>;
    /// The type of the elements being iterated over.
    type Item;

    /// Creates a stream from a value.
    fn into_rayon_stream(self) -> Self::IntoStream;
}

impl<T, I> IntoRayonStream for I
where
    I: ParallelIterator<Item = T> + 'static,
    T: 'static,
{
    type IntoStream = ParIterStream<I>;

    type Item = T;

    fn into_rayon_stream(self) -> Self::IntoStream {
        ParIterStream {
            par_iter: Some(self),
            receiver_stream: None,
        }
    }
}
/// A stream wrapping a `ParallelIterator`.
#[pin_project]
pub struct ParIterStream<I: ParallelIterator + 'static> {
    par_iter: Option<I>,
    #[pin]
    receiver_stream: Option<flume::r#async::RecvStream<'static, I::Item>>,
}

/// Transform a parallel iterator into an asynchronous stream.
/// Note that the parallel iterator will be started only upon awaiting the
/// stream's first element. There is no guarantee order on the stream's output.
pub fn to_par_iter_stream<I: ParallelIterator>(
    par_iter: I,
) -> ParIterStream<I> {
    ParIterStream {
        par_iter: Some(par_iter),
        receiver_stream: None,
    }
}

impl<I> Stream for ParIterStream<I>
where
    I: ParallelIterator + 'static,
    // I : ParallelIterator already imposes I::Item to be Send
{
    type Item = I::Item;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        if this.receiver_stream.is_none() {
            // the receiver has not been initialized yet

            // create and run all the machinery necessary to run the parallel
            // iterator

            let (sender, receiver) = flume::unbounded::<I::Item>();

            // launch the Rayon tasks
            let par_iter = this.par_iter.take().unwrap();

            rayon::spawn(move || {
                par_iter.for_each_with(sender, |s, item| {
                    s.send(item).unwrap();
                })
            });

            // set our receiver
            this.receiver_stream.set(Some(receiver.into_stream()));
        }
        // poll the receiver
        this.receiver_stream.as_pin_mut().unwrap().poll_next(cx)
    }
}
