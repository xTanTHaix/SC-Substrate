//! Error types for `sc-cas-layout`.

/// All failure modes surfaced by layout computations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// Field alignment must be a non-zero power of two.
    InvalidAlignment(u64),
    /// Buffer capacity must be a non-zero power of two for modular indexing.
    CapacityNotPowerOfTwo(u64),
    /// Buffer capacity must be > 0.
    ZeroCapacity,
    /// The given `(slope, intercept, domain_size)` combination would cause an
    /// out-of-bounds access: `f(i) = slope * i + intercept` escapes `[0, N-1]`.
    IndexOutOfBounds {
        /// Maximum index value produced by the expression.
        max_index: i128,
        /// Declared buffer size N.
        buffer_size: u64,
    },
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAlignment(a) => write!(f, "Alignment {a} is not a power-of-two > 0"),
            Self::CapacityNotPowerOfTwo(c) => {
                write!(f, "Ring buffer capacity {c} must be a power of two")
            }
            Self::ZeroCapacity => write!(f, "Buffer capacity must be > 0"),
            Self::IndexOutOfBounds { max_index, buffer_size } => write!(
                f,
                "Index expression produces max={max_index} which exceeds N-1={}",
                buffer_size - 1
            ),
        }
    }
}

impl std::error::Error for LayoutError {}
