//! Layer 01: Micro-Architecture Topology & Hardware Feature Detection

/// Hardware features supported across modern co-design backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchFeature {
    /// Intel AVX-512 Foundation instructions.
    Avx512F,
    /// Intel AVX-512 Vector Neural Network Instructions.
    Avx512Vnni,
    /// Intel Advanced Matrix Extensions (AMX) TMUL tile matrix multiplication.
    AmxTile,
    /// Intel AMX BF16 tile operations.
    AmxBf16,
    /// ARM Scalable Matrix Extension 2 (SME2).
    ArmSme2,
    /// RISC-V Vector Extension 1.0.
    RiscvRvv,
}

/// Host CPU/Co-processor topology state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareTopology {
    features: Vec<ArchFeature>,
}

impl HardwareTopology {
    /// Detect host CPU capabilities safely.
    pub fn detect() -> Self {
        #[allow(unused_mut)]
        let mut features = Vec::new();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                features.push(ArchFeature::Avx512F);
            }
            if is_x86_feature_detected!("avx512vnni") {
                features.push(ArchFeature::Avx512Vnni);
            }
        }

        Self { features }
    }

    /// Check if target feature is present on current host execution context.
    #[inline]
    pub fn has_feature(&self, feature: ArchFeature) -> bool {
        self.features.contains(&feature)
    }

    /// Total count of detected hardware acceleration extensions.
    #[inline]
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }
}

impl Default for HardwareTopology {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_detect_safe() {
        let topo = HardwareTopology::detect();
        // Detection must not panic and must return valid feature query
        let _ = topo.has_feature(ArchFeature::Avx512F);
    }
}
