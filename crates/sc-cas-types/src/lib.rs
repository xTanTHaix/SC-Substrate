//! Layer 03 & 04: Rigorous Ball Arithmetic & Geometric Clifford Multivectors
//! Enforces #![deny(unsafe_code)], Arb-style ball arithmetic [m +/- r],
//! cascading precision promotion, and 16-bit blade bitmask Clifford algebra.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod ball;
pub mod cascade;
pub mod clifford;

pub use ball::{Ball, BallError};
pub use cascade::{AdaptiveBall, PrecisionTier};
pub use clifford::{CliffordSignature, Multivector};
