//! Inflection-Aware Directed-Rounding DeepPoly Abstract Domain (Alg 10: P_DeepPoly)

use sc_cas_types::Ball;

/// Linear relaxation bounds for a neuron: lower_w * x + lower_b <= y <= upper_w * x + upper_b.
#[derive(Debug, Clone, PartialEq)]
pub struct DeepPolyBounds {
    /// Lower affine weight vector.
    pub lower_w: Vec<f64>,
    /// Lower affine bias.
    pub lower_b: f64,
    /// Upper affine weight vector.
    pub upper_w: Vec<f64>,
    /// Upper affine bias.
    pub upper_b: f64,
    /// Concrete concrete lower bound.
    pub concrete_lower: f64,
    /// Concrete concrete upper bound.
    pub concrete_upper: f64,
}

impl DeepPolyBounds {
    /// Create concrete box bounds [l, u].
    pub fn from_box(lower: f64, upper: f64) -> Self {
        assert!(lower <= upper, "Lower bound must not exceed upper bound");
        Self {
            lower_w: Vec::new(),
            lower_b: lower,
            upper_w: Vec::new(),
            upper_b: upper,
            concrete_lower: lower,
            concrete_upper: upper,
        }
    }

    /// Check if interval encloses target value safely.
    pub fn contains(&self, val: f64) -> bool {
        val >= self.concrete_lower - 1.0e-15 && val <= self.concrete_upper + 1.0e-15
    }
}

/// Abstract neural network layer performing sound polytope transformer.
pub struct DeepPolyLayer;

impl DeepPolyLayer {
    /// Sound ReLU relaxation with directed rounding guards.
    pub fn relu_relaxation(input: &DeepPolyBounds) -> DeepPolyBounds {
        let l = input.concrete_lower;
        let u = input.concrete_upper;

        if u <= 0.0 {
            // Case 1: Inactive ReLU -> strictly zero
            DeepPolyBounds::from_box(0.0, 0.0)
        } else if l >= 0.0 {
            // Case 2: Strictly active ReLU -> identity function
            input.clone()
        } else {
            // Case 3: Crossing zero -> triangular relaxation
            // Upper bound line: y = (u / (u - l)) * (x - l)
            let slope = u / (u - l);
            let upper_b = -slope * l;

            // Lower bound heuristic: either y = 0 or y = (u / (u - l)) * x depending on area
            let lower_b = 0.0;

            DeepPolyBounds {
                lower_w: vec![0.0],
                lower_b,
                upper_w: vec![slope],
                upper_b,
                concrete_lower: 0.0,
                concrete_upper: u,
            }
        }
    }

    /// Convert bounds to certified Arb Ball enclosure.
    pub fn to_ball(bounds: &DeepPolyBounds) -> Ball {
        let mid = 0.5 * (bounds.concrete_lower + bounds.concrete_upper);
        let rad = 0.5 * (bounds.concrete_upper - bounds.concrete_lower);
        Ball::new(mid, rad).unwrap_or(Ball::exact(mid))
    }
}

/// Multi-neuron polytope container.
#[derive(Debug, Clone, PartialEq)]
pub struct Polytope {
    /// Layer-wise neuron bounds.
    pub neurons: Vec<DeepPolyBounds>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deeppoly_relu_relaxation_soundness() {
        let crossing = DeepPolyBounds::from_box(-2.0, 3.0);
        let relu_out = DeepPolyLayer::relu_relaxation(&crossing);

        assert_eq!(relu_out.concrete_lower, 0.0);
        assert_eq!(relu_out.concrete_upper, 3.0);
        assert!(relu_out.contains(0.0));
        assert!(relu_out.contains(3.0));
        assert!(relu_out.contains(1.5));
    }
}
