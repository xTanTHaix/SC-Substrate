//! Leaky bucket: burst capacity and drain rate sizing.
//!
//! A leaky bucket accepts bursts up to `capacity` tokens and drains at a
//! constant `drain_rate` (tokens per second).  Unlike the token bucket, it
//! models the smoothing behaviour of an output queue: excess traffic is shaped,
//! not dropped.
//!
//! All time accounting uses exact `u64` microsecond ticks internally to avoid
//! floating-point clock drift.

use crate::error::RatesError;

/// Computed sizing metrics for a leaky bucket.
#[derive(Debug, Clone, Copy)]
pub struct LeakyMetrics {
    /// Burst capacity in tokens.
    pub capacity: u64,
    /// Drain rate in tokens per second.
    pub drain_rate_per_sec: u64,
    /// Maximum burst drain time in microseconds (capacity / drain_rate).
    pub max_drain_time_us: u64,
    /// Minimum inter-token departure interval in microseconds (1_000_000 / drain_rate).
    pub inter_token_gap_us: u64,
}

/// A leaky bucket rate limiter.
///
/// Stores current fill level in tokens.  Tokens accumulate on `add` and drain
/// at a constant rate on `advance_time_us`.
#[derive(Debug, Clone)]
pub struct LeakyBucket {
    /// Maximum tokens the bucket holds before overflow.
    capacity: u64,
    /// Drain rate: tokens drained per second.
    drain_rate_per_sec: u64,
    /// Current fill level in micro-tokens (tokens × 1_000_000) to avoid division
    /// rounding on sub-second intervals.
    fill_micro: u128,
}

impl LeakyBucket {
    /// Construct a leaky bucket.
    ///
    /// - `capacity`: maximum burst size in tokens.
    /// - `drain_rate_per_sec`: tokens drained per second (must be ≥ 1).
    ///
    /// Starts empty.
    ///
    /// # Errors
    /// - [`RatesError::ZeroBurstCapacity`] when `capacity` is zero.
    /// - [`RatesError::ZeroDrainRate`] when `drain_rate_per_sec` is zero.
    pub fn new(capacity: u64, drain_rate_per_sec: u64) -> Result<Self, RatesError> {
        if capacity == 0 {
            return Err(RatesError::ZeroBurstCapacity);
        }
        if drain_rate_per_sec == 0 {
            return Err(RatesError::ZeroDrainRate);
        }
        Ok(Self { capacity, drain_rate_per_sec, fill_micro: 0 })
    }

    /// Compute sizing metrics without mutating state.
    pub fn metrics(&self) -> LeakyMetrics {
        // max_drain_time = capacity / drain_rate (in seconds → convert to µs)
        let max_drain_time_us =
            (self.capacity as u128 * 1_000_000 / self.drain_rate_per_sec as u128) as u64;
        let inter_token_gap_us = 1_000_000 / self.drain_rate_per_sec;
        LeakyMetrics {
            capacity: self.capacity,
            drain_rate_per_sec: self.drain_rate_per_sec,
            max_drain_time_us,
            inter_token_gap_us,
        }
    }

    /// Attempt to add `n` tokens to the bucket.
    ///
    /// Returns `true` when the tokens fit (fill + n ≤ capacity).
    /// Returns `false` when the burst is exceeded and tokens are dropped.
    pub fn add(&mut self, n: u64) -> bool {
        let new_fill_micro = self.fill_micro + (n as u128 * 1_000_000);
        let cap_micro = self.capacity as u128 * 1_000_000;
        if new_fill_micro > cap_micro {
            return false;
        }
        self.fill_micro = new_fill_micro;
        true
    }

    /// Advance the simulated clock by `elapsed_us` microseconds, draining tokens.
    ///
    /// The drain amount is computed exactly in micro-token units to prevent
    /// rounding drift across many small tick advances.
    pub fn advance_time_us(&mut self, elapsed_us: u64) {
        let drained_micro = self.drain_rate_per_sec as u128 * elapsed_us as u128;
        self.fill_micro = self.fill_micro.saturating_sub(drained_micro);
    }

    /// Current fill level in whole tokens (floor).
    #[inline]
    pub fn fill_tokens(&self) -> u64 {
        (self.fill_micro / 1_000_000) as u64
    }

    /// True when the bucket is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fill_micro == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_empty() {
        let lb = LeakyBucket::new(100, 10).unwrap();
        assert!(lb.is_empty());
        assert_eq!(lb.fill_tokens(), 0);
    }

    #[test]
    fn test_add_and_drain() {
        let mut lb = LeakyBucket::new(100, 10).unwrap();
        assert!(lb.add(50));
        assert_eq!(lb.fill_tokens(), 50);
        // Advance 1 second (1_000_000 µs) at 10 tokens/s → drain 10.
        lb.advance_time_us(1_000_000);
        assert_eq!(lb.fill_tokens(), 40);
    }

    #[test]
    fn test_burst_overflow_drops_tokens() {
        let mut lb = LeakyBucket::new(10, 1).unwrap();
        assert!(lb.add(10));
        // Attempting to add 1 more overflows.
        assert!(!lb.add(1));
        assert_eq!(lb.fill_tokens(), 10);
    }

    #[test]
    fn test_drain_floors_at_zero() {
        let mut lb = LeakyBucket::new(100, 1000).unwrap();
        lb.add(5);
        // Drain 10 seconds worth — fill cannot go negative.
        lb.advance_time_us(10_000_000);
        assert!(lb.is_empty());
    }

    #[test]
    fn test_metrics_math() {
        // 1000 tokens capacity, 100 tokens/s → max drain = 10 s = 10_000_000 µs.
        let lb = LeakyBucket::new(1000, 100).unwrap();
        let m = lb.metrics();
        assert_eq!(m.max_drain_time_us, 10_000_000);
        // Inter-token gap: 1_000_000 / 100 = 10_000 µs.
        assert_eq!(m.inter_token_gap_us, 10_000);
    }

    #[test]
    fn test_zero_capacity_rejected() {
        assert!(matches!(
            LeakyBucket::new(0, 10),
            Err(RatesError::ZeroBurstCapacity)
        ));
    }

    #[test]
    fn test_zero_drain_rejected() {
        assert!(matches!(
            LeakyBucket::new(100, 0),
            Err(RatesError::ZeroDrainRate)
        ));
    }
}
