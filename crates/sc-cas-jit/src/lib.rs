//! Layer 10: Bare-Metal JIT Engine & Relocation Trampoline Pool
//! Quarantined unsafe memory management: JIT stencil memory allocation,
//! RSP 16-byte stack alignment, RISC-V RVV 1.0 code generation, and GPU SPIR-V/WGSL compute shaders.

#![warn(missing_docs)]

pub mod gpu_shader;
pub mod rvv_jit;
pub mod trampoline;

pub use gpu_shader::{GpuBackend, GpuShaderCompiler};
pub use rvv_jit::{RvvInstruction, RvvJitGenerator};
pub use trampoline::{JitMemoryPool, TrampolineError};
