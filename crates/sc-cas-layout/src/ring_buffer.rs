//! Modular ring-buffer index arithmetic with formal wrap-around correctness.
//!
//! A `RingBuffer` of capacity `N` (must be a power of two) maps any logical
//! index via bitwise AND masking: `physical = logical & (N - 1)`.  This is
//! equivalent to `logical % N` but branch-free and constant-time.
//!
//! The power-of-two constraint is validated at construction time; once
//! constructed all index operations are infallible.

use crate::error::LayoutError;

/// A power-of-two ring-buffer descriptor.
///
/// Holds the capacity and the precomputed bitmask `capacity - 1` so that all
/// index operations are a single AND instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingBuffer {
    capacity: u64,
    /// Bitmask = capacity - 1; valid because capacity is a power of two.
    mask: u64,
}

impl RingBuffer {
    /// Construct a ring buffer of the given capacity.
    ///
    /// # Errors
    /// Returns [`LayoutError::ZeroCapacity`] for zero, or
    /// [`LayoutError::CapacityNotPowerOfTwo`] when `capacity` is not a power of two.
    pub fn new(capacity: u64) -> Result<Self, LayoutError> {
        if capacity == 0 {
            return Err(LayoutError::ZeroCapacity);
        }
        if !capacity.is_power_of_two() {
            return Err(LayoutError::CapacityNotPowerOfTwo(capacity));
        }
        Ok(Self { capacity, mask: capacity - 1 })
    }

    /// Map a logical index to its physical slot via bitmask modular reduction.
    ///
    /// Correct for any `u64` logical index; never panics.
    #[inline]
    pub fn slot(&self, logical: u64) -> u64 {
        logical & self.mask
    }

    /// Advance a head or tail pointer by `delta` slots, wrapping correctly.
    ///
    /// Equivalent to `(pointer + delta) % capacity` but branch-free.
    #[inline]
    pub fn advance(&self, pointer: u64, delta: u64) -> u64 {
        (pointer + delta) & self.mask
    }

    /// Number of occupied slots between `head` and `tail` (tail is exclusive).
    ///
    /// Returns the correct count even when `tail` has wrapped around `head`.
    #[inline]
    pub fn occupied(&self, head: u64, tail: u64) -> u64 {
        tail.wrapping_sub(head) & self.mask
    }

    /// Number of free slots given current `head` and `tail`.
    #[inline]
    pub fn free(&self, head: u64, tail: u64) -> u64 {
        self.capacity - self.occupied(head, tail)
    }

    /// Declared capacity of the ring.
    #[inline]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Bitmask precomputed from `capacity - 1`.
    #[inline]
    pub fn mask(&self) -> u64 {
        self.mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_wraps_correctly() {
        let rb = RingBuffer::new(16).unwrap();
        assert_eq!(rb.slot(0), 0);
        assert_eq!(rb.slot(15), 15);
        // Wrap: index 16 → slot 0.
        assert_eq!(rb.slot(16), 0);
        assert_eq!(rb.slot(31), 15);
        assert_eq!(rb.slot(32), 0);
        // Large logical index wraps correctly.
        assert_eq!(rb.slot(u64::MAX), rb.slot(u64::MAX % 16));
    }

    #[test]
    fn test_advance_wraps() {
        let rb = RingBuffer::new(8).unwrap();
        // Starting at 6, advance by 4: (6+4) % 8 = 2.
        assert_eq!(rb.advance(6, 4), 2);
        assert_eq!(rb.advance(0, 8), 0);
    }

    #[test]
    fn test_occupied_and_free() {
        let rb = RingBuffer::new(8).unwrap();
        // Empty: head == tail.
        assert_eq!(rb.occupied(3, 3), 0);
        assert_eq!(rb.free(3, 3), 8);
        // 3 elements: head=1, tail=4.
        assert_eq!(rb.occupied(1, 4), 3);
        assert_eq!(rb.free(1, 4), 5);
        // Wrapped: head=6, tail=2 → 4 elements (6→7→0→1→2).
        assert_eq!(rb.occupied(6, 2), 4);
        assert_eq!(rb.free(6, 2), 4);
    }

    #[test]
    fn test_non_power_of_two_rejected() {
        assert!(matches!(
            RingBuffer::new(3),
            Err(LayoutError::CapacityNotPowerOfTwo(3))
        ));
        assert!(matches!(
            RingBuffer::new(100),
            Err(LayoutError::CapacityNotPowerOfTwo(100))
        ));
    }

    #[test]
    fn test_zero_capacity_rejected() {
        assert!(matches!(RingBuffer::new(0), Err(LayoutError::ZeroCapacity)));
    }

    #[test]
    fn test_power_of_two_capacities_accepted() {
        for &cap in &[1u64, 2, 4, 8, 16, 1024, 1 << 20] {
            assert!(RingBuffer::new(cap).is_ok(), "capacity {cap} should be accepted");
        }
    }
}
