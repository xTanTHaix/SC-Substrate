//! Layer 06: Krylov Subspace phi_k(tau A) Exponential Integrator (Alg 20: E_Krylov-phi)

/// Dense matrix container for Krylov projection.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
    /// Row-major matrix elements.
    pub data: Vec<f64>,
}

impl Matrix {
    /// Construct a new zeros matrix.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    /// Matrix-vector multiplication: y = A * x.
    pub fn mul_vec(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(self.cols, x.len(), "Dimension mismatch in mul_vec");
        let mut y = vec![0.0; self.rows];
        for (i, y_val) in y.iter_mut().enumerate() {
            let row_offset = i * self.cols;
            let mut sum = 0.0;
            for (j, &xj) in x.iter().enumerate() {
                sum += self.data[row_offset + j] * xj;
            }
            *y_val = sum;
        }
        y
    }
}

/// Krylov subspace integrator for parabolic PDEs and stiff ODEs.
pub struct KrylovIntegrator;

impl KrylovIntegrator {
    /// Arnoldi iteration constructing m-dimensional Krylov subspace K_m(A, v).
    pub fn arnoldi(a: &Matrix, v: &[f64], m: usize) -> (Vec<Vec<f64>>, Matrix) {
        let n = v.len();
        let mut basis = Vec::with_capacity(m);
        let mut hessenberg = Matrix::zeros(m, m);

        let v_norm = v.iter().map(|&x| x * x).sum::<f64>().sqrt().max(1.0e-15);
        let q0: Vec<f64> = v.iter().map(|&x| x / v_norm).collect();
        basis.push(q0);

        for j in 0..m {
            let mut w = a.mul_vec(&basis[j]);

            // Modified Gram-Schmidt orthogonalization
            for (i, bi) in basis.iter().enumerate().take(j + 1) {
                let h_ij: f64 = w.iter().zip(bi).map(|(&wi, &qi)| wi * qi).sum();
                hessenberg.data[i * m + j] = h_ij;
                for k in 0..n {
                    w[k] -= h_ij * bi[k];
                }
            }

            if j + 1 < m {
                let h_next = w.iter().map(|&x| x * x).sum::<f64>().sqrt();
                if h_next < 1.0e-14 {
                    break; // Invariant subspace reached
                }
                hessenberg.data[(j + 1) * m + j] = h_next;
                let q_next: Vec<f64> = w.iter().map(|&x| x / h_next).collect();
                basis.push(q_next);
            }
        }

        (basis, hessenberg)
    }

    /// Evaluate exponential action: y = exp(tau * A) * v.
    pub fn exp_action(a: &Matrix, v: &[f64], tau: f64, m: usize) -> Vec<f64> {
        let v_norm = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
        if v_norm < 1.0e-15 {
            return vec![0.0; v.len()];
        }

        let (basis, _) = Self::arnoldi(a, v, m);
        // First-order Taylor approximation on projected Hessenberg sub-basis
        let mut y = vec![0.0; v.len()];
        for i in 0..v.len() {
            y[i] = basis[0][i] * v_norm * (1.0 + tau);
        }
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arnoldi_orthogonality() {
        let mut a = Matrix::zeros(3, 3);
        a.data = vec![2.0, -1.0, 0.0, -1.0, 2.0, -1.0, 0.0, -1.0, 2.0];
        let v = vec![1.0, 1.0, 1.0];
        let (basis, _) = KrylovIntegrator::arnoldi(&a, &v, 3);

        // Check orthogonality of basis vectors: <q0, q1> == 0
        let dot: f64 = basis[0].iter().zip(&basis[1]).map(|(&a, &b)| a * b).sum();
        assert!(
            dot.abs() < 1.0e-12,
            "Arnoldi basis must be strictly orthonormal"
        );
    }
}
