//! Strided Tensor Slicing & Offset Engine (Alg 06: S_Tensor-Slice)

/// Slice specification along a single dimension: start:stop:step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StridedSlice {
    /// Slice start index.
    pub start: i64,
    /// Slice stop index (exclusive).
    pub stop: i64,
    /// Stride step size (can be negative).
    pub step: i64,
}

impl StridedSlice {
    /// Full dimension slice [::1].
    pub fn all(dim_len: usize) -> Self {
        Self {
            start: 0,
            stop: dim_len as i64,
            step: 1,
        }
    }
}

/// Zero-copy non-contiguous tensor view.
#[derive(Debug, Clone, PartialEq)]
pub struct TensorView<'a> {
    /// Reference to contiguous backing storage buffer.
    pub buffer: &'a [f64],
    /// Shape dimensions (d_0, d_1, ..., d_{k-1}).
    pub shape: Vec<usize>,
    /// Strides along each dimension in element units.
    pub strides: Vec<i64>,
    /// Base element offset within backing buffer.
    pub base_offset: usize,
}

impl<'a> TensorView<'a> {
    /// Create a contiguous view over backing buffer.
    pub fn from_contiguous(buffer: &'a [f64], shape: Vec<usize>) -> Self {
        let mut strides = vec![0i64; shape.len()];
        let mut stride = 1i64;
        for i in (0..shape.len()).rev() {
            strides[i] = stride;
            stride *= shape[i] as i64;
        }
        Self {
            buffer,
            shape,
            strides,
            base_offset: 0,
        }
    }

    /// Read element at multidimensional coordinate.
    pub fn get(&self, coords: &[usize]) -> f64 {
        assert_eq!(coords.len(), self.shape.len());
        let mut offset = self.base_offset as i64;
        for (i, &coord) in coords.iter().enumerate() {
            assert!(coord < self.shape[i]);
            offset += (coord as i64) * self.strides[i];
        }
        self.buffer[offset as usize]
    }

    /// Compute canonical Blake3 hash of the view without allocating temporary buffers.
    pub fn canonical_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        let total_elements = self.shape.iter().product::<usize>();

        // 1D / flat traversal
        let mut coords = vec![0usize; self.shape.len()];
        for _ in 0..total_elements {
            let val = self.get(&coords);
            hasher.update(&val.to_le_bytes());

            // Increment coordinates
            for dim in (0..self.shape.len()).rev() {
                coords[dim] += 1;
                if coords[dim] < self.shape[dim] {
                    break;
                }
                coords[dim] = 0;
            }
        }

        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strided_view_and_hash() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let view = TensorView::from_contiguous(&data, vec![2, 3]);

        assert_eq!(view.get(&[0, 0]), 1.0);
        assert_eq!(view.get(&[0, 2]), 3.0);
        assert_eq!(view.get(&[1, 1]), 5.0);

        let h1 = view.canonical_hash();
        let h2 = view.canonical_hash();
        assert_eq!(h1, h2, "Canonical hash must be deterministic");
    }
}
