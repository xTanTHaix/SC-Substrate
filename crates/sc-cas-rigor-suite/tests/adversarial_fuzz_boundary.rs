//! Adversarial Fuzzing & Extreme Boundary Condition Test Suite
//!
//! Submits denormals, infinity limits, corrupted canaries, misaligned offsets,
//! and mutilated wire envelopes to assert that no unhandled crash can occur.

use sc_cas_types::Ball;
use sc_cas_wasm::canonical_ieee::CANONICAL_NAN_F64_BITS;
use sc_cas_wasm::{
    canonicalize_f64, decode_request, decode_response, dispatch_request, encode_request,
    next_down_f64, next_up_f64, OpCode,
};

#[test]
fn test_extreme_floating_point_boundaries() {
    let min_positive = f64::MIN_POSITIVE;
    let subnormal = min_positive / 2.0;
    assert!(subnormal > 0.0);
    assert_eq!(canonicalize_f64(subnormal), subnormal);

    let up_zero = next_up_f64(0.0);
    let down_zero = next_down_f64(0.0);
    assert!(up_zero > 0.0);
    assert!(down_zero < 0.0);

    assert_eq!(canonicalize_f64(0.0), 0.0);
    assert_eq!(canonicalize_f64(-0.0), -0.0);

    let nans = [
        f64::NAN,
        f64::from_bits(0x7fff_ffff_ffff_ffff),
        f64::from_bits(0x7ff0_0000_0000_0001),
        f64::from_bits(0xfff8_0000_0000_0000),
    ];
    for nan in nans {
        let canon = canonicalize_f64(nan);
        assert_eq!(canon.to_bits(), CANONICAL_NAN_F64_BITS);
    }

    let ball_zero = Ball::new(0.0, 0.1).unwrap();
    let ball_normal = Ball::new(1.0, 0.01).unwrap();
    assert!((ball_normal / ball_zero).is_err());
}

#[test]
fn test_mutilated_wire_packet_fuzzing() {
    let short_buf = vec![0x53, 0x43, 0x43, 0x41];
    assert!(decode_request(&short_buf).is_err());

    let mut bad_preamble = encode_request(OpCode::EvalSymbolic, 128, b"x");
    bad_preamble[0] = 0x00;
    assert!(decode_request(&bad_preamble).is_err());

    let mut bad_opcode = encode_request(OpCode::EvalSymbolic, 128, b"x");
    bad_opcode[8..12].copy_from_slice(&999u32.to_le_bytes());
    assert!(decode_request(&bad_opcode).is_err());

    let mut bad_len = encode_request(OpCode::EvalSymbolic, 128, b"x");
    bad_len[16..20].copy_from_slice(&100_000u32.to_le_bytes());
    assert!(decode_request(&bad_len).is_err());

    let resp = dispatch_request(&encode_request(OpCode::EvalSymbolic, 128, b"sin(x)"));
    let mut tampered_resp = resp.clone();
    let len = tampered_resp.len();
    tampered_resp[len - 1] ^= 0xFF;
    assert!(decode_response(&tampered_resp).is_err());

    let mut payload_tampered = resp;
    payload_tampered[20] ^= 0x55;
    assert!(decode_response(&payload_tampered).is_err());
}
