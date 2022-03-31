//! Streams adapters and utilities.
//!
//! This module defines functions to combine `Stream`s with Rayon.

use futures::{Stream, StreamExt};
use rayon::iter::ParallelIterator;

mod rayon_map;
pub use rayon_map::RayonMap;

mod par_iter_stream;
pub use par_iter_stream::ParIterStream;

impl<T> RayonStreamExt for T where T: StreamExt + ?Sized {}

/// An extension trait for `Stream`s that provides a variety of convenient
/// combinator functions to interact with Rayon.
pub trait RayonStreamExt: StreamExt {
    /// Applies `map_op` to each item of this stream, producing a new
    /// stream with the results.
    /// The `map_op` function is run on Rayon's global thread pool and can be
    /// blocking or CPU intensive.
    ///
    /// # Examples
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, StreamExt};
    /// use futures::join;
    /// use async_rayon::stream::RayonStreamExt;
    ///
    /// let mut stream = stream::iter(1..=3).rayon_map(|x| {
    ///     // Make sure that the closure is run on a Rayon worker thread
    ///     assert!(rayon::current_thread_index().is_some());
    ///     x+3
    /// });
    ///
    /// assert_eq!(stream.next().await, Some(Ok(4)));
    /// assert_eq!(stream.next().await, Some(Ok(5)));
    /// assert_eq!(stream.next().await, Some(Ok(6)));
    /// assert_eq!(stream.next().await, None);
    /// # });
    /// ```
    ///
    /// You can safely cancel the futures generated by the stream's `next()`
    /// method.
    /// ```
    /// # let rt = tokio::runtime::Builder::new_current_thread()
    /// #                                   .enable_time()
    /// #                                   .build()
    /// #                                   .unwrap();
    /// # rt.block_on(async {
    /// use futures::stream::{self, StreamExt};
    /// use async_rayon::stream::RayonStreamExt;
    /// use std::time::Duration;
    ///
    /// let mut stream = stream::iter(1..=3).rayon_map(|x| {
    ///     // Simulate a CPU-intensive/blocking workload
    ///     std::thread::sleep(std::time::Duration::from_millis(200));
    ///     x+3
    /// });
    /// assert_eq!(stream.next().await, Some(Ok(4)));
    /// assert!(tokio::time::timeout(Duration::from_millis(50),
    ///                                         stream.next()).await.is_err());
    /// assert_eq!(stream.next().await, Some(Ok(5)));
    ///
    /// # });
    /// ```

    fn rayon_map<F, T>(self, map_op: F) -> RayonMap<Self, F, T>
    where
        Self: Sized,
        Self::Item: Send + 'static,
        F: Fn(Self::Item) -> T + Send + Sync + 'static,
        T: Send + 'static,
    {
        RayonMap::new(self, map_op)
    }
}

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
        ParIterStream::new(self)
    }
}
