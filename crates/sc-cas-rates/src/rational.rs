//! Exact rational arithmetic for lossless rate representation.
//!
//! `Rational` stores a fraction `p/q` in canonical reduced form (GCD-normalised,
//! denominator always positive).  All arithmetic operations produce a new
//! reduced rational, so accumulated rounding error is structurally impossible.

use crate::error::RatesError;

/// Compute GCD via the iterative binary (Stein) algorithm — no division.
fn gcd(mut a: u64, mut b: u64) -> u64 {
    if a == 0 { return b; }
    if b == 0 { return a; }
    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    loop {
        b >>= b.trailing_zeros();
        if a > b { std::mem::swap(&mut a, &mut b); }
        b -= a;
        if b == 0 { break; }
    }
    a << shift
}

/// An exact non-negative rational number `numerator / denominator` in reduced form.
///
/// Both components are `u64`; the value is always non-negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    /// Reduced numerator.
    pub numerator: u64,
    /// Reduced denominator (always ≥ 1).
    pub denominator: u64,
}

impl Rational {
    /// Construct a reduced rational from `numerator / denominator`.
    ///
    /// # Errors
    /// Returns [`RatesError::ZeroRateComponent`] when either component is zero.
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, RatesError> {
        if numerator == 0 || denominator == 0 {
            return Err(RatesError::ZeroRateComponent { numerator, denominator });
        }
        let g = gcd(numerator, denominator);
        Ok(Self { numerator: numerator / g, denominator: denominator / g })
    }

    /// Evaluate as an `f64` for display or threshold comparisons.
    ///
    /// This conversion is deliberately the only place where precision is lost;
    /// all internal arithmetic stays exact.
    #[inline]
    pub fn to_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// Compute `self * scalar` exactly as a new reduced `Rational`.
    ///
    /// Uses `u128` intermediate to prevent overflow before reduction.
    pub fn mul_u64(self, scalar: u64) -> Self {
        let num = self.numerator as u128 * scalar as u128;
        let den = self.denominator as u128;
        // Reduce using u128 GCD to handle large products safely.
        let g = {
            let mut a = num;
            let mut b = den;
            while b != 0 { let t = b; b = a % b; a = t; }
            a
        };
        Self {
            numerator: (num / g) as u64,
            denominator: (den / g) as u64,
        }
    }


    /// True when this rational is greater than or equal to `other`.
    #[inline]
    pub fn ge(self, other: Self) -> bool {
        // Cross-multiply using u128 to avoid overflow.
        (self.numerator as u128) * (other.denominator as u128)
            >= (other.numerator as u128) * (self.denominator as u128)
    }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{} ({:.6})", self.numerator, self.denominator, self.to_f64())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduction_to_canonical_form() {
        let r = Rational::new(6, 4).unwrap();
        assert_eq!(r.numerator, 3);
        assert_eq!(r.denominator, 2);
    }

    #[test]
    fn test_prime_fraction_unchanged() {
        let r = Rational::new(7, 13).unwrap();
        assert_eq!(r.numerator, 7);
        assert_eq!(r.denominator, 13);
    }

    #[test]
    fn test_to_f64_precision() {
        // 1337 req / 60 sec = the roadmap example.
        let r = Rational::new(1337, 60).unwrap();
        let expected = 1337.0 / 60.0;
        assert!((r.to_f64() - expected).abs() < 1e-12);
    }

    #[test]
    fn test_zero_component_rejected() {
        assert!(matches!(
            Rational::new(0, 60),
            Err(RatesError::ZeroRateComponent { numerator: 0, .. })
        ));
        assert!(matches!(
            Rational::new(1337, 0),
            Err(RatesError::ZeroRateComponent { denominator: 0, .. })
        ));
    }

    #[test]
    fn test_ge_comparison() {
        let a = Rational::new(3, 4).unwrap(); // 0.75
        let b = Rational::new(2, 3).unwrap(); // 0.666…
        assert!(a.ge(b));
        assert!(!b.ge(a));
        // Equal fractions.
        let c = Rational::new(6, 8).unwrap(); // reduces to 3/4
        assert!(a.ge(c));
        assert!(c.ge(a));
    }
}
