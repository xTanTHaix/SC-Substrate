//! `sc-cas-rates` — Lossless Rate Limiting & Token Bucket Sizing
//!
//! Provides two rate-limiting abstractions built on exact rational arithmetic:
//! - **`TokenBucket`**: deterministic token bucket where the refill rate
//!   `R = N/T` is stored as a reduced fraction — no floating-point drift.
//! - **`LeakyBucket`**: leaky bucket sizing for burst capacity and drain rate.
//!
//! Rational arithmetic guarantees that repeated quota computations never
//! accumulate rounding error, critical for long-running gateway processes.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod leaky;
pub mod rational;
pub mod token;

pub use error::RatesError;
pub use leaky::{LeakyBucket, LeakyMetrics};
pub use rational::Rational;
pub use token::{BucketState, TokenBucket};
