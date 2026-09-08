//! Layer 05: Lie Point Symmetry & Similarity Reduction for PDEs (Alg 29: L_Lie-PDE)

/// Infinitesimal generator: X = xi^x * d/dx + xi^t * d/dt + eta^u * d/du.
#[derive(Debug, Clone, PartialEq)]
pub struct InfinitesimalGenerator {
    /// Spatial coefficient xi^x(x, t, u).
    pub xi_x: String,
    /// Temporal coefficient xi^t(x, t, u).
    pub xi_t: String,
    /// Dependent variable coefficient eta^u(x, t, u).
    pub eta_u: String,
}

/// Lie symmetry analyzer reducing non-linear PDEs to ODEs.
pub struct LieSymmetry;

impl LieSymmetry {
    /// Compute traveling wave similarity variable: xi = x - c * t.
    pub fn traveling_wave_ansatz(wave_speed_c: f64) -> String {
        format!("xi = x - {} * t", wave_speed_c)
    }

    /// Compute scaling similarity variable: xi = x / t^(1/alpha).
    pub fn scaling_ansatz(alpha: f64) -> String {
        format!("xi = x / (t ^ (1.0 / {}))", alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lie_similarity_variable_generation() {
        let ansatz = LieSymmetry::traveling_wave_ansatz(2.5);
        assert_eq!(ansatz, "xi = x - 2.5 * t");
    }
}
