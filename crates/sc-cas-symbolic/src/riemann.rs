//! Layer 05: Multi-Sheet Riemann Analytic Continuation (Alg 17: Theta_Riemann-Cont)

/// Branch cut curve in the complex plane.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchCut {
    /// Branch point origin (re, im).
    pub origin: (f64, f64),
    /// Branch angle orientation in radians.
    pub angle: f64,
    /// Identifier of target sheet accessed when crossed clockwise.
    pub sheet_jump: i32,
}

/// Sheet representation of multi-valued Riemann surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiemannSheet {
    /// Sheet index (..., -1, 0, 1, ...).
    pub sheet_index: i32,
}

/// Analytic continuation manager tracking path traversal across branch cuts.
pub struct RiemannContinuation {
    cuts: Vec<BranchCut>,
}

impl RiemannContinuation {
    /// Create continuation engine with registered branch cuts.
    pub fn new(cuts: Vec<BranchCut>) -> Self {
        Self { cuts }
    }

    /// Transport point (z_re, z_im) along step (dz_re, dz_im) and update sheet index.
    pub fn transport_step(
        &self,
        current_pos: (f64, f64),
        step: (f64, f64),
        sheet: &mut RiemannSheet,
    ) {
        let next_pos = (current_pos.0 + step.0, current_pos.1 + step.1);

        for cut in &self.cuts {
            // Check if line segment crosses cut ray
            let _dx = next_pos.0 - current_pos.0;
            let dy = next_pos.1 - current_pos.1;
            if dy.abs() > 1.0e-12 && current_pos.1 <= cut.origin.1 && next_pos.1 > cut.origin.1 {
                // Crossing from below
                sheet.sheet_index += cut.sheet_jump;
            } else if dy.abs() > 1.0e-12
                && current_pos.1 >= cut.origin.1
                && next_pos.1 < cut.origin.1
            {
                // Crossing from above
                sheet.sheet_index -= cut.sheet_jump;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_cut_sheet_tracking() {
        let cut = BranchCut {
            origin: (0.0, 0.0),
            angle: 0.0,
            sheet_jump: 1,
        };
        let cont = RiemannContinuation::new(vec![cut]);
        let mut sheet = RiemannSheet { sheet_index: 0 };

        // Step across real axis branch cut from y = -0.1 to y = +0.1
        cont.transport_step((1.0, -0.1), (0.0, 0.2), &mut sheet);
        assert_eq!(
            sheet.sheet_index, 1,
            "Must promote to sheet 1 upon crossing"
        );

        // Step back
        cont.transport_step((1.0, 0.1), (0.0, -0.2), &mut sheet);
        assert_eq!(
            sheet.sheet_index, 0,
            "Must return to sheet 0 upon reverse crossing"
        );
    }
}
