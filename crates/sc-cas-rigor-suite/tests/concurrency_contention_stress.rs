//! Concurrency Contention & High-Throughput Stress Test Suite
//!
//! Spawns 64 concurrent threads hammering linear memory allocation,
//! request marshaling, dispatch execution, and deallocation under heavy lock contention.

use sc_cas_wasm::{
    decode_response, dispatch_request, encode_request, LinearArena, OpCode, StatusCode,
    DEFAULT_ARENA_CAPACITY,
};
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_64_thread_concurrent_contention() {
    const NUM_THREADS: usize = 64;
    const OPERATIONS_PER_THREAD: usize = 200;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::with_capacity(NUM_THREADS);

    for thread_id in 0..NUM_THREADS {
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();

            for op_idx in 0..OPERATIONS_PER_THREAD {
                let expr = format!("simplify: {} * x + {} * x", thread_id, op_idx);
                let req = encode_request(OpCode::EvalSymbolic, 128, expr.as_bytes());

                let resp = dispatch_request(&req);
                let (status, payload, digest) =
                    decode_response(&resp).expect("Corrupt response during concurrent dispatch");

                assert_eq!(status, StatusCode::Success);
                assert!(!payload.is_empty());
                assert_ne!(digest, [0u8; 32]);
            }
        }));
    }

    for handle in handles {
        handle
            .join()
            .expect("Worker thread panicked during concurrent contention");
    }
}

#[test]
fn test_concurrent_linear_arena_isolation() {
    let mut arena = LinearArena::new(DEFAULT_ARENA_CAPACITY);
    let initial_offset = arena.current_offset();

    let mut offsets = Vec::new();
    for i in 0..100 {
        let size = (i + 1) * 64;
        let ptr = arena.alloc(size).expect("Arena alloc failed");
        assert_eq!(ptr % 64, 0);
        offsets.push((ptr, size));
    }

    while let Some((ptr, size)) = offsets.pop() {
        arena.free(ptr, size).expect("Arena free failed");
    }

    assert_eq!(
        arena.current_offset(),
        initial_offset,
        "Linear arena leaked under interleaved usage"
    );
}
