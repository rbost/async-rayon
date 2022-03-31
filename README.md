# Async Rayon

The `async-rayon` is a compatibility layer making
[Rayon](https://github.com/rayon-rs/rayon/) easy to use in asynchronous contexts.

## Features

- [x] Run a closure on the Rayon thread pool, and wait for the result from the
  call site.
- [x] Run parallel iterators.
- [x] A `map`-like adapter for asynchronous streams, whose closure is run on
  the Rayon thread pool
- [ ] A `for_each`-like adapter for asynchronous streams, whose closure is run on
  the Rayon thread pool
- [ ] A `fold`-like adapter for asynchronous streams, whose closure is run on
  the Rayon thread pool
- [ ] A `reduce`-like adapter for asynchronous streams, whose closure is run on
  the Rayon thread pool
- [ ] Choosing the thread pool on which closures will be run from the code.

## Goals

- Easy and intuitive to use: using a parallel iterator in async code must be as
  easy as combining streams.
- Sound: the code must never block or result in an unexpected behavior (for
  example, panicking because a future was cancelled).
- Safe: the crate has no unsafe code and uses the `#![forbid(unsafe_code)]`
  directive to prevent any use of unsafe code.
- Executor-agnostic: no async executor library (such as [Tokio](https://tokio.rs),
  [async-std](https://github.com/async-rs/async-std), or
  [smol](https://github.com/smol-rs/smol)) is relied upon. Nor a helper thread.
  The two dependencies allowing this crate are the
  [`futures`](https://github.com/rust-lang/futures-rs) crate (for its `oneshot`
  channel), and the [`flume`](https://github.com/zesterer/flume) crate (an MPMC
  channel that can be used both in synchronous and asynchronous contexts).

## Non Goals

- Non-`'static` tasks: rather than a non-goal, this is due to a Rust
  'limitation'. Indeed, destructors are not guaranteed to be called. This
  prevents the implementation of safe join handles for threads. See
  [this blog post](https://cglab.ca/~abeinges/blah/everyone-poops/) for a more
  detailed explanation of the problem. Note that there is no real hope of
  solving this issue (without using unsafe code) in the foreseeable future.
- Using thread pools other than Rayon: we could use a different async executor
  to run CPU-intensive code without interfering with the code dealing with IOs
  in the main executor. This has the advantage of being able to make async calls
  within the CPU-intensive code. However, we would not be able to use Rayon's
  parallel iterators properly.

## License

Rayon is distributed under the terms of both the MIT license and the
Apache License (Version 2.0). See [LICENSE-APACHE](LICENSE-APACHE) and
[LICENSE-MIT](LICENSE-MIT) for details. Opening a pull requests is
assumed to signal agreement with these licensing terms.
