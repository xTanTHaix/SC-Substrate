//! NIST FIPS 204 ML-DSA-65 500-Iteration Signature Tamper Injection Suite
//!
//! Asserts that 100% of bit-flipped or corrupted signatures and messages
//! are rejected by the post-quantum attestation verifier (Zero False Acceptance).

use sc_cas_core::pqc::{ml_dsa_sign, ml_dsa_verify, KeyPair};

#[test]
fn test_500_iteration_pqc_signature_tamper_rejection() {
    let seed = [0x42u8; 32];
    let keypair = KeyPair::generate_from_seed(&seed);
    let original_msg = b"ATTESTATION_PAYLOAD_CANONICAL_0x5343535F5345414C";
    let valid_signature = ml_dsa_sign(original_msg, &keypair.secret_key);

    assert!(ml_dsa_verify(
        original_msg,
        &valid_signature,
        &keypair.public_key
    ));

    for i in 0..500 {
        let mut tampered_sig = valid_signature.clone();
        let byte_pos = i % 64; // Flip bits inside commitment R or challenge C
        let bit_mask = 1u8 << (i % 8);
        tampered_sig.0[byte_pos] ^= bit_mask;

        assert!(
            !ml_dsa_verify(original_msg, &tampered_sig, &keypair.public_key),
            "SECURITY VULNERABILITY: Tampered signature accepted at iteration {i}"
        );
    }

    for i in 0..100 {
        let mut tampered_msg = original_msg.to_vec();
        let pos = i % tampered_msg.len();
        tampered_msg[pos] ^= 0x01;

        assert!(
            !ml_dsa_verify(&tampered_msg, &valid_signature, &keypair.public_key),
            "SECURITY VULNERABILITY: Signature verified on tampered message at iteration {i}"
        );
    }
}
