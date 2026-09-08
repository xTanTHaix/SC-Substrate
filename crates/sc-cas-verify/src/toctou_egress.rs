//! Two-Tier Pre-Dispatch Egress Guard with TOCTOU Defense (Alg 13 / Gate 07)

/// Trailing canary value: "SCS_SEAL" in ASCII (0x5343535F5345414C).
pub const SCS_SEAL_CANARY: u64 = 0x5343535F5345414C;

/// Certified egress envelope containing immutable payload, Blake3 digest, and canary seal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifiedEnvelope {
    /// Serialized payload bytes.
    pub payload: Vec<u8>,
    /// 32-byte Blake3 cryptographic attestation digest.
    pub digest: [u8; 32],
    /// 8-byte trailing canary seal.
    pub canary: u64,
}

/// Egress guard preventing Time-of-Check to Time-of-Use race conditions.
pub struct EgressGuard;

impl EgressGuard {
    /// Seal a verified memory buffer into an immutable certified envelope.
    pub fn seal(data: &[u8]) -> CertifiedEnvelope {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SC-SUBSTRATE-GATE07-EGRESS");
        hasher.update(data);
        let digest = *hasher.finalize().as_bytes();

        CertifiedEnvelope {
            payload: data.to_vec(),
            digest,
            canary: SCS_SEAL_CANARY,
        }
    }

    /// Verify integrity of an egress envelope before external consumer dispatch.
    pub fn verify_seal(envelope: &CertifiedEnvelope) -> bool {
        if envelope.canary != SCS_SEAL_CANARY {
            return false;
        }

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SC-SUBSTRATE-GATE07-EGRESS");
        hasher.update(&envelope.payload);
        let expected_digest = *hasher.finalize().as_bytes();

        envelope.digest == expected_digest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate07_egress_sealing_and_tamper_detection() {
        let payload = b"Certified Quantum Symplectic Orbit Trajectory";
        let envelope = EgressGuard::seal(payload);

        assert_eq!(envelope.canary, SCS_SEAL_CANARY);
        assert!(EgressGuard::verify_seal(&envelope));

        // Tamper with payload
        let mut corrupted = envelope.clone();
        corrupted.payload[0] ^= 0xFF;
        assert!(
            !EgressGuard::verify_seal(&corrupted),
            "Tampered payload must fail seal verification"
        );
    }
}
