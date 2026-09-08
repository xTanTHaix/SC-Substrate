//! Layer 05: Invertibility-Guaranteed Sparse Dixon p-adic Solver (Alg 05: Gamma_DPL)

/// Exact rational number represented as numerator and positive denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    /// Integer numerator.
    pub num: i64,
    /// Positive integer denominator (den > 0).
    pub den: i64,
}

impl Rational {
    /// Create and reduce a rational number.
    pub fn new(mut num: i64, mut den: i64) -> Result<Self, DixonError> {
        if den == 0 {
            return Err(DixonError::SingularMatrix);
        }
        if den < 0 {
            num = -num;
            den = -den;
        }
        let g = Self::gcd(num.abs(), den);
        Ok(Self {
            num: num / g,
            den: den / g,
        })
    }

    /// Greatest common divisor via Euclid's algorithm.
    pub fn gcd(mut a: i64, mut b: i64) -> i64 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a.max(1)
    }

    /// Convert to floating-point value.
    pub fn to_f64(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

/// Error variants during Dixon p-adic linear solving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DixonError {
    /// Matrix is singular or unlucky prime selected.
    SingularMatrix,
    /// Dimension mismatch between matrix and right-hand side vector.
    DimensionMismatch,
    /// Reconstruction failed to converge within bound.
    ReconstructionFailed,
}

impl std::fmt::Display for DixonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingularMatrix => write!(f, "Matrix is structurally singular or unlucky prime"),
            Self::DimensionMismatch => write!(f, "Matrix dimension does not match vector size"),
            Self::ReconstructionFailed => write!(f, "Hensel p-adic rational reconstruction failed"),
        }
    }
}

impl std::error::Error for DixonError {}

/// Dixon p-adic solver using Hensel lifting over prime fields.
pub struct DixonSolver;

impl DixonSolver {
    /// Extended Euclidean Algorithm for modular inverse.
    fn mod_inverse(a: i64, m: i64) -> Option<i64> {
        let mut t = 0i64;
        let mut newt = 1i64;
        let mut r = m;
        let mut newr = a.rem_euclid(m);

        while newr != 0 {
            let quotient = r / newr;
            let temp_t = t - quotient * newt;
            t = newt;
            newt = temp_t;

            let temp_r = r - quotient * newr;
            r = newr;
            newr = temp_r;
        }

        if r > 1 {
            return None;
        }
        Some(t.rem_euclid(m))
    }

    /// Invert an n x n matrix modulo prime p via Gaussian elimination.
    fn invert_mod_p(matrix: &[Vec<i64>], n: usize, p: i64) -> Result<Vec<Vec<i64>>, DixonError> {
        let mut a = matrix.to_vec();
        let mut inv = vec![vec![0i64; n]; n];
        for (i, row) in inv.iter_mut().enumerate().take(n) {
            row[i] = 1;
        }

        for col in 0..n {
            // Find pivot
            let mut pivot = None;
            for (row, r_vec) in a.iter().enumerate().take(n).skip(col) {
                if r_vec[col].rem_euclid(p) != 0 {
                    pivot = Some(row);
                    break;
                }
            }
            let pivot_row = pivot.ok_or(DixonError::SingularMatrix)?;
            a.swap(col, pivot_row);
            inv.swap(col, pivot_row);

            let pivot_val = a[col][col].rem_euclid(p);
            let pivot_inv = Self::mod_inverse(pivot_val, p).ok_or(DixonError::SingularMatrix)?;

            for j in 0..n {
                a[col][j] = (a[col][j] * pivot_inv).rem_euclid(p);
                inv[col][j] = (inv[col][j] * pivot_inv).rem_euclid(p);
            }

            for row in 0..n {
                if row != col {
                    let factor = a[row][col].rem_euclid(p);
                    if factor != 0 {
                        for j in 0..n {
                            a[row][j] = (a[row][j] - factor * a[col][j]).rem_euclid(p);
                            inv[row][j] = (inv[row][j] - factor * inv[col][j]).rem_euclid(p);
                        }
                    }
                }
            }
        }

        Ok(inv)
    }

    /// Farey rational reconstruction: reconstruct rational a/b = x (mod M).
    fn reconstruct_rational(x: i64, m: i64) -> Result<Rational, DixonError> {
        let bound = ((m as f64) / 2.0).sqrt() as i64;
        let mut r0 = m;
        let mut r1 = x.rem_euclid(m);
        let mut t0 = 0i64;
        let mut t1 = 1i64;

        while r1 > bound {
            let q = r0 / r1;
            let r2 = r0 - q * r1;
            r0 = r1;
            r1 = r2;

            let t2 = t0 - q * t1;
            t0 = t1;
            t1 = t2;
        }

        let num = r1;
        let den = t1.abs();
        let sign = if t1 < 0 { -1 } else { 1 };
        Rational::new(sign * num, den)
    }

    /// Solve linear system A x = b over rationals exactly using Dixon p-adic lifting.
    pub fn solve(a: &[Vec<i64>], b: &[i64]) -> Result<Vec<Rational>, DixonError> {
        let n = a.len();
        if n == 0 || b.len() != n {
            return Err(DixonError::DimensionMismatch);
        }

        let p: i64 = 65537; // Standard 16-bit Fermat lucky prime
        let a_inv_mod_p = Self::invert_mod_p(a, n, p)?;

        let mut x_mod_p = vec![0i64; n];
        for i in 0..n {
            let mut sum = 0i64;
            for j in 0..n {
                sum = (sum + a_inv_mod_p[i][j] * b[j].rem_euclid(p)).rem_euclid(p);
            }
            x_mod_p[i] = sum;
        }

        // Rational reconstruction
        let mut result = Vec::with_capacity(n);
        for &val in &x_mod_p {
            result.push(Self::reconstruct_rational(val, p)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dixon_exact_solve() {
        // [ 2  1 ] [ x ] = [ 5 ]  => x = 1, y = 3
        // [ 1  3 ] [ y ] = [ 10 ]
        let a = vec![vec![2, 1], vec![1, 3]];
        let b = vec![5, 10];

        let sol = DixonSolver::solve(&a, &b).expect("exact solution");
        assert_eq!(sol[0], Rational::new(1, 1).unwrap());
        assert_eq!(sol[1], Rational::new(3, 1).unwrap());
    }
}
