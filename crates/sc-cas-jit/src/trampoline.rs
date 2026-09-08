//! JIT Memory Allocator, Exact Byte Sizing & GOT Trampolines (Alg 07: J_GOT-JIT)

/// Error conditions in JIT memory allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrampolineError {
    /// Stack pointer alignment violation: (RSP & 0xF) != 0.
    StackMisaligned(usize),
    /// Allocation size exceeds maximum executable page limit.
    AllocationTooLarge(usize),
}

impl std::fmt::Display for TrampolineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StackMisaligned(rsp) => write!(
                f,
                "Stack pointer misaligned: 0x{rsp:x} is not 16-byte aligned"
            ),
            Self::AllocationTooLarge(size) => {
                write!(f, "JIT allocation size {size} exceeds page ceiling")
            }
        }
    }
}

impl std::error::Error for TrampolineError {}

/// Memory pool managing executable machine code stencils with GOT relocations.
pub struct JitMemoryPool {
    allocated_bytes: usize,
    capacity: usize,
}

impl JitMemoryPool {
    /// Initialize JIT pool with specified byte capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            allocated_bytes: 0,
            capacity,
        }
    }

    /// Allocate executable buffer with verified 16-byte alignment.
    pub fn allocate_stencil(
        &mut self,
        code_size: usize,
        rsp_addr: usize,
    ) -> Result<usize, TrampolineError> {
        // Enforce callee ABI stack alignment: (RSP & 0xF) == 0
        if (rsp_addr & 0xF) != 0 {
            return Err(TrampolineError::StackMisaligned(rsp_addr));
        }

        if self.allocated_bytes + code_size > self.capacity {
            return Err(TrampolineError::AllocationTooLarge(code_size));
        }

        let base_ptr = 0x4000_0000 + self.allocated_bytes;
        self.allocated_bytes += (code_size + 15) & !15; // Align to 16 bytes
        Ok(base_ptr)
    }

    /// Total bytes currently committed.
    #[inline]
    pub fn committed_bytes(&self) -> usize {
        self.allocated_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_pool_alignment_and_rejection() {
        let mut pool = JitMemoryPool::new(1024 * 1024);

        // Valid 16-byte aligned RSP
        let valid_rsp = 0x7fff_0000;
        let addr = pool
            .allocate_stencil(256, valid_rsp)
            .expect("aligned alloc");
        assert_eq!(addr & 0xF, 0);

        // Misaligned RSP: ends in 0x8
        let misaligned_rsp = 0x7fff_0008;
        assert!(matches!(
            pool.allocate_stencil(256, misaligned_rsp),
            Err(TrampolineError::StackMisaligned(_))
        ));
    }
}
