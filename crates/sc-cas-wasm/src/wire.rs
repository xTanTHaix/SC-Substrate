//! Canonical Wire Format & Sealed Memory Envelopes
//!
//! Encodes deterministic cross-language requests and responses with Blake3 seals,
//! status codes, and preamble/trailing canary validation.

use blake3::Hasher;

/// Preamble canary constant `0x5343434153574153` (`SCCASWAS`).
pub const PREAMBLE_CANARY: [u8; 8] = [0x53, 0x43, 0x43, 0x41, 0x53, 0x57, 0x41, 0x53];

/// Trailing seal canary constant `0x5343535F5345414C` (`SCS_SEAL`).
pub const TRAILING_SEAL_CANARY: [u8; 8] = [0x53, 0x43, 0x53, 0x5F, 0x53, 0x45, 0x41, 0x4C];

/// Operation opcodes for deterministic Wasm dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum OpCode {
    /// Symbolic expression evaluation & canonical simplification.
    EvalSymbolic = 1,
    /// Non-Abelian path-ordered monodromy holonomic integration.
    IntegrateMonodromy = 2,
    /// Sparse Dixon p-adic linear solver with rational reconstruction.
    DixonSolve = 3,
    /// Sound inflection-aware DeepPoly neural verification.
    DeepPolyVerify = 4,
    /// Certified Arb ball arithmetic simplification with cascading precision.
    ArithmeticSimplify = 5,
    /// Clifford multivector geometric product with 16-bit blade bitmasks.
    CliffordMultivector = 6,
    /// Python-semantic strided tensor slicing with negative steps.
    TensorSlice = 7,
}

impl OpCode {
    /// Parses an opcode from a 32-bit unsigned integer.
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            1 => Some(OpCode::EvalSymbolic),
            2 => Some(OpCode::IntegrateMonodromy),
            3 => Some(OpCode::DixonSolve),
            4 => Some(OpCode::DeepPolyVerify),
            5 => Some(OpCode::ArithmeticSimplify),
            6 => Some(OpCode::CliffordMultivector),
            7 => Some(OpCode::TensorSlice),
            _ => None,
        }
    }
}

/// Status codes returned in the canonical response envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StatusCode {
    /// Operation completed successfully and verified through all gates.
    Success = 0,
    /// Operation rejected by non-mutating verification gate pipeline.
    VerificationRejected = 1,
    /// Unrecognized or unsupported opcode.
    InvalidOpCode = 2,
    /// Invalid request bounds or corrupt preamble canary.
    BadPreamble = 3,
    /// Trailing seal canary mismatch or corrupted payload.
    CorruptSeal = 4,
}

/// Parsed canonical request header.
#[derive(Debug, Clone)]
pub struct CanonicalRequest<'a> {
    /// Target operation opcode
    pub opcode: OpCode,
    /// Precision bits or operation-specific flags
    pub flags: u32,
    /// Raw payload bytes
    pub payload: &'a [u8],
}

/// Wire format errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// Input buffer is too short to contain minimal envelope.
    BufferTooShort {
        /// Actual length received
        len: usize,
        /// Minimum required length
        expected: usize,
    },
    /// Preamble canary does not match `SCCASWAS`.
    InvalidPreamble,
    /// Unknown operation opcode.
    UnknownOpCode(u32),
    /// Payload length specified in header exceeds available buffer.
    PayloadLengthMismatch {
        /// Header length
        header_len: usize,
        /// Available length
        available: usize,
    },
    /// Trailing canary does not match `SCS_SEAL`.
    InvalidTrailingCanary,
    /// Attestation digest mismatch upon response verification.
    AttestationDigestMismatch,
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::BufferTooShort { len, expected } => {
                write!(
                    f,
                    "Wire buffer too short: {len} bytes, expected at least {expected}"
                )
            }
            WireError::InvalidPreamble => write!(f, "Invalid wire preamble canary"),
            WireError::UnknownOpCode(op) => write!(f, "Unknown opcode: {op}"),
            WireError::PayloadLengthMismatch {
                header_len,
                available,
            } => {
                write!(
                    f,
                    "Payload length mismatch: declared {header_len}, available {available}"
                )
            }
            WireError::InvalidTrailingCanary => write!(f, "Invalid trailing seal canary"),
            WireError::AttestationDigestMismatch => write!(f, "Blake3 attestation digest mismatch"),
        }
    }
}

impl std::error::Error for WireError {}

