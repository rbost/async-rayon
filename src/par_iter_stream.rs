//! Stream adapter for rayon's parallel iterators.

#![allow(unused_imports)]
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::Arc;

use pin_project::pin_project;

use futures::pin_mut;
use futures::{Stream, StreamExt};
use rayon::iter::ParallelIterator;
use rayon::Scope;

#[pin_project]
pub struct ParIterStream<I: ParallelIterator + 'static> {
    // #[pin]
    par_iter: Option<I>,
    #[pin]
    receiver_stream: Option<flume::r#async::RecvStream<'static, I::Item>>,
}

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

            // launch the rayon tasks
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
