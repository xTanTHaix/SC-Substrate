//! ALT Longevity 50,000-Cycle Zero-Leak Verification
//!
//! Simulates 50,000 allocations, executions, and deallocations in linear memory,
//! strictly proving that the memory bump pointer resets without drift (leak slope k = 0).

#[test]
fn test_50k_alt_longevity_zero_leak() {
    use sc_cas_wasm::{with_arena_mut, LinearArena, DEFAULT_ARENA_CAPACITY};

    let mut arena = LinearArena::new(DEFAULT_ARENA_CAPACITY);
    let initial_offset = arena.current_offset();

    for cycle in 1..=50_000 {
        let p = arena.alloc(256).unwrap();
        arena.write_slice(p, b"CYCLE_STRESS_PAYLOAD").unwrap();
        let _ = arena.read_slice(p, 20).unwrap();
        arena.free(p, 256).unwrap();

        if cycle % 10_000 == 0 {
            assert_eq!(arena.current_offset(), initial_offset, "Memory drift detected at cycle {cycle}");
        }
    }

    assert_eq!(arena.current_offset(), initial_offset);
}
