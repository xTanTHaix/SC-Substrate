//! `sc-cas-layout` — Memory Layout, Cache Alignment & Buffer Bound Safety
//!
//! Provides three complementary tools for low-level and embedded developers:
//! - **`StructLayout`**: compute struct field offsets, padding waste, and
//!   false-sharing footprint on 64-byte CPU cache lines.
//! - **`RingBuffer`**: modular ring-buffer index arithmetic with formal
//!   wrap-around correctness proofs baked into construction.
//! - **`BoundProof`**: static proof that an index expression `f(i)` is
//!   provably confined to `[0, N-1]` for all inputs in a given domain.
//!
//! All computation is pure safe Rust.  No allocations occur during index
//! arithmetic; only `StructLayout` uses a `Vec` for the field list.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod bound_proof;
pub mod error;
pub mod ring_buffer;
pub mod struct_layout;

pub use bound_proof::{BoundProof, IndexRange};
pub use error::LayoutError;
pub use ring_buffer::RingBuffer;
pub use struct_layout::{FieldSpec, StructLayout, StructReport};
