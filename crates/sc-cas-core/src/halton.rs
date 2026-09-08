//! Layer 02: Low-Discrepancy Halton Sequence Generator

/// Error variants for Halton sequence operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltonError {
    /// Dimension exceeds the precomputed prime table capacity.
    DimensionExceeded {
        /// The dimension requested by the caller.
        requested: usize,
        /// The maximum supported dimension.
        max: usize,
    },
    /// Dimension must be non-zero.
    ZeroDimension,
}

impl std::fmt::Display for HaltonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionExceeded { requested, max } => {
                write!(
                    f,
                    "Requested dimension {requested} exceeds max prime table {max}"
                )
            }
            Self::ZeroDimension => write!(f, "Dimension must be at least 1"),
        }
    }
}

impl std::error::Error for HaltonError {}

/// First 64 prime bases for deterministic quasi-Monte Carlo Halton evaluation.
const PRIMES: [u64; 64] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
    311,
];

/// Deterministic 64-dimensional Halton sequence generator with radical inversion.
#[derive(Debug, Clone)]
pub struct HaltonSequence {
    dimension: usize,
    index: u64,
    scramble_seed: u64,
}

impl HaltonSequence {
    /// Create a new Halton sequence generator with target dimension and scramble seed.
    pub fn new(dimension: usize, scramble_seed: u64) -> Result<Self, HaltonError> {
        if dimension == 0 {
            return Err(HaltonError::ZeroDimension);
        }
        if dimension > PRIMES.len() {
            return Err(HaltonError::DimensionExceeded {
                requested: dimension,
                max: PRIMES.len(),
            });
        }
        Ok(Self {
            dimension,
            index: 1,
            scramble_seed,
        })
    }

    /// Compute the radical inverse of integer `n` in base `b`.
    #[inline]
    fn radical_inverse(mut n: u64, base: u64, scramble: u64) -> f64 {
        let mut f = 1.0;
        let mut r = 0.0;
        let base_f = base as f64;

        while n > 0 {
            f /= base_f;
            let digit = (n % base) ^ (scramble % base);
            r += (digit % base) as f64 * f;
            n /= base;
        }
        r
    }

    /// Generate the next low-discrepancy vector in [0, 1)^d.
    pub fn next_point(&mut self) -> Vec<f64> {
        let mut point = Vec::with_capacity(self.dimension);
        for (dim, &base) in PRIMES.iter().enumerate().take(self.dimension) {
            let scramble = (self.scramble_seed.wrapping_mul(base)) ^ (dim as u64);
            let val = Self::radical_inverse(self.index, base, scramble);
            point.push(val);
        }
        self.index = self.index.wrapping_add(1);
        point
    }

    /// Current iteration count.
    #[inline]
    pub fn current_index(&self) -> u64 {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_halton_bounds_and_determinism() {
        let mut seq1 = HaltonSequence::new(4, 42).expect("valid creation");
        let mut seq2 = HaltonSequence::new(4, 42).expect("valid creation");

        for _ in 0..1000 {
            let p1 = seq1.next_point();
            let p2 = seq2.next_point();
            assert_eq!(p1, p2, "Halton sequence must be strictly deterministic");
            for &coord in &p1 {
                assert!(
                    (0.0..1.0).contains(&coord),
                    "Coordinates must reside in [0, 1)"
                );
            }
        }
    }
}
