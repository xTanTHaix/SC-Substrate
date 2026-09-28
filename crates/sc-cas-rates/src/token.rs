//! Deterministic token bucket with lossless rational fill rate.
//!
//! The fill rate `R = tokens_per_interval / interval_seconds` is stored as an
//! exact [`Rational`].  Token accounting uses a `u64` counter of accumulated
//! micro-tokens (scaled by the denominator) so no floating-point is ever used
//! during the hot consume/refill loop.

use crate::error::RatesError;
use crate::rational::Rational;

/// Current state of a token bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketState {
    /// Available tokens (in exact integer token units).
    pub available: u64,
    /// Maximum burst capacity.
    pub capacity: u64,
}

impl BucketState {
    /// True when at least `n` tokens are available.
    #[inline]
    pub fn has_tokens(&self, n: u64) -> bool {
        self.available >= n
    }
}

/// A deterministic token bucket whose refill rate is stored as an exact rational.
///
/// Internal token tracking uses integer micro-token accumulation scaled by the
/// denominator of the rate fraction; no floating-point arithmetic occurs in the
/// consume or refill path.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Exact fill rate: `numerator` tokens per `denominator` ticks.
    rate: Rational,
    /// Maximum tokens the bucket can hold (burst capacity).
    capacity: u64,
    /// Accumulated micro-token credit (scaled by `rate.denominator`).
    /// One real token = `rate.denominator` micro-token units.
    micro_tokens: u64,
}

impl TokenBucket {
    /// Construct a new token bucket.
    ///
    /// - `tokens_per_interval`: how many tokens to add per refill tick (N).
    /// - `interval_ticks`: refill period in ticks (T).  The exact rate is N/T.
    /// - `burst_capacity`: maximum tokens the bucket holds.
    ///
    /// Starts full (available == burst_capacity).
    ///
    /// # Errors
    /// - [`RatesError::ZeroRateComponent`] when N or T is zero.
    /// - [`RatesError::ZeroBurstCapacity`] when `burst_capacity` is zero.
    pub fn new(
        tokens_per_interval: u64,
        interval_ticks: u64,
        burst_capacity: u64,
    ) -> Result<Self, RatesError> {
        if burst_capacity == 0 {
            return Err(RatesError::ZeroBurstCapacity);
        }
        let rate = Rational::new(tokens_per_interval, interval_ticks)?;
        // Micro-tokens: start full.  One real token = rate.denominator micro units.
        let micro_tokens = burst_capacity * rate.denominator;
        Ok(Self {
            rate,
            capacity: burst_capacity,
            micro_tokens,
        })
    }

    /// Attempt to consume `n` tokens.
    ///
    /// Returns the updated [`BucketState`] on success.
    ///
    /// # Errors
    /// Returns [`RatesError::BurstExceeded`] when `n` exceeds the burst capacity,
    /// or when insufficient tokens are available (the caller must retry later).
    pub fn consume(&mut self, n: u64) -> Result<BucketState, RatesError> {
        if n > self.capacity {
            return Err(RatesError::BurstExceeded { requested: n, capacity: self.capacity });
        }
        let cost = n * self.rate.denominator;
        if self.micro_tokens < cost {
            return Err(RatesError::BurstExceeded {
                requested: n,
                capacity: self.micro_tokens / self.rate.denominator,
            });
        }
        self.micro_tokens -= cost;
        Ok(self.state())
    }

    /// Advance the bucket by `elapsed_ticks` ticks, adding tokens at the exact rate.
    ///
    /// Fractional token credits accumulate in `tick_remainder` and are only
    /// converted to micro-tokens when a full `denominator` of ticks has elapsed —
    /// this is the key mechanism preventing floating-point rounding drift.
    pub fn refill(&mut self, elapsed_ticks: u64) {
        // Exact micro-token gain: numerator * elapsed_ticks micro-tokens per tick.
        let gain = self.rate.numerator as u128 * elapsed_ticks as u128;
        let max_micro = self.capacity as u128 * self.rate.denominator as u128;
        let new_micro = (self.micro_tokens as u128 + gain).min(max_micro);
        self.micro_tokens = new_micro as u64;
    }

    /// Current snapshot of the bucket state.
    pub fn state(&self) -> BucketState {
        BucketState {
            available: self.micro_tokens / self.rate.denominator,
            capacity: self.capacity,
        }
    }

    /// Exact fill rate as a [`Rational`].
    #[inline]
    pub fn rate(&self) -> Rational {
        self.rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_starts_full() {
        let tb = TokenBucket::new(10, 1, 100).unwrap();
        let state = tb.state();
        assert_eq!(state.available, 100);
        assert_eq!(state.capacity, 100);
    }

    #[test]
    fn test_consume_decrements_exactly() {
        let mut tb = TokenBucket::new(1337, 60, 200).unwrap();
        let after = tb.consume(50).unwrap();
        assert_eq!(after.available, 150);
        let after2 = tb.consume(150).unwrap();
        assert_eq!(after2.available, 0);
    }

    #[test]
    fn test_insufficient_tokens_rejected() {
        let mut tb = TokenBucket::new(10, 1, 50).unwrap();
        tb.consume(50).unwrap();
        // Bucket empty — next consume should fail.
        assert!(matches!(
            tb.consume(1),
            Err(RatesError::BurstExceeded { .. })
        ));
    }

    #[test]
    fn test_refill_caps_at_capacity() {
        let mut tb = TokenBucket::new(100, 1, 50).unwrap();
        tb.consume(50).unwrap(); // drain fully
        tb.refill(1);
        let state = tb.state();
        // After 1 tick with rate 100/1, gain = 100 tokens, but capped at 50.
        assert_eq!(state.available, 50);
    }

    #[test]
    fn test_lossless_rate_roadmap_example() {
        // 1337 req / 60 sec from the roadmap.
        let tb = TokenBucket::new(1337, 60, 5000).unwrap();
        let r = tb.rate();
        // GCD(1337, 60) = 1 (1337 is prime-like), so fraction stays 1337/60.
        assert_eq!(r.numerator, 1337);
        assert_eq!(r.denominator, 60);
    }

    #[test]
    fn test_zero_capacity_rejected() {
        assert!(matches!(
            TokenBucket::new(10, 1, 0),
            Err(RatesError::ZeroBurstCapacity)
        ));
    }

    #[test]
    fn test_zero_rate_rejected() {
        assert!(matches!(
            TokenBucket::new(0, 60, 100),
            Err(RatesError::ZeroRateComponent { .. })
        ));
    }
}
