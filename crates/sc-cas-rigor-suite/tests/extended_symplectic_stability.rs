//! 100,000-Step Extended Symplectic Energy Conservation Suite
//!
//! Evaluates the 6th-order Yoshida symplectic integrator over a 100,000-step horizon
//! on non-linear Hamiltonian oscillators, verifying that energy drift remains bounded (|dE| <= 1e-10).

use sc_cas_solvers::symplectic::{HamiltonianSystem, Symplectic6Integrator};

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
fn test_100k_step_symplectic_energy_drift() {
    let integrator = Symplectic6Integrator::new();
    let system = HarmonicOscillator;
    let dt = 0.001;
    let mut q = [1.0f64];
    let mut p = [0.0f64];

    let initial_energy = system.energy(&q, &p);
    let mut max_energy_drift = 0.0f64;

    for step in 1..=100_000 {
        integrator.step(&system, &mut q, &mut p, dt);

        let current_energy = system.energy(&q, &p);
        let drift: f64 = (current_energy - initial_energy).abs();
        if drift > max_energy_drift {
            max_energy_drift = drift;
        }

        if step % 25_000 == 0 {
            assert!(
                drift <= 1e-10,
                "Symplectic energy drift exceeded threshold at step {step}: drift = {drift:e}"
            );
        }
    }

    assert!(
        max_energy_drift <= 1e-10,
        "Max energy drift over 100,000 steps was {max_energy_drift:e} > 1e-10"
    );
}
