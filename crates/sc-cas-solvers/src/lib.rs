//! Layer 07: Structure-Preserving Solvers
//! Enforces #![deny(unsafe_code)], 6th-order symplectic Hamiltonian integrator,
//! DAE manifold projection, open quantum Lindblad CPTP solver,
//! discrete exterior calculus Hodge decomposition, and Clifford+T synthesis.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod dae;
pub mod dec;
pub mod lindblad;
pub mod quantum_synth;
pub mod symplectic;

pub use dae::{DaeError, DaeManifoldSolver};
pub use dec::{DiscreteHodge, Mesh1D, SimplicialComplex};
pub use lindblad::{DensityMatrix, LindbladError, LindbladSolver};
pub use quantum_synth::{CliffordTGate, QuantumSynth};
pub use symplectic::{HamiltonianSystem, Symplectic6Integrator};
