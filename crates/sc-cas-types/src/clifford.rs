//! Layer 04: Geometric Clifford Multivectors with 16-bit Blade Bitmasks

/// Metric signature for Clifford algebra Cl(p, q, r).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CliffordSignature {
    /// Positive metric basis vectors (e_i^2 = +1).
    pub p: u8,
    /// Negative metric basis vectors (e_i^2 = -1).
    pub q: u8,
    /// Degenerate / zero metric basis vectors (e_i^2 = 0) (e.g. PGA).
    pub r: u8,
}

impl CliffordSignature {
    /// Standard Euclidean 3D space: Cl(3, 0, 0).
    pub const fn r3() -> Self {
        Self { p: 3, q: 0, r: 0 }
    }

    /// Spacetime algebra (Minkowski): Cl(1, 3, 0).
    pub const fn sta() -> Self {
        Self { p: 1, q: 3, r: 0 }
    }

    /// 3D Projective Geometric Algebra (PGA): Cl(3, 0, 1).
    pub const fn pga3d() -> Self {
        Self { p: 3, q: 0, r: 1 }
    }

    /// Total vector space dimension.
    #[inline]
    pub fn dimension(&self) -> usize {
        (self.p + self.q + self.r) as usize
    }
}

/// Sparse multivector represented as pairs of (16-bit blade bitmask, coefficient).
#[derive(Debug, Clone, PartialEq)]
pub struct Multivector {
    /// Metric signature.
    pub signature: CliffordSignature,
    /// Blade coefficients sorted by blade index.
    pub blades: Vec<(u16, f64)>,
}

impl Multivector {
    /// Create a scalar multivector.
    pub fn scalar(signature: CliffordSignature, val: f64) -> Self {
        Self {
            signature,
            blades: vec![(0, val)],
        }
    }

    /// Create a basis 1-vector e_i (0-indexed).
    pub fn basis_vector(signature: CliffordSignature, i: usize) -> Self {
        assert!(
            i < signature.dimension(),
            "Basis index out of dimension bounds"
        );
        let mask = 1u16 << i;
        Self {
            signature,
            blades: vec![(mask, 1.0)],
        }
    }

    /// Count swaps required to sort blade factors (popcount sign parity).
    #[inline]
    fn blade_product_sign(m1: u16, m2: u16, sig: &CliffordSignature) -> (i8, u16) {
        let mut swaps = 0;

        // For each bit i in m1, count bits in m2 strictly less than i
        for i in 0..16 {
            if (m1 & (1 << i)) != 0 {
                let mask_below = (1u16 << i).wrapping_sub(1);
                swaps += (m2 & mask_below).count_ones();
            }
        }

        let mut sign: i8 = if swaps % 2 != 0 { -1 } else { 1 };

        // Apply metric contractions for overlapping basis vectors (m1 & m2)
        let common = m1 & m2;
        let p = sig.p as usize;
        let q = sig.q as usize;
        let r = sig.r as usize;

        for i in 0..(p + q + r) {
            if (common & (1 << i)) != 0 {
                if i < p {
                    // e_i^2 = +1 (Euclidean positive metric)
                } else if i < p + q {
                    // e_i^2 = -1 (Negative metric)
                    sign = -sign;
                } else {
                    // e_i^2 = 0 (Degenerate metric, e.g. PGA)
                    return (0, 0);
                }
            }
        }

        (sign, m1 ^ m2)
    }

    /// Geometric product of two multivectors: A * B.
    pub fn geometric_product(&self, rhs: &Self) -> Self {
        assert_eq!(self.signature, rhs.signature, "Signatures must match");
        let mut result_map = std::collections::BTreeMap::<u16, f64>::new();

        for &(m1, c1) in &self.blades {
            for &(m2, c2) in &rhs.blades {
                let (sign, out_mask) = Self::blade_product_sign(m1, m2, &self.signature);
                if sign != 0 {
                    let term = (sign as f64) * c1 * c2;
                    *result_map.entry(out_mask).or_insert(0.0) += term;
                }
            }
        }

        let blades: Vec<(u16, f64)> = result_map
            .into_iter()
            .filter(|&(_, c)| c.abs() > 1.0e-15)
            .collect();

        Self {
            signature: self.signature,
            blades,
        }
    }

    /// Outer (wedge) product: A ^ B.
    pub fn wedge(&self, rhs: &Self) -> Self {
        assert_eq!(self.signature, rhs.signature, "Signatures must match");
        let mut result_map = std::collections::BTreeMap::<u16, f64>::new();

        for &(m1, c1) in &self.blades {
            for &(m2, c2) in &rhs.blades {
                // Outer product is non-zero only if blade sets are disjoint
                if (m1 & m2) == 0 {
                    let (sign, out_mask) = Self::blade_product_sign(m1, m2, &self.signature);
                    if sign != 0 {
                        let term = (sign as f64) * c1 * c2;
                        *result_map.entry(out_mask).or_insert(0.0) += term;
                    }
                }
            }
        }

        let blades: Vec<(u16, f64)> = result_map
            .into_iter()
            .filter(|&(_, c)| c.abs() > 1.0e-15)
            .collect();

        Self {
            signature: self.signature,
            blades,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clifford_basis_anticommutation() {
        let sig = CliffordSignature::r3();
        let e1 = Multivector::basis_vector(sig, 0);
        let e2 = Multivector::basis_vector(sig, 1);

        let e1_e2 = e1.geometric_product(&e2);
        let e2_e1 = e2.geometric_product(&e1);

        // e1 * e2 == -(e2 * e1)
        assert_eq!(e1_e2.blades.len(), 1);
        assert_eq!(e2_e1.blades.len(), 1);
        assert_eq!(e1_e2.blades[0].0, e2_e1.blades[0].0); // Same blade index
        assert!((e1_e2.blades[0].1 + e2_e1.blades[0].1).abs() < 1.0e-14); // Opposite signs
    }
}
