//! Layer 11: Non-Mutating Verification Core & Multi-Tier Gates
//! Enforces #![deny(unsafe_code)], Gates 01-06 physical and mathematical sanity verification,
//! and Gate 07 two-tier pre-dispatch egress sealing with TOCTOU defense.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod gates;
pub mod toctou_egress;

pub use gates::{GateError, VerificationGates};
pub use toctou_egress::{CertifiedEnvelope, EgressGuard, SCS_SEAL_CANARY};
