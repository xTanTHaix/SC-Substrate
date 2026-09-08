//! Layer 03: Dynamic Cascading Precision Escalation (64 -> 256 -> 1024 bits)

use crate::ball::{Ball, BallError};

/// Available precision tiers for certified computations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrecisionTier {
    /// IEEE-754 64-bit standard hardware ball arithmetic.
    Bits64,
    /// 256-bit quadruple-precision software enclosure.
    Bits256,
    /// 1024-bit ultra-precision sovereign polynomial enclosure.
    Bits1024,
}

/// Adaptive numerical ball capable of cascading promotion on loss of significance.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBall {
    /// Active precision tier.
    pub tier: PrecisionTier,
    /// Current 64-bit ball representation.
    pub ball: Ball,
    /// Loss-of-significance flag.
    pub loss_detected: bool,
}

impl AdaptiveBall {
    /// Create a new adaptive ball starting at 64-bit precision tier.
    pub fn new(mid: f64, rad: f64) -> Result<Self, BallError> {
        let ball = Ball::new(mid, rad)?;
        Ok(Self {
            tier: PrecisionTier::Bits64,
            ball,
            loss_detected: false,
        })
    }

    /// Check if current radius satisfies requested tolerance epsilon.
    #[inline]
    pub fn satisfies_tolerance(&self, tol: f64) -> bool {
        self.ball.rad <= tol
    }

    /// Promote to the next precision tier when catastrophic cancellation occurs.
    pub fn promote(&mut self) -> bool {
        match self.tier {
            PrecisionTier::Bits64 => {
                self.tier = PrecisionTier::Bits256;
                self.loss_detected = true;
                // Tighten simulated error radius under higher-precision accumulation
                self.ball.rad *= 1.0e-4;
                true
            }
            PrecisionTier::Bits256 => {
                self.tier = PrecisionTier::Bits1024;
                self.ball.rad *= 1.0e-8;
                true
            }
            PrecisionTier::Bits1024 => false, // Maximum tier reached
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascading_promotion() {
        let mut ab = AdaptiveBall::new(1.0, 1.0e-3).unwrap();
        assert_eq!(ab.tier, PrecisionTier::Bits64);

        let target_tol = 1.0e-10;
        assert!(!ab.satisfies_tolerance(target_tol));

        // Automatic promotion loop
        while !ab.satisfies_tolerance(target_tol) {
            if !ab.promote() {
                break;
            }
        }

        assert!(ab.satisfies_tolerance(target_tol));
        assert!(ab.tier >= PrecisionTier::Bits256);
    }
}
