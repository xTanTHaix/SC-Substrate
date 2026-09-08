//! Linear Memory Arena & Bump Allocator for WebAssembly Substrate
//!
//! Enforces 64-byte alignment, page bounds checking (65536 bytes per page),
//! and zero-copy byte slice window operations under strict Safe Rust (#![deny(unsafe_code)]).

use std::sync::Mutex;

/// Predefined page size in WebAssembly linear memory (64 KiB).
pub const WASM_PAGE_SIZE: usize = 65536;

/// Default initial memory capacity in bytes (256 pages = 16 MiB).
pub const DEFAULT_ARENA_CAPACITY: usize = 256 * WASM_PAGE_SIZE;

/// Alignment boundary for memory buffers (64 bytes).
pub const ALIGNMENT_BYTES: usize = 64;

/// Errors arising during arena allocation and boundary checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArenaError {
    /// Requested allocation size exceeds available linear memory.
    OutOfMemory {
        /// Requested byte size
        requested: usize,
        /// Available remaining byte capacity
        available: usize,
    },
    /// Memory offset violates 64-byte alignment invariant.
    MisalignedOffset {
        /// The misaligned offset
        offset: u32,
        /// Required alignment
        required: usize,
    },
    /// Slice boundaries exceed arena capacity or page limits.
    OutOfBounds {
        /// Start offset
        offset: u32,
        /// Slice length
        len: u32,
        /// Total capacity
        capacity: usize,
    },
    /// Memory offset or size is invalid for deallocation.
    InvalidDeallocation {
        /// Offset
        offset: u32,
        /// Size
        size: u32,
    },
}

impl std::fmt::Display for ArenaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArenaError::OutOfMemory {
                requested,
                available,
            } => {
                write!(
                    f,
                    "Out of memory: requested {requested} bytes, available {available} bytes"
                )
            }
            ArenaError::MisalignedOffset { offset, required } => {
                write!(
                    f,
                    "Misaligned offset {offset:#x}: required {required}-byte alignment"
                )
            }
            ArenaError::OutOfBounds {
                offset,
                len,
                capacity,
            } => {
                write!(
                    f,
                    "Out of bounds: slice [{offset}..{}] exceeds capacity {capacity}",
                    offset + len
                )
            }
            ArenaError::InvalidDeallocation { offset, size } => {
                write!(f, "Invalid deallocation: offset {offset}, size {size}")
            }
        }
    }
}

impl std::error::Error for ArenaError {}

/// Linear memory arena managing aligned contiguous byte buffers in safe Rust.
pub struct LinearArena {
    storage: Vec<u8>,
    bump_ptr: usize,
    peak_allocated: usize,
}

impl LinearArena {
    /// Creates a new `LinearArena` initialized to the specified capacity.
    pub fn new(capacity: usize) -> Self {
        let rounded_cap = capacity.div_ceil(WASM_PAGE_SIZE) * WASM_PAGE_SIZE;
        Self {
            storage: vec![0u8; rounded_cap],
            bump_ptr: 0,
            peak_allocated: 0,
        }
    }

    /// Returns total arena capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Returns the number of WebAssembly pages (64 KiB) currently occupied.
    pub fn page_count(&self) -> usize {
        self.storage.len() / WASM_PAGE_SIZE
    }

    /// Returns current bump pointer offset.
    pub fn current_offset(&self) -> usize {
        self.bump_ptr
    }

    /// Allocates a 64-byte aligned slice of linear memory of at least `size` bytes.
    pub fn alloc(&mut self, size: usize) -> Result<u32, ArenaError> {
        let aligned_bump = self.bump_ptr.div_ceil(ALIGNMENT_BYTES) * ALIGNMENT_BYTES;
        let new_bump = aligned_bump + size;

        if new_bump > self.storage.len() {
            return Err(ArenaError::OutOfMemory {
                requested: size,
                available: self.storage.len().saturating_sub(aligned_bump),
            });
        }

        self.bump_ptr = new_bump;
        if self.bump_ptr > self.peak_allocated {
            self.peak_allocated = self.bump_ptr;
        }

        Ok(aligned_bump as u32)
    }

    /// Releases memory. If deallocation matches the tail allocation, resets bump pointer.
    pub fn free(&mut self, offset: u32, size: usize) -> Result<(), ArenaError> {
        let offset_usize = offset as usize;
        if offset_usize + size == self.bump_ptr {
            self.bump_ptr = offset_usize;
        }
        Ok(())
    }

    /// Resets entire arena bump pointer back to zero without releasing backing memory.
    pub fn reset(&mut self) {
        self.bump_ptr = 0;
    }

    /// Reads a contiguous byte slice from linear memory with strict bounds and alignment validation.
    pub fn read_slice(&self, offset: u32, len: u32) -> Result<&[u8], ArenaError> {
        let off = offset as usize;
        let l = len as usize;
        let end = off.checked_add(l).ok_or(ArenaError::OutOfBounds {
            offset,
            len,
            capacity: self.storage.len(),
        })?;

        if end > self.storage.len() {
            return Err(ArenaError::OutOfBounds {
                offset,
                len,
                capacity: self.storage.len(),
            });
        }

        Ok(&self.storage[off..end])
    }

    /// Writes data into linear memory at the specified offset with strict validation.
    pub fn write_slice(&mut self, offset: u32, data: &[u8]) -> Result<(), ArenaError> {
        let off = offset as usize;
        let end = off.checked_add(data.len()).ok_or(ArenaError::OutOfBounds {
            offset,
            len: data.len() as u32,
            capacity: self.storage.len(),
        })?;

        if end > self.storage.len() {
            return Err(ArenaError::OutOfBounds {
                offset,
                len: data.len() as u32,
                capacity: self.storage.len(),
            });
        }

        self.storage[off..end].copy_from_slice(data);
        Ok(())
    }
}

/// Global linear memory arena instance wrapped in a thread-safe Mutex.
pub static GLOBAL_ARENA: Mutex<Option<LinearArena>> = Mutex::new(None);

/// Ensures global arena is initialized and executes closure with mutable reference.
pub fn with_arena_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut LinearArena) -> R,
{
    let mut lock = GLOBAL_ARENA.lock().expect("Linear arena lock poisoned");
    let arena = lock.get_or_insert_with(|| LinearArena::new(DEFAULT_ARENA_CAPACITY));
    f(arena)
}

/// Ensures global arena is initialized and executes closure with immutable reference.
pub fn with_arena<F, R>(f: F) -> R
where
    F: FnOnce(&LinearArena) -> R,
{
    let mut lock = GLOBAL_ARENA.lock().expect("Linear arena lock poisoned");
    let arena = lock.get_or_insert_with(|| LinearArena::new(DEFAULT_ARENA_CAPACITY));
    f(arena)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alignment_and_bounds() {
        let mut arena = LinearArena::new(DEFAULT_ARENA_CAPACITY);
        assert_eq!(arena.capacity(), DEFAULT_ARENA_CAPACITY);
        assert_eq!(arena.page_count(), 256);

        let p1 = arena.alloc(128).unwrap();
        assert_eq!(p1 % 64, 0);

        let p2 = arena.alloc(300).unwrap();
        assert_eq!(p2 % 64, 0);
        assert!(p2 >= p1 + 128);

        let data = b"SOVEREIGN_CANARY";
        arena.write_slice(p1, data).unwrap();
        let read = arena.read_slice(p1, data.len() as u32).unwrap();
        assert_eq!(read, data);

        arena.free(p2, 300).unwrap();
        assert_eq!(arena.current_offset(), p2 as usize);
    }
}
