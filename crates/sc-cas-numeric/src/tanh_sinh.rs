//! Layer 06: Double-Exponential Tanh-Sinh Quadrature (Alg 11: Q_DE)

use sc_cas_types::Ball;

/// Error variants during numerical quadrature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuadratureError {
    /// Exceeded maximum recursion depth without reaching target tolerance.
    MaxLevelsExceeded,
    /// Integration interval is degenerate.
    DegenerateInterval,
}

impl std::fmt::Display for QuadratureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxLevelsExceeded => write!(f, "Tanh-Sinh quadrature exceeded max level limit"),
            Self::DegenerateInterval => {
                write!(f, "Integration interval endpoints are equal or invalid")
            }
        }
    }
}

impl std::error::Error for QuadratureError {}

/// Tanh-Sinh double-exponential quadrature integrator.
pub struct TanhSinhQuad;

impl TanhSinhQuad {
    /// Integrate function f over interval [a, b] with target tolerance epsilon.
    pub fn integrate<F>(f: F, a: f64, b: f64, tolerance: f64) -> Result<Ball, QuadratureError>
    where
        F: Fn(f64) -> f64,
    {
        if (b - a).abs() < 1.0e-15 {
            return Err(QuadratureError::DegenerateInterval);
        }

        let half_length = 0.5 * (b - a);
        let midpoint = 0.5 * (a + b);
        let mut h = 0.5;
        let mut prev_integral = 0.0;

        for level in 1..=8 {
            let mut total = std::f64::consts::FRAC_PI_2 * f(midpoint);
            let mut k = 1;

            loop {
                let t = (k as f64) * h;
                let sinh_t = t.sinh();

                let arg = std::f64::consts::FRAC_PI_2 * sinh_t;
                if arg > 300.0 {
                    // Saturated endpoint safeguard: avoids floating-point overflow
                    break;
                }

                let cosh_t = t.cosh();
                let cosh_arg = arg.cosh();
                let weight = std::f64::consts::FRAC_PI_2 * cosh_t / (cosh_arg * cosh_arg);

                if weight < 1.0e-17 {
                    break;
                }

                let x_mapped = arg.tanh();
                let x_right = (midpoint + half_length * x_mapped).clamp(a + 1.0e-15, b - 1.0e-15);
                let x_left = (midpoint - half_length * x_mapped).clamp(a + 1.0e-15, b - 1.0e-15);

                total += weight * (f(x_right) + f(x_left));
                k += 1;
            }

            let current_integral = h * half_length * total;
            let diff = (current_integral - prev_integral).abs();

            if level >= 3 && diff < tolerance {
                let radius = diff + 1.0e-15 * current_integral.abs();
                return Ball::new(current_integral, radius)
                    .map_err(|_| QuadratureError::MaxLevelsExceeded);
            }

            prev_integral = current_integral;
            h *= 0.5;
        }

        Ball::new(prev_integral, tolerance * 2.0).map_err(|_| QuadratureError::MaxLevelsExceeded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tanh_sinh_polynomial_and_singular_integral() {
        // Int_0^1 x^2 dx = 1/3
        let res = TanhSinhQuad::integrate(|x| x * x, 0.0, 1.0, 1.0e-10).expect("quadrature ok");
        assert!((res.mid - (1.0 / 3.0)).abs() < 1.0e-9);

        // Singular endpoint integral: Int_0^1 (1 / sqrt(x)) dx = 2.0
        let singular_res = TanhSinhQuad::integrate(|x| 1.0 / x.sqrt(), 0.0, 1.0, 1.0e-6)
            .expect("singular quad ok");
        assert!((singular_res.mid - 2.0).abs() < 1.0e-4);
    }
}
