//! Layer 06: High-Order Numerical & Quadrature Engines
//! Enforces #![deny(unsafe_code)], Tanh-Sinh double-exponential quadrature,
//! Krylov exponential integration, and 5th-Order WENO-Z relativistic hydrodynamics.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod krylov;
pub mod tanh_sinh;
pub mod weno;

pub use krylov::{KrylovIntegrator, Matrix};
pub use tanh_sinh::{QuadratureError, TanhSinhQuad};
pub use weno::{HydroState, WenoError, WenoSolver};
