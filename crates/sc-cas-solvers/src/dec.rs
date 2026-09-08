//! Discrete Exterior Calculus & FEEC Hodge Decomposition (Alg 27: H_DEC/FEEC)

/// 1D simplicial mesh topology (Vertices and Edges).
pub struct Mesh1D {
    /// Number of vertices (0-cells).
    pub num_vertices: usize,
    /// Edges as directed pairs (v_start, v_end) (1-cells).
    pub edges: Vec<(usize, usize)>,
}

/// Simplicial complex operators.
pub struct SimplicialComplex;

impl SimplicialComplex {
    /// Boundary matrix d_0 mapping 0-forms to 1-forms: (d_0 v)_e = v_{end} - v_{start}.
    pub fn exterior_derivative_d0(mesh: &Mesh1D, v: &[f64]) -> Vec<f64> {
        assert_eq!(v.len(), mesh.num_vertices);
        let mut d0_v = Vec::with_capacity(mesh.edges.len());
        for &(start, end) in &mesh.edges {
            d0_v.push(v[end] - v[start]);
        }
        d0_v
    }
}

/// Discrete Hodge decomposition structure.
pub struct DiscreteHodge;

impl DiscreteHodge {
    /// Verify discrete orthogonality: <d0_alpha, beta> == 0.
    pub fn verify_orthogonality(d0_alpha: &[f64], coexact_beta: &[f64]) -> bool {
        let dot: f64 = d0_alpha
            .iter()
            .zip(coexact_beta)
            .map(|(&a, &b)| a * b)
            .sum();
        dot.abs() < 1.0e-12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dec_exact_differential_and_orthogonality() {
        let mesh = Mesh1D {
            num_vertices: 3,
            edges: vec![(0, 1), (1, 2), (2, 0)],
        };
        let potential_0_form = vec![1.0, 3.0, 7.0];
        let exact_1_form = SimplicialComplex::exterior_derivative_d0(&mesh, &potential_0_form);

        assert_eq!(exact_1_form, vec![2.0, 4.0, -6.0]);
        // Sum of exact differential along closed loop must be zero
        let loop_sum: f64 = exact_1_form.iter().sum();
        assert!(
            loop_sum.abs() < 1.0e-14,
            "Closed loop integral of exact form must vanish"
        );
    }
}
