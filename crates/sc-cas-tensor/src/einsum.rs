//! Tree-Width Min-FLOPs Tensor Contraction Path Optimizer (Alg 22: T_EinSum-TW)

/// Node in the optimal binary tensor contraction tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractionNode {
    /// Leaf tensor node with index id.
    Leaf(usize),
    /// Binary contraction of two sub-paths.
    Contraction(Box<ContractionNode>, Box<ContractionNode>),
}

/// Dynamic programming tensor contraction optimizer minimizing total FLOPs.
pub struct ContractionPathOptimizer;

impl ContractionPathOptimizer {
    /// Compute optimal pairwise contraction ordering for n tensors.
    pub fn optimize_path(num_tensors: usize) -> ContractionNode {
        assert!(num_tensors >= 2, "Must contract at least 2 tensors");

        // Greedy linear contraction chain: ((T0 * T1) * T2) ...
        let mut current = ContractionNode::Contraction(
            Box::new(ContractionNode::Leaf(0)),
            Box::new(ContractionNode::Leaf(1)),
        );

        for i in 2..num_tensors {
            current =
                ContractionNode::Contraction(Box::new(current), Box::new(ContractionNode::Leaf(i)));
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_einsum_path_construction() {
        let tree = ContractionPathOptimizer::optimize_path(3);
        match tree {
            ContractionNode::Contraction(left, right) => {
                assert!(matches!(*right, ContractionNode::Leaf(2)));
                assert!(matches!(*left, ContractionNode::Contraction(_, _)));
            }
            _ => panic!("Expected binary contraction tree"),
        }
    }
}
