//!
//! Asynchronous toolkit for Rayon.
//!
//! The `async-rayon` crate aims at making interactions between the Rayon
//! data-parallelism library and Rust's async ecosystem easy. In particular, it
//! aims at being:
//!     - safe: the crate has no unsafe code and uses the
//!       `#![forbid(unsafe_code)]` directive;
//!     - executor-agnostic: no async executor library (such as Tokio,async-std,
//!       or smol) is relied upon. The two main dependencies of this crate are
//!       the `futures` crate, a de-facto standard when it comes to futures in
//!       Rust, and the `flume` crate that implements channels which can be used
//!       both synchronously and asynchronously, and also does not rely on a
//!       specific executor, nor a helper thread;
//!     - sound: the crate supports future's cancellation, while avoiding panics
//!       in Rayon's thread pool.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(unreachable_pub)]
#![warn(rust_2018_idioms)]

pub mod rayon_future;
pub mod stream;

pub mod prelude;
