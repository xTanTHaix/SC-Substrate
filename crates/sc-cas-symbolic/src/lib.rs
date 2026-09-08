//! Layer 05: Exact Symbolic Core & Polynomial Rings
//! Enforces #![deny(unsafe_code)], Dixon p-adic linear solver,
//! F5 signature Gröbner bases, Riemann multi-sheet continuation,
//! and Lie point symmetry reductions.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod dixon;
pub mod expr;
pub mod groebner;
pub mod lie;
pub mod riemann;

pub use dixon::{DixonError, DixonSolver, Rational};
pub use expr::{Expr, ExprError};
pub use groebner::{GroebnerBasis, Monomial, Polynomial};
pub use lie::{InfinitesimalGenerator, LieSymmetry};
pub use riemann::{BranchCut, RiemannContinuation, RiemannSheet};
