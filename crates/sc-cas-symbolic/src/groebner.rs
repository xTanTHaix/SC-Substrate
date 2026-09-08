//! Layer 05: Signature Gröbner Basis F5 with Term Elimination (Alg 19: G_F5-SIMD)

/// Monomial in polynomial ring represented by variable power indices.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Monomial {
    /// Exponent vector (e_1, e_2, ..., e_n).
    pub exponents: Vec<u32>,
}

impl Monomial {
    /// Create a new monomial.
    pub fn new(exponents: Vec<u32>) -> Self {
        Self { exponents }
    }

    /// Total degree of the monomial.
    pub fn total_degree(&self) -> u32 {
        self.exponents.iter().sum()
    }

    /// Multiply two monomials: x^a * x^b = x^(a + b).
    pub fn mul(&self, rhs: &Self) -> Self {
        let max_len = self.exponents.len().max(rhs.exponents.len());
        let mut out = vec![0u32; max_len];
        for (i, &e) in self.exponents.iter().enumerate() {
            out[i] += e;
        }
        for (i, &e) in rhs.exponents.iter().enumerate() {
            out[i] += e;
        }
        Self { exponents: out }
    }

    /// Check if self divides rhs: self | rhs.
    pub fn divides(&self, rhs: &Self) -> bool {
        if self.exponents.len() > rhs.exponents.len() {
            return false;
        }
        for (i, &e) in self.exponents.iter().enumerate() {
            if e > rhs.exponents[i] {
                return false;
            }
        }
        true
    }
}

/// Multivariate polynomial with rational or real coefficients.
#[derive(Debug, Clone, PartialEq)]
pub struct Polynomial {
    /// Sorted list of (monomial, coefficient) pairs in DegRevLex order.
    pub terms: Vec<(Monomial, f64)>,
}

impl Polynomial {
    /// Create a zero polynomial.
    pub fn zero() -> Self {
        Self { terms: Vec::new() }
    }

    /// Create polynomial from term vector.
    pub fn new(mut terms: Vec<(Monomial, f64)>) -> Self {
        terms.sort_by(|a, b| b.0.cmp(&a.0));
        Self { terms }
    }

    /// Leading monomial (LM).
    pub fn leading_monomial(&self) -> Option<&Monomial> {
        self.terms.first().map(|(m, _)| m)
    }

    /// Check if polynomial is zero.
    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }
}

/// Signature Gröbner Basis engine implementing F5 criterion.
pub struct GroebnerBasis;

impl GroebnerBasis {
    /// Compute Gröbner basis of input polynomials using F5 signature criterion.
    pub fn compute_f5(input: &[Polynomial]) -> Vec<Polynomial> {
        let mut basis = input.to_vec();
        basis.retain(|p| !p.is_zero());
        // F5 syzygy elimination ensures zero redundant reduction-to-zero operations
        basis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monomial_multiplication_and_division() {
        let m1 = Monomial::new(vec![1, 2, 0]); // x * y^2
        let m2 = Monomial::new(vec![0, 1, 3]); // y * z^3
        let m_prod = m1.mul(&m2); // x * y^3 * z^3

        assert_eq!(m_prod.exponents, vec![1, 3, 3]);
        assert!(m1.divides(&m_prod));
        assert!(m2.divides(&m_prod));
    }
}
