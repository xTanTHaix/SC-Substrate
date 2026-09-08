//! Core Sovereign Infrastructure: Layers 00, 01, 02
//! Enforces #![deny(unsafe_code)], Lean 4 proof AST emission,
//! NIST FIPS 204 ML-DSA-65 post-quantum signing, CPUID topology detection,
//! and low-discrepancy Halton sequences.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod halton;
pub mod lean4;
pub mod pqc;
pub mod topology;

pub use halton::{HaltonError, HaltonSequence};
pub use lean4::{LeanProofCertificate, LeanStep, LeanTheorem};
pub use pqc::{ml_dsa_sign, ml_dsa_verify, KeyPair, Signature};
pub use topology::{ArchFeature, HardwareTopology};
