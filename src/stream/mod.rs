//! Streams
//!
//! This module defines functions to combine `Stream`s with rayon.
//!

use futures::StreamExt;

mod rayon_map;
pub use rayon_map::RayonMap;

impl<T: ?Sized> RayonStreamExt for T where T: StreamExt {}

/// An extension trait for `Stream`s that provides a variety of convenient
/// combinator functions to interact with rayon.
pub trait RayonStreamExt: StreamExt {
    /// Applies `map_op` to each item of this stream, producing a new
    /// stream with the results.
    /// The `map_op` function is run in rayon's global thread pool and can be blocking or CPU intensive.
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
