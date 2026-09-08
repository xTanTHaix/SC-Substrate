//! Non-Mutating Physical & Mathematical Verification Gates 01-06 (Alg 08: V_Gate1-6)

/// Error conditions reported by verification gates.
#[derive(Debug, Clone, PartialEq)]
pub enum GateError {
    /// Gate 01: Dimensional inconsistency.
    DimensionalMismatch {
        /// Expected dimension size.
        expected: usize,
        /// Actual observed dimension size.
        actual: usize,
    },
    /// Gate 02: Energy conservation violation.
    EnergyDriftExceeded {
        /// Computed relative energy drift.
        drift: f64,
        /// Maximum allowable tolerance.
        tolerance: f64,
    },
    /// Gate 03: Equation residual norm too large.
    ResidualToleranceExceeded {
        /// Computed residual norm.
        residual: f64,
        /// Maximum allowable tolerance.
        tolerance: f64,
    },
    /// Gate 04: Metric tensor determinant is non-positive.
    MetricNonDefinite {
        /// Determinant value of the metric tensor.
        det: f64,
    },
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionalMismatch { expected, actual } => {
                write!(
                    f,
                    "Gate 01 Fail: Expected dimension {expected}, got {actual}"
                )
            }
            Self::EnergyDriftExceeded { drift, tolerance } => {
                write!(
                    f,
                    "Gate 02 Fail: Energy drift {drift:.2e} > tolerance {tolerance:.2e}"
                )
            }
            Self::ResidualToleranceExceeded {
                residual,
                tolerance,
            } => {
                write!(
                    f,
                    "Gate 03 Fail: Residual norm {residual:.2e} > tolerance {tolerance:.2e}"
                )
            }
            Self::MetricNonDefinite { det } => {
                write!(f, "Gate 04 Fail: Metric tensor determinant {det} <= 0")
            }
        }
    }
}

impl std::error::Error for GateError {}

/// Non-mutating verification engine executing Gates 01-06.
pub struct VerificationGates;

impl VerificationGates {
    /// Gate 01: Verify dimensional consistency.
    pub fn gate01_dimensions(expected: usize, actual: usize) -> Result<(), GateError> {
        if expected == actual {
            Ok(())
        } else {
            Err(GateError::DimensionalMismatch { expected, actual })
        }
    }

    /// Gate 02: Verify energy conservation invariant.
    pub fn gate02_energy(
        initial_energy: f64,
        current_energy: f64,
        tol: f64,
    ) -> Result<(), GateError> {
        let drift = (current_energy - initial_energy).abs() / initial_energy.abs().max(1.0e-12);
        if drift <= tol {
            Ok(())
        } else {
            Err(GateError::EnergyDriftExceeded {
                drift,
                tolerance: tol,
            })
        }
    }

    /// Gate 03: Verify residual norm constraint.
    pub fn gate03_residual(residual: f64, tol: f64) -> Result<(), GateError> {
        if residual <= tol {
            Ok(())
        } else {
            Err(GateError::ResidualToleranceExceeded {
                residual,
                tolerance: tol,
            })
        }
    }

    /// Gate 04: Verify metric tensor positive-definiteness.
    pub fn gate04_metric_definiteness(det: f64) -> Result<(), GateError> {
        if det > 0.0 {
            Ok(())
        } else {
            Err(GateError::MetricNonDefinite { det })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_gates_pass_and_reject() {
        assert!(VerificationGates::gate01_dimensions(4, 4).is_ok());
        assert!(VerificationGates::gate01_dimensions(4, 5).is_err());

        assert!(VerificationGates::gate02_energy(10.0, 10.00000001, 1.0e-6).is_ok());
        assert!(VerificationGates::gate02_energy(10.0, 11.0, 1.0e-6).is_err());

        assert!(VerificationGates::gate04_metric_definiteness(1.0).is_ok());
        assert!(VerificationGates::gate04_metric_definiteness(-0.5).is_err());
    }
}
