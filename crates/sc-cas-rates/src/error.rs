//! Error types for `sc-cas-rates`.

/// All failure modes surfaced by rate-limiting computations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatesError {
    /// Rate numerator and denominator must both be > 0.
    ZeroRateComponent {
        /// Token count (N) component of the rate N/T.
        numerator: u64,
        /// Time period (T) component of the rate N/T.
        denominator: u64,
    },
    /// Burst capacity must be ≥ 1 token.
    ZeroBurstCapacity,
    /// Drain rate must be > 0 tokens per second.
    ZeroDrainRate,
    /// Attempted to consume more tokens than the burst capacity allows.
    BurstExceeded {
        /// Number of tokens requested by the caller.
        requested: u64,
        /// Current available token count.
        capacity: u64,
    },
}

impl std::fmt::Display for RatesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroRateComponent { numerator, denominator } => write!(
                f,
                "Rate N={numerator}/T={denominator}: both components must be > 0"
            ),
            Self::ZeroBurstCapacity => write!(f, "Burst capacity must be at least 1 token"),
            Self::ZeroDrainRate => write!(f, "Drain rate must be > 0 tokens per second"),
            Self::BurstExceeded { requested, capacity } => write!(
                f,
                "Requested {requested} tokens but burst capacity is {capacity}"
            ),
        }
    }
}

impl std::error::Error for RatesError {}
