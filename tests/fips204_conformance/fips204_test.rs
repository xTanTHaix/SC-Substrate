//! NIST FIPS 204 ML-DSA-65 Conformance & Attestation Suite
//!
//! Validates post-quantum digital signature generation and verification
//! over Blake3 certified envelope digests.

#[test]
fn test_fips204_ml_dsa_65_signing() {
    use sc_cas_core::pqc_signer::PqcSigner;

    let signer = PqcSigner::generate();
    let message = b"SC-SUBSTRATE-CANONICAL-ATTESTATION-DIGEST-0x5343535F5345414C";
    let sig = signer.sign(message);
    assert!(signer.verify(message, &sig));
}
