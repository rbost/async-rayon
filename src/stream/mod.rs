//! Streams
//!
//! This module defines functions to combine `Stream`s with rayon.
//!

use futures::StreamExt;

pub mod rayon_map;
use rayon_map::RayonMap;

impl<T: ?Sized> RayonStreamExt for T where T: StreamExt {}

pub trait RayonStreamExt: StreamExt {
    fn rayon_map<F, T>(self, f: F) -> RayonMap<Self, F, T>
    where
        Self: Sized,
        Self::Item: Send + 'static,
        F: Fn(Self::Item) -> T + Send + Sync + 'static,
        T: Send + 'static,
    {
        RayonMap::new(self, f)
    }
}