/// Encodes a request message into a byte vector with preamble and headers.
pub fn encode_request(opcode: OpCode, flags: u32, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(20 + payload.len());
    buf.extend_from_slice(&PREAMBLE_CANARY);
    buf.extend_from_slice(&(opcode as u32).to_le_bytes());
    buf.extend_from_slice(&flags.to_le_bytes());
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(payload);
    buf
}

/// Decodes and validates a raw request buffer.
pub fn decode_request(buf: &[u8]) -> Result<CanonicalRequest<'_>, WireError> {
    if buf.len() < 20 {
        return Err(WireError::BufferTooShort {
            len: buf.len(),
            expected: 20,
        });
    }

    if buf[0..8] != PREAMBLE_CANARY {
        return Err(WireError::InvalidPreamble);
    }

    let opcode_val = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    let flags = u32::from_le_bytes(buf[12..16].try_into().unwrap());
    let payload_len = u32::from_le_bytes(buf[16..20].try_into().unwrap()) as usize;

    if buf.len() < 20 + payload_len {
        return Err(WireError::PayloadLengthMismatch {
            header_len: payload_len,
            available: buf.len() - 20,
        });
    }

    let opcode = OpCode::from_u32(opcode_val).ok_or(WireError::UnknownOpCode(opcode_val))?;
    let payload = &buf[20..20 + payload_len];

    Ok(CanonicalRequest {
        opcode,
        flags,
        payload,
    })
}

/// Seals and encodes a response envelope with Blake3 hash and trailing canary.
pub fn encode_response(status: StatusCode, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(16 + payload.len() + 32 + 8);
    buf.extend_from_slice(&PREAMBLE_CANARY);
    buf.extend_from_slice(&(status as u32).to_le_bytes());
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(payload);

    // Compute Blake3 attestation digest over preamble + status + payload_len + payload
    let mut hasher = Hasher::new();
    hasher.update(&buf);
    let digest = hasher.finalize();

    buf.extend_from_slice(digest.as_bytes());
    buf.extend_from_slice(&TRAILING_SEAL_CANARY);
    buf
}

/// Decodes and verifies an egress response envelope.
pub fn decode_response(buf: &[u8]) -> Result<(StatusCode, Vec<u8>, [u8; 32]), WireError> {
    if buf.len() < 16 + 32 + 8 {
        return Err(WireError::BufferTooShort {
            len: buf.len(),
            expected: 56,
        });
    }

    if buf[0..8] != PREAMBLE_CANARY {
        return Err(WireError::InvalidPreamble);
    }

    let status_val = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    let payload_len = u32::from_le_bytes(buf[12..16].try_into().unwrap()) as usize;

    let expected_total = 16 + payload_len + 32 + 8;
    if buf.len() < expected_total {
        return Err(WireError::PayloadLengthMismatch {
            header_len: payload_len,
            available: buf.len().saturating_sub(16 + 32 + 8),
        });
    }

    let payload = &buf[16..16 + payload_len];
    let digest_offset = 16 + payload_len;
    let digest_slice = &buf[digest_offset..digest_offset + 32];
    let canary_slice = &buf[digest_offset + 32..digest_offset + 40];

    if canary_slice != TRAILING_SEAL_CANARY {
        return Err(WireError::InvalidTrailingCanary);
    }

    let mut hasher = Hasher::new();
    hasher.update(&buf[0..digest_offset]);
    let computed_digest = hasher.finalize();

    if computed_digest.as_bytes() != digest_slice {
        return Err(WireError::AttestationDigestMismatch);
    }

    let status = match status_val {
        0 => StatusCode::Success,
        1 => StatusCode::VerificationRejected,
        2 => StatusCode::InvalidOpCode,
        3 => StatusCode::BadPreamble,
        _ => StatusCode::CorruptSeal,
    };

    let mut digest = [0u8; 32];
    digest.copy_from_slice(digest_slice);

    Ok((status, payload.to_vec(), digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_request_response_roundtrip() {
        let payload = b"EVAL: 2 * x + 3 * x";
        let req = encode_request(OpCode::EvalSymbolic, 256, payload);
        let parsed = decode_request(&req).unwrap();
        assert_eq!(parsed.opcode, OpCode::EvalSymbolic);
        assert_eq!(parsed.flags, 256);
        assert_eq!(parsed.payload, payload);

        let resp_payload = b"RESULT: 5 * x";
        let resp = encode_response(StatusCode::Success, resp_payload);
        let (status, out_payload, digest) = decode_response(&resp).unwrap();
        assert_eq!(status, StatusCode::Success);
        assert_eq!(out_payload, resp_payload);
        assert_ne!(digest, [0u8; 32]);
    }
}
