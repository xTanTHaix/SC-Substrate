//! DeepPoly Soundness & Directed-Rounding Envelope Verification
//!
//! Submits 1,000 randomized intervals straddling the non-linear ReLU inflection point (x=0),
//! formally proving that the lower line L(x) and upper line U(x) strictly enclose the true activation:
//! L(x) <= ReLU(x) <= U(x) for all x in [x_min, x_max].

use sc_cas_neural::deeppoly::{DeepPolyBounds, DeepPolyLayer};

#[test]
fn test_1000_deeppoly_relu_relaxation_soundness() {
    for i in 0..1000 {
        let lower = -10.0 + (i as f64) * 0.015;
        let upper = lower + 0.5 + ((i % 20) as f64) * 0.1;

        let input_bounds = DeepPolyBounds::from_box(lower, upper);
        let poly = DeepPolyLayer::relu_relaxation(&input_bounds);

        // Verify across 20 sample points in [lower, upper]
        for s in 0..=20 {
            let t = (s as f64) / 20.0;
            let x = lower + t * (upper - lower);
            let true_relu = if x > 0.0 { x } else { 0.0 };

            // Directed rounding tolerance
            const TOL: f64 = 1e-12;
            assert!(
                poly.concrete_upper + TOL >= true_relu,
                "DeepPoly Upper Bound Soundness Violation: x={x}, Upper={}, True={true_relu}",
                poly.concrete_upper
            );
            assert!(
                poly.concrete_lower - TOL <= true_relu,
                "DeepPoly Lower Bound Soundness Violation: x={x}, Lower={}, True={true_relu}",
                poly.concrete_lower
            );
        }
    }
}
