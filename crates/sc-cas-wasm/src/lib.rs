//! Layer 12: Universal Deterministic WebAssembly Substrate & Polyglot FFI
//!
//! Provides a bit-exact, sandbox-isolated mathematical evaluation kernel targeting
//! `wasm32-unknown-unknown` and WASI Component Model, completely free of ambient authority
//! and operating strictly under `#![deny(unsafe_code)]`.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod arena;
pub mod canonical_ieee;
pub mod dispatch;
pub mod wire;

pub use arena::{
    with_arena, with_arena_mut, ArenaError, LinearArena, DEFAULT_ARENA_CAPACITY, WASM_PAGE_SIZE,
};
pub use canonical_ieee::{canonicalize_f32, canonicalize_f64, next_down_f64, next_up_f64};
pub use dispatch::dispatch_request;
pub use wire::{
    decode_request, decode_response, encode_request, encode_response, CanonicalRequest, OpCode,
    StatusCode, WireError, PREAMBLE_CANARY, TRAILING_SEAL_CANARY,
};

/// Canonical Substrate Version identifier (v1.0.0).
pub const SUBSTRATE_VERSION: u32 = 0x0100_0000;

/// Allocates memory inside the linear memory arena with 64-byte alignment.
/// Returns memory offset as a 32-bit unsigned integer (or 0 on failure).
#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn sc_cas_wasm_alloc(size: u32) -> u32 {
    with_arena_mut(|arena| arena.alloc(size as usize).unwrap_or(0))
}

/// Releases memory back to the linear memory arena.
#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn sc_cas_wasm_free(offset: u32, size: u32) {
    with_arena_mut(|arena| {
        let _ = arena.free(offset, size as usize);
    });
}

/// Executes verified CAS kernels and writes the sealed certified envelope to output offset.
/// Returns number of bytes written to output buffer (or 0 on bounds violation).
#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn sc_cas_wasm_dispatch(
    cmd_offset: u32,
    cmd_len: u32,
    out_offset: u32,
    out_cap: u32,
) -> u32 {
    with_arena_mut(|arena| {
        // Linear memory bounds and alignment assertions
        if !cmd_offset.is_multiple_of(64) || !out_offset.is_multiple_of(64) {
            return 0;
        }

        let req_bytes = match arena.read_slice(cmd_offset, cmd_len) {
            Ok(s) => s.to_vec(),
            Err(_) => return 0,
        };

        let resp_bytes = dispatch_request(&req_bytes);

        if resp_bytes.len() > out_cap as usize {
            return 0;
        }

        if arena.write_slice(out_offset, &resp_bytes).is_err() {
            return 0;
        }

        resp_bytes.len() as u32
    })
}

/// Returns canonical substrate version hash.
#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn sc_cas_wasm_version() -> u32 {
    SUBSTRATE_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_abi_lifecycle() {
        let payload = b"simplify: x + x";
        let req = encode_request(OpCode::EvalSymbolic, 128, payload);

        let in_ptr = sc_cas_wasm_alloc(req.len() as u32);
        assert_eq!(in_ptr % 64, 0);

        with_arena_mut(|arena| {
            arena.write_slice(in_ptr, &req).unwrap();
        });

        let out_cap = 65536u32;
        let out_ptr = sc_cas_wasm_alloc(out_cap);
        assert_eq!(out_ptr % 64, 0);

        let written = sc_cas_wasm_dispatch(in_ptr, req.len() as u32, out_ptr, out_cap);
        assert!(written > 0);

        let resp_slice = with_arena(|arena| arena.read_slice(out_ptr, written).unwrap().to_vec());

        let (status, resp_payload, digest) = decode_response(&resp_slice).unwrap();
        assert_eq!(status, StatusCode::Success);
        assert!(String::from_utf8_lossy(&resp_payload).contains("OmniMinCanonical"));
        assert_ne!(digest, [0u8; 32]);

        sc_cas_wasm_free(in_ptr, req.len() as u32);
        sc_cas_wasm_free(out_ptr, out_cap);
    }
}
