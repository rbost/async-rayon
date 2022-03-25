//! The async-rayon prelude imports the various traits, and methods.
//! The intention is that one can include `use async-rayon::prelude::*` and
//! have easy access to the various traits and methods you will need.
//!
pub use crate::par_iter_stream;
pub use crate::rayon_future;
pub use crate::stream;
