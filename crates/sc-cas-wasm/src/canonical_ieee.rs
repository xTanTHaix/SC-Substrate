//! IEEE-754 Canonical NaN Normalization & Software Directed Rounding
//!
//! Enforces bit-exact reproducibility across divergent host architectures (x86, ARM, RISC-V)
//! and JavaScript runtimes (V8, SpiderMonkey, JSC).

/// Canonical NaN bit pattern for 64-bit IEEE-754 floating point numbers.
pub const CANONICAL_NAN_F64_BITS: u64 = 0x7ff8_0000_0000_0000;

/// Canonical NaN bit pattern for 32-bit IEEE-754 floating point numbers.
pub const CANONICAL_NAN_F32_BITS: u32 = 0x7fc0_0000;

/// Sanitizes 64-bit floats to enforce canonical NaN bit pattern.
pub fn canonicalize_f64(v: f64) -> f64 {
    if v.is_nan() {
        f64::from_bits(CANONICAL_NAN_F64_BITS)
    } else {
        v
    }
}

/// Sanitizes 32-bit floats to enforce canonical NaN bit pattern.
pub fn canonicalize_f32(v: f32) -> f32 {
    if v.is_nan() {
        f32::from_bits(CANONICAL_NAN_F32_BITS)
    } else {
        v
    }
}

/// Software directed rounding: returns the next representable IEEE-754 float toward positive infinity.
pub fn next_up_f64(x: f64) -> f64 {
    if x.is_nan() || x == f64::INFINITY {
        return x;
    }
    if x == -f64::INFINITY {
        return -f64::MAX;
    }
    if x == 0.0 {
        return f64::from_bits(1);
    }
    let bits = x.to_bits();
    if x > 0.0 {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    }
}

/// Software directed rounding: returns the next representable IEEE-754 float toward negative infinity.
pub fn next_down_f64(x: f64) -> f64 {
    if x.is_nan() || x == -f64::INFINITY {
        return x;
    }
    if x == f64::INFINITY {
        return f64::MAX;
    }
    if x == 0.0 {
        return -f64::from_bits(1);
    }
    let bits = x.to_bits();
    if x > 0.0 {
        f64::from_bits(bits - 1)
    } else {
        f64::from_bits(bits + 1)
    }
}

/// Evaluates a certified addition $[a_l, a_u] + [b_l, b_u]$ using software directed rounding.
pub fn add_directed(a_low: f64, a_high: f64, b_low: f64, b_high: f64) -> (f64, f64) {
    let low = next_down_f64(a_low + b_low);
    let high = next_up_f64(a_high + b_high);
    (low, high)
}

/// Evaluates a certified multiplication $[a_l, a_u]     imes [b_l, b_u]$ using software directed rounding.
pub fn mul_directed(a_low: f64, a_high: f64, b_low: f64, b_high: f64) -> (f64, f64) {
    let p1 = a_low * b_low;
    let p2 = a_low * b_high;
    let p3 = a_high * b_low;
    let p4 = a_high * b_high;

    let min_p = p1.min(p2).min(p3).min(p4);
    let max_p = p1.max(p2).max(p3).max(p4);

    (next_down_f64(min_p), next_up_f64(max_p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_nan() {
        let nan1 = f64::NAN;
        let nan2 = f64::from_bits(0x7fff_ffff_ffff_ffff);
        assert_eq!(canonicalize_f64(nan1).to_bits(), CANONICAL_NAN_F64_BITS);
        assert_eq!(canonicalize_f64(nan2).to_bits(), CANONICAL_NAN_F64_BITS);

        let normal = std::f64::consts::PI;
        assert_eq!(canonicalize_f64(normal), normal);
    }

    #[test]
    fn test_software_directed_rounding() {
        let zero = 0.0f64;
        let up = next_up_f64(zero);
        let down = next_down_f64(zero);
        assert!(up > 0.0);
        assert!(down < 0.0);

        let (low, high) = add_directed(1.0, 1.0, 2.0, 2.0);
        assert!(low <= 3.0);
        assert!(high >= 3.0);
    }
}
