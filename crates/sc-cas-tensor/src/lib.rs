//! Layer 08: High-Performance Linear Algebra & Tensor Core
//! Enforces #![deny(unsafe_code)], strided tensor slicing with negative steps,
//! masked soft-impute SVD, and tree-width min-FLOPs tensor contraction path optimization.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod einsum;
pub mod slice;
pub mod soft_impute;

pub use einsum::{ContractionNode, ContractionPathOptimizer};
pub use slice::{StridedSlice, TensorView};
pub use soft_impute::SoftImputeSvd;
