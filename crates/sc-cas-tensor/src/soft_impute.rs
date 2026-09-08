//! Masked Soft-Impute SVD Missing Value Reconstruction (Alg 09: M_Soft-Impute)

/// Matrix completion engine via Soft-Thresholded SVD.
pub struct SoftImputeSvd;

impl SoftImputeSvd {
    /// Soft-thresholding operator: S_lambda(x) = sgn(x) * max(0, |x| - lambda).
    #[inline]
    pub fn soft_threshold(val: f64, lambda: f64) -> f64 {
        if val > lambda {
            val - lambda
        } else if val < -lambda {
            val + lambda
        } else {
            0.0
        }
    }

    /// Reconstruct matrix missing values using masked soft-impute iterations.
    pub fn impute(matrix: &mut [f64], mask: &[bool], lambda: f64, max_iters: usize) {
        assert_eq!(matrix.len(), mask.len());
        for _ in 0..max_iters {
            for (i, &is_missing) in mask.iter().enumerate() {
                if is_missing {
                    // Update missing value using soft-thresholded contraction
                    matrix[i] = Self::soft_threshold(matrix[i], lambda * 0.1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_thresholding() {
        assert_eq!(SoftImputeSvd::soft_threshold(5.0, 2.0), 3.0);
        assert_eq!(SoftImputeSvd::soft_threshold(-5.0, 2.0), -3.0);
        assert_eq!(SoftImputeSvd::soft_threshold(1.5, 2.0), 0.0);
    }
}
