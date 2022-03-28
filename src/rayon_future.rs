//! Spawn a Rayon task and run it as a future

use std::future::Future;

type ToFutureError = futures::channel::oneshot::Canceled;

/// Spawn a new task on Rayon's global thread pool and return a future whose
/// output is the task's output.
/// The `f` function is run on Rayon's global thread pool and can be
/// blocking or CPU intensive.
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use futures::stream::{self, StreamExt};
/// use futures::join;
/// use async_rayon::prelude::to_rayon_future;
///
/// let out = to_rayon_future(|| {
///         std::thread::sleep(std::time::Duration::from_millis(200));
///         // Make sure that the closure is run on a Rayon worker thread
///         assert!(rayon::current_thread_index().is_some());
///         42
/// }).await;
///
/// assert_eq!(out, Ok(42));
/// # });
/// ```
// This implementation is not ideal as it starts the concurrent Rayon task right
// away, and not when the future is polled.
pub fn to_rayon_future<F, T>(
    f: F,
) -> impl Future<Output = Result<T, ToFutureError>>
where
    F: Fn() -> T + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = futures::channel::oneshot::channel::<T>();
    rayon::spawn(move || {
        let val = f();
        // Explicitly ignore the error, which is raised when the receiver has
        // been dropped. We cannot do anything smart is this case: it
        // means that the receiving future has been dropped/canceled and we just
        // do not want to crash the entire program because of that.
        let _ = sender.send(val);
    });

    receiver
}
