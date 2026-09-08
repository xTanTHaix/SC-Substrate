//! Open Quantum Lindblad Master Equation Solver (Alg 21: Q_Lindblad-CPTP)

/// Quantum density matrix of dimension d x d represented in row-major layout.
#[derive(Debug, Clone, PartialEq)]
pub struct DensityMatrix {
    /// Hilbert space dimension d.
    pub dim: usize,
    /// Real components of density matrix elements: Re(rho_{ij}).
    pub re: Vec<f64>,
    /// Imaginary components of density matrix elements: Im(rho_{ij}).
    pub im: Vec<f64>,
}

impl DensityMatrix {
    /// Construct a normalized pure state density matrix |psi><psi|.
    pub fn pure_state(state_vec: &[(f64, f64)]) -> Self {
        let d = state_vec.len();
        let norm_sq: f64 = state_vec.iter().map(|&(r, i)| r * r + i * i).sum();
        let scale = if norm_sq > 0.0 { 1.0 / norm_sq } else { 1.0 };

        let mut re = vec![0.0; d * d];
        let mut im = vec![0.0; d * d];

        for i in 0..d {
            for j in 0..d {
                let idx = i * d + j;
                // rho_ij = psi_i * psi_j^*
                re[idx] =
                    (state_vec[i].0 * state_vec[j].0 + state_vec[i].1 * state_vec[j].1) * scale;
                im[idx] =
                    (state_vec[i].1 * state_vec[j].0 - state_vec[i].0 * state_vec[j].1) * scale;
            }
        }

        Self { dim: d, re, im }
    }

    /// Compute trace of the density matrix: Tr(rho) = sum_i rho_{ii}.
    pub fn trace(&self) -> f64 {
        let mut tr = 0.0;
        for i in 0..self.dim {
            tr += self.re[i * self.dim + i];
        }
        tr
    }

    /// Enforce Trace = 1 normalization.
    pub fn normalize_trace(&mut self) {
        let tr = self.trace();
        if tr > 0.0 {
            for val in &mut self.re {
                *val /= tr;
            }
            for val in &mut self.im {
                *val /= tr;
            }
        }
    }
}

/// Error variants for Lindblad quantum dynamics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LindbladError {
    /// Density matrix lost positive semidefiniteness.
    PositivityViolation,
}

impl std::fmt::Display for LindbladError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PositivityViolation => write!(f, "Density matrix eigenvalue became negative"),
        }
    }
}

impl std::error::Error for LindbladError {}

/// Open Quantum Lindblad Master Equation CPTP solver.
pub struct LindbladSolver;

impl LindbladSolver {
    /// Advance density matrix by time step dt under dephasing dissipator: D[Z](rho).
    pub fn dephasing_step(rho: &mut DensityMatrix, dephasing_rate: f64, dt: f64) {
        let decay = (-dephasing_rate * dt).exp();
        for i in 0..rho.dim {
            for j in 0..rho.dim {
                if i != j {
                    let idx = i * rho.dim + j;
                    rho.re[idx] *= decay;
                    rho.im[idx] *= decay;
                }
            }
        }
        rho.normalize_trace();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lindblad_trace_preservation_and_dephasing() {
        // Qubit in superposition: (|0> + |1>) / sqrt(2)
        let state = vec![(1.0 / 2.0f64.sqrt(), 0.0), (1.0 / 2.0f64.sqrt(), 0.0)];
        let mut rho = DensityMatrix::pure_state(&state);

        assert!(
            (rho.trace() - 1.0).abs() < 1.0e-15,
            "Initial trace must be 1.0"
        );

        // Apply dephasing dissipation
        LindbladSolver::dephasing_step(&mut rho, 1.5, 0.2);

        assert!(
            (rho.trace() - 1.0).abs() < 1.0e-15,
            "Trace must strictly equal 1.0"
        );
        // Off-diagonal elements must decay
        assert!(rho.re[1] < 0.5, "Coherence must decay under dephasing");
    }
}
