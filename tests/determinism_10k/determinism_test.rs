//! Determinism 10,000-Cycle Bit-Exact Replay Suite
//!
//! Executes 10,000 identical mathematical operations and verifies
//! that every single Blake3 attestation digest matches bit-for-bit.

#[test]
fn test_10k_bit_exact_replay() {
    use sc_cas_wasm::{dispatch_request, encode_request, decode_response, OpCode, StatusCode};

    let req = encode_request(OpCode::EvalSymbolic, 256, b"exp(i * pi) + 1");
    let initial_resp = dispatch_request(&req);
    let (status, payload, initial_digest) = decode_response(&initial_resp).unwrap();
    assert_eq!(status, StatusCode::Success);

    for cycle in 1..=10_000 {
        let resp = dispatch_request(&req);
        let (s, p, digest) = decode_response(&resp).unwrap();
        assert_eq!(s, StatusCode::Success);
        assert_eq!(p, payload, "Payload diverged at cycle {cycle}");
        assert_eq!(digest, initial_digest, "Bit-exact Blake3 digest diverged at cycle {cycle}");
    }
}
