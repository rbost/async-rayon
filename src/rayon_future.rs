use std::future::Future;

type ToFutureError = futures::channel::oneshot::Canceled;

// This implementation is not ideal as it starts the concurrent rayon task right away, and not when the future is polled.
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