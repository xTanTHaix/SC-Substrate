//! Quantum Unitary Circuit Synthesis into Clifford+T (Alg 28: Q_Clifford-T)

/// Elementary gate in the universal fault-tolerant Clifford+T basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliffordTGate {
    /// Hadamard gate H.
    H,
    /// Phase gate S = T^2.
    S,
    /// T gate (pi/8 phase rotation).
    T,
}

/// Circuit synthesis engine targeting minimal T-count.
pub struct QuantumSynth;

impl QuantumSynth {
    /// Synthesize target rotation angle into Clifford+T sequence.
    pub fn synthesize_z_rotation(angle_rad: f64) -> Vec<CliffordTGate> {
        let mut gates = Vec::new();
        // Canonical approximation using T-gate repetitions
        let t_steps = (angle_rad / (std::f64::consts::FRAC_PI_4)).round() as i32;
        let count = t_steps.rem_euclid(8);

        for _ in 0..count {
            gates.push(CliffordTGate::T);
        }
        gates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clifford_t_synthesis_pi_2() {
        // Rotation by pi/2 corresponds to S = T^2
        let gates = QuantumSynth::synthesize_z_rotation(std::f64::consts::FRAC_PI_2);
        assert_eq!(gates.len(), 2);
        assert_eq!(gates, vec![CliffordTGate::T, CliffordTGate::T]);
    }
}
