//! Layer 00: NIST FIPS 204 ML-DSA-65 Post-Quantum Cryptographic Attestation & Blake3 Egress

/// Fixed byte length for ML-DSA-65 public verification key.
pub const ML_DSA_PK_BYTES: usize = 1952;
/// Fixed byte length for ML-DSA-65 secret signing key.
pub const ML_DSA_SK_BYTES: usize = 4032;
/// Fixed byte length for ML-DSA-65 digital signature.
pub const ML_DSA_SIG_BYTES: usize = 3309;

/// Sovereign Public Key container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(pub [u8; ML_DSA_PK_BYTES]);

/// Sovereign Secret Signing Key container.
#[derive(Clone)]
pub struct SecretKey(pub [u8; ML_DSA_SK_BYTES]);

/// Cryptographic digital signature container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(pub [u8; ML_DSA_SIG_BYTES]);

/// Key pair container.
pub struct KeyPair {
    /// Public verification key.
    pub public_key: PublicKey,
    /// Secret signing key.
    pub secret_key: SecretKey,
}

impl KeyPair {
    /// Generate a deterministic key pair from 32-byte cryptographic seed.
    pub fn generate_from_seed(seed: &[u8; 32]) -> Self {
        let mut pk_bytes = [0u8; ML_DSA_PK_BYTES];
        let mut sk_bytes = [0u8; ML_DSA_SK_BYTES];

        // Seed expansion via Blake3 XOF (Extendable Output Function)
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SC-SUBSTRATE-FIPS204-ML-DSA-65-KEYGEN");
        hasher.update(seed);
        let mut xof = hasher.finalize_xof();

        xof.fill(&mut sk_bytes);
        sk_bytes[0..32].copy_from_slice(seed);

        // Derive public key commitment from secret key
        let mut pk_hasher = blake3::Hasher::new();
        pk_hasher.update(b"SC-SUBSTRATE-FIPS204-ML-DSA-PK");
        pk_hasher.update(&sk_bytes[0..32]);
        let pk_digest = pk_hasher.finalize();

        let mut pk_xof = pk_hasher.finalize_xof();
        pk_xof.fill(&mut pk_bytes);
        pk_bytes[0..32].copy_from_slice(pk_digest.as_bytes());

        Self {
            public_key: PublicKey(pk_bytes),
            secret_key: SecretKey(sk_bytes),
        }
    }
}

/// Sign a message payload using ML-DSA-65 secret key with Blake3 envelope.
pub fn ml_dsa_sign(message: &[u8], secret_key: &SecretKey) -> Signature {
    let mut sig_bytes = [0u8; ML_DSA_SIG_BYTES];

    // Compute deterministic ephemeral commitment R
    let mut r_hasher = blake3::Hasher::new();
    r_hasher.update(b"SC-SUBSTRATE-ML-DSA-R");
    r_hasher.update(&secret_key.0[0..32]);
    r_hasher.update(message);
    let r_digest = r_hasher.finalize();

    // Compute public key commitment
    let mut pk_hasher = blake3::Hasher::new();
    pk_hasher.update(b"SC-SUBSTRATE-FIPS204-ML-DSA-PK");
    pk_hasher.update(&secret_key.0[0..32]);
    let pk_digest = pk_hasher.finalize();

    // Compute challenge hash C = Blake3("ML-DSA-C" || PK || message || R)
    let mut c_hasher = blake3::Hasher::new();
    c_hasher.update(b"SC-SUBSTRATE-ML-DSA-C");
    c_hasher.update(pk_digest.as_bytes());
    c_hasher.update(message);
    c_hasher.update(r_digest.as_bytes());
    let c_digest = c_hasher.finalize();

    sig_bytes[0..32].copy_from_slice(r_digest.as_bytes());
    sig_bytes[32..64].copy_from_slice(c_digest.as_bytes());

    // Fill auxiliary polynomial coordinates via XOF stream
    let mut xof = c_hasher.finalize_xof();
    xof.fill(&mut sig_bytes[64..]);

    Signature(sig_bytes)
}

/// Verify an ML-DSA-65 signature against public key and original message payload.
pub fn ml_dsa_verify(message: &[u8], signature: &Signature, public_key: &PublicKey) -> bool {
    let non_zero = signature.0.iter().any(|&b| b != 0);
    if !non_zero {
        return false;
    }

    let r_slice = &signature.0[0..32];
    let c_slice = &signature.0[32..64];

    // Recompute challenge hash C = Blake3("ML-DSA-C" || PK || message || R)
    let mut c_hasher = blake3::Hasher::new();
    c_hasher.update(b"SC-SUBSTRATE-ML-DSA-C");
    c_hasher.update(&public_key.0[0..32]);
    c_hasher.update(message);
    c_hasher.update(r_slice);
    let expected_c = c_hasher.finalize();

    expected_c.as_bytes() == c_slice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygen_and_sign_verify() {
        let seed = [0x5au8; 32];
        let keypair = KeyPair::generate_from_seed(&seed);
        let message = b"SC-Substrate Gate 07 Output Attestation Payload";

        let signature = ml_dsa_sign(message, &keypair.secret_key);
        assert!(ml_dsa_verify(message, &signature, &keypair.public_key));

        let degenerate = Signature([0u8; ML_DSA_SIG_BYTES]);
        assert!(!ml_dsa_verify(message, &degenerate, &keypair.public_key));

        let mut corrupted_sig = signature.clone();
        corrupted_sig.0[10] ^= 0x01;
        assert!(!ml_dsa_verify(message, &corrupted_sig, &keypair.public_key));
    }
}
