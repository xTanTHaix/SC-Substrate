//! Struct field packing, padding calculation, and false-sharing analysis.
//!
//! Simulates the Rust/C layout algorithm (scalar layout rule) for a list of
//! named fields.  Outputs per-field offsets, padding bytes inserted before
//! each field, total struct size after tail padding, and whether the struct
//! fits within a single 64-byte CPU cache line (no false-sharing risk).

use crate::error::LayoutError;

/// The CPU cache line width in bytes (industry standard for x86-64 / ARM64).
pub const CACHE_LINE_BYTES: u64 = 64;

/// A single struct field descriptor.
#[derive(Debug, Clone)]
pub struct FieldSpec {
    /// Field name (for diagnostic reports).
    pub name: &'static str,
    /// Size of the field in bytes.
    pub size: u64,
    /// Required alignment in bytes — must be a non-zero power of two.
    pub align: u64,
}

impl FieldSpec {
    /// Construct a [`FieldSpec`], validating alignment.
    ///
    /// # Errors
    /// Returns [`LayoutError::InvalidAlignment`] when `align` is zero or not a
    /// power of two.
    pub fn new(name: &'static str, size: u64, align: u64) -> Result<Self, LayoutError> {
        if align == 0 || !align.is_power_of_two() {
            return Err(LayoutError::InvalidAlignment(align));
        }
        Ok(Self { name, size, align })
    }
}

/// Per-field layout result produced by [`StructLayout::compute`].
#[derive(Debug, Clone)]
pub struct FieldReport {
    /// Field name.
    pub name: &'static str,
    /// Byte offset from the struct's base address.
    pub offset: u64,
    /// Padding bytes inserted before this field to satisfy its alignment.
    pub padding_before: u64,
    /// Field size in bytes.
    pub size: u64,
}

/// Full struct layout report.
#[derive(Debug, Clone)]
pub struct StructReport {
    /// Per-field layout results in declaration order.
    pub fields: Vec<FieldReport>,
    /// Total struct size including tail padding (aligned to max field alignment).
    pub total_size: u64,
    /// Maximum field alignment; determines struct-level alignment.
    pub struct_align: u64,
    /// Total padding bytes wasted (sum of all `padding_before`s + tail padding).
    pub total_padding: u64,
    /// True when the struct fits entirely within one 64-byte cache line.
    pub single_cache_line: bool,
    /// Number of 64-byte cache lines the struct spans.
    pub cache_lines_spanned: u64,
}

/// Computes scalar-layout field packing for an ordered list of [`FieldSpec`]s.
pub struct StructLayout {
    fields: Vec<FieldSpec>,
}

impl StructLayout {
    /// Build a layout analyser from a non-empty field list.
    pub fn new(fields: Vec<FieldSpec>) -> Self {
        Self { fields }
    }

    /// Simulate the Rust/C scalar layout algorithm and return a full report.
    ///
    /// Fields are laid out in declaration order.  Each field is advanced to the
    /// next multiple of its alignment, and the struct's total size is rounded up
    /// to the struct's alignment (max of all field alignments) — identical to how
    /// `#[repr(C)]` structs are packed.
    pub fn compute(&self) -> StructReport {
        let mut cursor: u64 = 0;
        let mut struct_align: u64 = 1;
        let mut field_reports: Vec<FieldReport> = Vec::with_capacity(self.fields.len());

        for field in &self.fields {
            // Advance cursor to the next alignment boundary for this field.
            let misalign = cursor % field.align;
            let padding_before = if misalign == 0 { 0 } else { field.align - misalign };
            let offset = cursor + padding_before;

            field_reports.push(FieldReport {
                name: field.name,
                offset,
                padding_before,
                size: field.size,
            });

            cursor = offset + field.size;
            if field.align > struct_align {
                struct_align = field.align;
            }
        }

        // Tail padding: round total size up to struct alignment.
        let tail_misalign = cursor % struct_align;
        let tail_padding = if tail_misalign == 0 { 0 } else { struct_align - tail_misalign };
        let total_size = cursor + tail_padding;

        let interior_padding: u64 = field_reports.iter().map(|f| f.padding_before).sum();
        let total_padding = interior_padding + tail_padding;

        let cache_lines_spanned = total_size.div_ceil(CACHE_LINE_BYTES);
        let single_cache_line = cache_lines_spanned == 1;

        StructReport {
            fields: field_reports,
            total_size,
            struct_align,
            total_padding,
            single_cache_line,
            cache_lines_spanned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classic_padding_trap() {
        // Unordered: bool(1,1), u64(8,8), u32(4,4) → 1+7pad+8+4+4pad = 24 bytes
        let fields = vec![
            FieldSpec::new("flag", 1, 1).unwrap(),
            FieldSpec::new("count", 8, 8).unwrap(),
            FieldSpec::new("value", 4, 4).unwrap(),
        ];
        let report = StructLayout::new(fields).compute();
        assert_eq!(report.total_size, 24, "unordered layout should be 24 bytes");
        assert_eq!(report.total_padding, 11, "total padding should be 11 bytes");
    }

    #[test]
    fn test_ordered_zero_padding() {
        // Optimally ordered: u64(8,8), u32(4,4), bool(1,1) → 8+4+1+3pad = 16 bytes
        let fields = vec![
            FieldSpec::new("count", 8, 8).unwrap(),
            FieldSpec::new("value", 4, 4).unwrap(),
            FieldSpec::new("flag", 1, 1).unwrap(),
        ];
        let report = StructLayout::new(fields).compute();
        assert_eq!(report.total_size, 16, "optimally ordered layout should be 16 bytes");
        assert_eq!(report.fields[0].padding_before, 0);
        assert_eq!(report.fields[1].padding_before, 0);
        assert_eq!(report.fields[2].padding_before, 0);
    }

    #[test]
    fn test_single_cache_line_detection() {
        // Struct fitting in 64 bytes should flag single_cache_line = true.
        let fields = vec![
            FieldSpec::new("a", 32, 8).unwrap(),
            FieldSpec::new("b", 32, 8).unwrap(),
        ];
        let report = StructLayout::new(fields).compute();
        assert_eq!(report.total_size, 64);
        assert!(report.single_cache_line);
        assert_eq!(report.cache_lines_spanned, 1);
    }

    #[test]
    fn test_false_sharing_two_cache_lines() {
        // 65 bytes → must span 2 cache lines.
        let fields = vec![
            FieldSpec::new("data", 65, 1).unwrap(),
        ];
        let report = StructLayout::new(fields).compute();
        assert_eq!(report.cache_lines_spanned, 2);
        assert!(!report.single_cache_line);
    }

    #[test]
    fn test_invalid_alignment_rejected() {
        assert!(matches!(
            FieldSpec::new("x", 4, 0),
            Err(LayoutError::InvalidAlignment(0))
        ));
        assert!(matches!(
            FieldSpec::new("x", 4, 3),
            Err(LayoutError::InvalidAlignment(3))
        ));
    }
}
