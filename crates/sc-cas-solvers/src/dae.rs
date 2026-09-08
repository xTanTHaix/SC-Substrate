//! Dimensionally-Partitioned Manifold DAE Solver (Alg 04: Omega_H-DAE)

/// Error conditions in DAE manifold solving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaeError {
    /// Projection failed to converge to constraint manifold.
    ProjectionDivergence,
    /// Constraint Jacobian is rank-deficient.
    RankDeficientJacobian,
}

impl std::fmt::Display for DaeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProjectionDivergence => write!(f, "DAE Newton manifold projection diverged"),
            Self::RankDeficientJacobian => write!(f, "Constraint Jacobian matrix is singular"),
        }
    }
}

impl std::error::Error for DaeError {}

/// Dimensionally-partitioned DAE manifold projection solver.
pub struct DaeManifoldSolver;

impl DaeManifoldSolver {
    /// Project state vector q onto algebraic constraint manifold g(q) = 0.
    pub fn project_manifold<G, J>(
        q: &mut [f64],
        constraint_fn: G,
        jacobian_fn: J,
        tol: f64,
    ) -> Result<(), DaeError>
    where
        G: Fn(&[f64]) -> f64,
        J: Fn(&[f64], &mut [f64]),
    {
        let n = q.len();
        let mut grad = vec![0.0; n];

        for _ in 0..25 {
            let res = constraint_fn(q);
            if res.abs() <= tol {
                return Ok(());
            }

            jacobian_fn(q, &mut grad);
            let grad_norm_sq: f64 = grad.iter().map(|&g| g * g).sum();
            if grad_norm_sq < 1.0e-14 {
                return Err(DaeError::RankDeficientJacobian);
            }

            // Gauss-Newton step: dq = -res * (grad / ||grad||^2)
            let factor = -res / grad_norm_sq;
            for i in 0..n {
                q[i] += factor * grad[i];
            }
        }

        Err(DaeError::ProjectionDivergence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifold_projection_circle_constraint() {
        // Constraint: g(x, y) = x^2 + y^2 - 1 = 0
        let mut q = vec![1.2, 0.9]; // Off-manifold point
        let tol = 1.0e-12;

        DaeManifoldSolver::project_manifold(
            &mut q,
            |pos| pos[0] * pos[0] + pos[1] * pos[1] - 1.0,
            |pos, grad| {
                grad[0] = 2.0 * pos[0];
                grad[1] = 2.0 * pos[1];
            },
            tol,
        )
        .expect("projection success");

        let residual = (q[0] * q[0] + q[1] * q[1] - 1.0).abs();
        assert!(
            residual <= tol,
            "Must lie on manifold within tolerance: {residual}"
        );
    }
}
