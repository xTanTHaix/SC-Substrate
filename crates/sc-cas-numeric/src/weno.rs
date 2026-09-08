//! Layer 06: 5th-Order WENO-Z Relativistic Hydrodynamics (Alg 23: F_RHD-WENO)

/// Conservative relativistic hydrodynamics state: (D, S, tau).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HydroState {
    /// Relativistic rest-mass density: D = gamma * rho.
    pub d: f64,
    /// Relativistic momentum density: S = gamma^2 * rho * h * v.
    pub s: f64,
    /// Relativistic energy density: tau = gamma^2 * rho * h - p - D.
    pub tau: f64,
}

/// Primitive variables: (rho, v, p).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimitiveState {
    /// Proper rest-mass density (rho > 0).
    pub rho: f64,
    /// Fluid velocity (v < 1.0).
    pub v: f64,
    /// Thermal pressure (p > 0).
    pub p: f64,
}

/// Error variants for relativistic hydrodynamics state inversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WenoError {
    /// Negative density violates physical state invariants.
    NegativeDensity,
    /// Superluminal fluid velocity detected (v >= 1.0).
    SuperluminalVelocity,
    /// Newton-Raphson primitive recovery failed to converge.
    InversionFailed,
}

impl std::fmt::Display for WenoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeDensity => {
                write!(f, "Physical invariant violated: negative density rho <= 0")
            }
            Self::SuperluminalVelocity => write!(
                f,
                "Physical invariant violated: velocity exceeds light speed |v| >= 1"
            ),
            Self::InversionFailed => write!(
                f,
                "1D Newton-Raphson primitive state inversion failed to converge"
            ),
        }
    }
}

impl std::error::Error for WenoError {}

/// WENO-Z 5th-order relativistic shock-capturing solver.
pub struct WenoSolver;

impl WenoSolver {
    /// Invert conservative state U = (D, S, tau) to primitive (rho, v, p) with physical guards.
    pub fn recover_primitives(
        u: &HydroState,
        gamma_adiab: f64,
    ) -> Result<PrimitiveState, WenoError> {
        if u.d <= 0.0 {
            return Err(WenoError::NegativeDensity);
        }

        // 1D Newton-Raphson solver for pressure p
        let mut p = 1.0e-3;
        for _ in 0..50 {
            let denom = u.tau + p + u.d;
            if denom <= 0.0 {
                return Err(WenoError::InversionFailed);
            }
            let v = u.s / denom;
            if v.abs() >= 1.0 {
                return Err(WenoError::SuperluminalVelocity);
            }

            let lorentz = 1.0 / (1.0 - v * v).sqrt();
            let rho = u.d / lorentz;
            if rho <= 0.0 {
                return Err(WenoError::NegativeDensity);
            }

            // Ideal gas equation of state: e = p / ((gamma - 1) * rho)
            let f =
                (u.tau + p + u.d) * (1.0 - v * v) - u.d - (gamma_adiab / (gamma_adiab - 1.0)) * p;
            let df = 1.0 - v * v - (gamma_adiab / (gamma_adiab - 1.0));

            let step = f / df;
            p = (p - step).max(1.0e-12);

            if step.abs() < 1.0e-10 {
                let final_v = (u.s / (u.tau + p + u.d)).clamp(-0.9999999, 0.9999999);
                return Ok(PrimitiveState { rho, v: final_v, p });
            }
        }

        Err(WenoError::InversionFailed)
    }

    /// 5th-order WENO-Z non-linear stencil reconstruction.
    pub fn weno_z_reconstruct(v: &[f64; 5]) -> f64 {
        let eps = 1.0e-40;

        // Candidate stencils
        let p0 = (2.0 * v[0] - 7.0 * v[1] + 11.0 * v[2]) / 6.0;
        let p1 = (-v[1] + 5.0 * v[2] + 2.0 * v[3]) / 6.0;
        let p2 = (2.0 * v[2] + 5.0 * v[3] - v[4]) / 6.0;

        // Smoothness indicators (Jiang-Shu)
        let beta0 = (13.0 / 12.0) * (v[0] - 2.0 * v[1] + v[2]).powi(2)
            + 0.25 * (v[0] - 4.0 * v[1] + 3.0 * v[2]).powi(2);
        let beta1 =
            (13.0 / 12.0) * (v[1] - 2.0 * v[2] + v[3]).powi(2) + 0.25 * (v[1] - v[3]).powi(2);
        let beta2 = (13.0 / 12.0) * (v[2] - 2.0 * v[3] + v[4]).powi(2)
            + 0.25 * (3.0 * v[2] - 4.0 * v[3] + v[4]).powi(2);

        // Global smoothness indicator tau_5 for WENO-Z
        let tau5 = (beta0 - beta2).abs();

        // WENO-Z non-linear weights
        let d0 = 0.1;
        let d1 = 0.6;
        let d2 = 0.3;

        let alpha0 = d0 * (1.0 + (tau5 / (beta0 + eps)).powi(2));
        let alpha1 = d1 * (1.0 + (tau5 / (beta1 + eps)).powi(2));
        let alpha2 = d2 * (1.0 + (tau5 / (beta2 + eps)).powi(2));

        let sum = alpha0 + alpha1 + alpha2;
        (alpha0 * p0 + alpha1 * p1 + alpha2 * p2) / sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhd_primitive_recovery_and_weno_stencil() {
        let state = HydroState {
            d: 1.0,
            s: 0.1,
            tau: 2.5,
        };
        let prim = WenoSolver::recover_primitives(&state, 4.0 / 3.0).expect("primitive recovered");
        assert!(prim.rho > 0.0, "Density must be positive");
        assert!(
            prim.v.abs() < 1.0,
            "Fluid velocity must be strictly subluminal"
        );

        let stencil = [1.0, 1.2, 1.5, 1.8, 2.0];
        let val = WenoSolver::weno_z_reconstruct(&stencil);
        assert!(val > 1.2 && val < 1.8);
    }
}
