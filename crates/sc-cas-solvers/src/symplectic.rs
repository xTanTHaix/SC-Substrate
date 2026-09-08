//! 6th-Order Symplectic Hamiltonian Integrator (Alg 16: Omega_Sympl-6)

/// Trait defining a separable Hamiltonian system: H(p, q) = T(p) + V(q).
pub trait HamiltonianSystem {
    /// Dimension of the phase space coordinates (q, p) in R^d.
    fn dimension(&self) -> usize;

    /// Kinetic gradient: dq/dt = dT/dp.
    fn kinetic_grad(&self, p: &[f64], dq_dt: &mut [f64]);

    /// Potential gradient: dp/dt = -dV/dq.
    fn potential_grad(&self, q: &[f64], dp_dt: &mut [f64]);

    /// Total Hamiltonian energy: H(p, q) = T(p) + V(q).
    fn energy(&self, q: &[f64], p: &[f64]) -> f64;
}

/// 6th-Order Yoshida Symplectic Integrator conserving symplectic 2-form omega.
pub struct Symplectic6Integrator {
    w1: f64,
    w2: f64,
    w3: f64,
    w0: f64,
}

impl Symplectic6Integrator {
    /// Construct a 6th-order integrator with certified Yoshida coefficients.
    pub fn new() -> Self {
        // Yoshida (1990) 6th-order splitting coefficients (Solution A)
        let w1 = -0.117767998417887e1;
        let w2 = 0.235573213359357e0;
        let w3 = 0.784513610477560e0;
        let w0 = 1.0 - 2.0 * (w1 + w2 + w3);

        Self { w1, w2, w3, w0 }
    }

    /// Advance phase space state (q, p) by time step dt using 7-stage composition.
    pub fn step<S: HamiltonianSystem>(&self, system: &S, q: &mut [f64], p: &mut [f64], dt: f64) {
        let weights = [
            self.w3, self.w2, self.w1, self.w0, self.w1, self.w2, self.w3,
        ];
        let dim = system.dimension();
        let mut dq = vec![0.0; dim];
        let mut dp = vec![0.0; dim];

        for &w in &weights {
            let sub_dt = w * dt;

            // Half-step kick: p = p + (sub_dt / 2) * (-dV/dq)
            system.potential_grad(q, &mut dp);
            for i in 0..dim {
                p[i] += 0.5 * sub_dt * dp[i];
            }

            // Full-step drift: q = q + sub_dt * (dT/dp)
            system.kinetic_grad(p, &mut dq);
            for i in 0..dim {
                q[i] += sub_dt * dq[i];
            }

            // Half-step kick: p = p + (sub_dt / 2) * (-dV/dq)
            system.potential_grad(q, &mut dp);
            for i in 0..dim {
                p[i] += 0.5 * sub_dt * dp[i];
            }
        }
    }
}

impl Default for Symplectic6Integrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1D Harmonic Oscillator: H(p, q) = 0.5 * p^2 + 0.5 * q^2.
    struct HarmonicOscillator;

    impl HamiltonianSystem for HarmonicOscillator {
        fn dimension(&self) -> usize {
            1
        }
        fn kinetic_grad(&self, p: &[f64], dq_dt: &mut [f64]) {
            dq_dt[0] = p[0];
        }
        fn potential_grad(&self, q: &[f64], dp_dt: &mut [f64]) {
            dp_dt[0] = -q[0];
        }
        fn energy(&self, q: &[f64], p: &[f64]) -> f64 {
            0.5 * p[0] * p[0] + 0.5 * q[0] * q[0]
        }
    }

    #[test]
    fn test_symplectic_6_energy_conservation() {
        let sys = HarmonicOscillator;
        let mut q = vec![1.0];
        let mut p = vec![0.0];
        let initial_e = sys.energy(&q, &p);

        let integrator = Symplectic6Integrator::new();
        let dt = 0.02;

        // Integrate 1,000 steps
        for _ in 0..1000 {
            integrator.step(&sys, &mut q, &mut p, dt);
        }

        let final_e = sys.energy(&q, &p);
        let drift = (final_e - initial_e).abs() / initial_e;
        assert!(
            drift < 1.0e-11,
            "Energy drift must satisfy symplectic invariant: {drift}"
        );
    }
}
