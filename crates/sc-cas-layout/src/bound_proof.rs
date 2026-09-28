//! Static index bounds proof: verify that a linear index expression
//! `f(i) = slope * i + intercept` is provably confined to `[0, N-1]`
//! for all `i` in `[0, domain_size - 1]`.
//!
//! This module encodes the mathematical proof at construction time, not at
//! runtime per-access.  A successfully constructed [`BoundProof`] is a
//! certificate that no index in the expression's domain escapes the buffer.

use crate::error::LayoutError;

/// A closed integer range `[lo, hi]` representing either the input domain or
/// the output range of an index expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexRange {
    /// Inclusive lower bound.
    pub lo: i128,
    /// Inclusive upper bound.
    pub hi: i128,
}

impl IndexRange {
    /// Construct an `IndexRange`, clamping to ensure `lo <= hi`.
    pub fn new(lo: i128, hi: i128) -> Self {
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        Self { lo, hi }
    }
}

/// A proven certificate that the linear index expression `slope * i + intercept`
/// stays within `[0, N-1]` for every `i` in `[0, domain_size - 1]`.
///
/// Constructed via [`BoundProof::certify`].
#[derive(Debug, Clone, Copy)]
pub struct BoundProof {
    /// Coefficient of `i` in the index expression.
    pub slope: i64,
    /// Constant offset in the index expression.
    pub intercept: i64,
    /// Number of distinct input indices (0 to domain_size - 1).
    pub domain_size: u64,
    /// Declared buffer length N; valid indices are `[0, N-1]`.
    pub buffer_size: u64,
    /// The actual output range `[min_output, max_output]` proven at construction.
    pub output_range: IndexRange,
}

impl BoundProof {
    /// Prove that the linear expression `slope * i + intercept` stays within
    /// `[0, N-1]` for all `i in [0, domain_size - 1]`.
    ///
    /// The proof evaluates the expression at the domain boundaries
    /// (i = 0 and i = domain_size - 1) and takes the closed interval of those
    /// outputs, since a linear function achieves its extremes at endpoints.
    ///
    /// # Errors
    /// Returns [`LayoutError::IndexOutOfBounds`] when the maximum index value
    /// produced by the expression exceeds `buffer_size - 1` or the minimum is
    /// negative.
    pub fn certify(
        slope: i64,
        intercept: i64,
        domain_size: u64,
        buffer_size: u64,
    ) -> Result<Self, LayoutError> {
        // Evaluate at both domain endpoints using i128 to prevent overflow.
        let at_zero = intercept as i128;
        let at_max = (slope as i128) * (domain_size.saturating_sub(1) as i128)
            + (intercept as i128);

        let (min_out, max_out) = if at_zero <= at_max {
            (at_zero, at_max)
        } else {
            (at_max, at_zero)
        };

        let n_minus_1 = (buffer_size as i128).saturating_sub(1);

        if min_out < 0 || max_out > n_minus_1 {
            return Err(LayoutError::IndexOutOfBounds {
                max_index: max_out.max(min_out.unsigned_abs() as i128),
                buffer_size,
            });
        }

        Ok(Self {
            slope,
            intercept,
            domain_size,
            buffer_size,
            output_range: IndexRange::new(min_out, max_out),
        })
    }

    /// Evaluate the certified expression for a single `i`.
    ///
    /// # Panics
    /// Panics in debug builds when `i >= domain_size`, as this violates the
    /// domain contract established at proof construction.
    #[inline]
    pub fn index_at(&self, i: u64) -> u64 {
        debug_assert!(
            i < self.domain_size,
            "i={i} exceeds certified domain [0, {}]",
            self.domain_size - 1
        );
        ((self.slope as i128) * (i as i128) + (self.intercept as i128)) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_mapping_certified() {
        // f(i) = 1*i + 0, domain [0, 255], buffer 256.
        let proof = BoundProof::certify(1, 0, 256, 256).unwrap();
        assert_eq!(proof.output_range.lo, 0);
        assert_eq!(proof.output_range.hi, 255);
        assert_eq!(proof.index_at(0), 0);
        assert_eq!(proof.index_at(255), 255);
    }

    #[test]
    fn test_strided_access_certified() {
        // f(i) = 2*i + 0, domain [0, 7], buffer 16 → max = 14, in [0,15].
        let proof = BoundProof::certify(2, 0, 8, 16).unwrap();
        assert_eq!(proof.output_range.hi, 14);
        assert_eq!(proof.index_at(7), 14);
    }

    #[test]
    fn test_offset_access_certified() {
        // f(i) = 1*i + 4, domain [0, 3], buffer 8 → range [4, 7].
        let proof = BoundProof::certify(1, 4, 4, 8).unwrap();
        assert_eq!(proof.output_range.lo, 4);
        assert_eq!(proof.output_range.hi, 7);
    }

    #[test]
    fn test_out_of_bounds_rejected() {
        // f(i) = 1*i + 0, domain [0, 255], buffer 256 — stride 2 → max 510 > 255.
        assert!(matches!(
            BoundProof::certify(2, 0, 256, 256),
            Err(LayoutError::IndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_negative_index_rejected() {
        // f(i) = -1*i + 0, domain [0, 5] → min = -5 < 0.
        assert!(matches!(
            BoundProof::certify(-1, 0, 6, 256),
            Err(LayoutError::IndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_constant_expression_certified() {
        // f(i) = 0*i + 7 always maps to slot 7 in a 16-element buffer.
        let proof = BoundProof::certify(0, 7, 1000, 16).unwrap();
        assert_eq!(proof.output_range.lo, 7);
        assert_eq!(proof.output_range.hi, 7);
        assert_eq!(proof.index_at(999), 7);
    }
}
