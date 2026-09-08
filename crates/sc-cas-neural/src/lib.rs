//! Layer 09: Neural Enclosure & Certified Robustness
//! Enforces #![deny(unsafe_code)], inflection-aware DeepPoly abstract interpretation,
//! directed floating-point rounding, and Intel AMX / ARM SME2 batch polytope acceleration.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod amx_poly;
pub mod deeppoly;

pub use amx_poly::{AmxTileConfig, BatchedAmxPoly};
pub use deeppoly::{DeepPolyBounds, DeepPolyLayer, Polytope};
