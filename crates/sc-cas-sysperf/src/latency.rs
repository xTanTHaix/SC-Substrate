//! End-to-End Latency Bound propagation using certified Arb ball arithmetic.
//!
//! Models each microservice as a `ServiceHop` with a midpoint latency and a jitter
//! radius.  A `HopChain` sums these using Ball addition, which inflates the radius
//! monotonically — guaranteeing the true worst-case tail is always enclosed.

use sc_cas_types::Ball;

use crate::error::SysperfError;

/// A single service on the call path, described by its mean latency and jitter radius
/// (both in milliseconds).
#[derive(Debug, Clone)]
pub struct ServiceHop {
    /// Human-readable name for diagnostics.
    pub name: &'static str,
    /// Certified latency ball: midpoint ± jitter radius (ms).
    pub latency_ms: Ball,
}

impl ServiceHop {
    /// Construct a `ServiceHop` from explicit mean and jitter values.
    ///
    /// # Errors
    /// Returns [`SysperfError::InvalidBall`] when either value is non-finite or
    /// `jitter_ms` is negative.
    pub fn new(name: &'static str, mean_ms: f64, jitter_ms: f64) -> Result<Self, SysperfError> {
        let latency_ms = Ball::new(mean_ms, jitter_ms)
            .map_err(|e| SysperfError::InvalidBall(e.to_string()))?;
        Ok(Self { name, latency_ms })
    }
}

/// Certified worst-case latency result for a call chain.
#[derive(Debug, Clone, Copy)]
pub struct LatencyBound {
    /// Accumulated Ball enclosing the true end-to-end latency (ms).
    pub ball: Ball,
}

impl LatencyBound {
    /// Lower bound: best-case latency (ms).
    #[inline]
    pub fn lower_ms(&self) -> f64 {
        self.ball.lower()
    }

    /// Upper bound: certified worst-case tail latency (ms).
    #[inline]
    pub fn upper_ms(&self) -> f64 {
        self.ball.upper()
    }

    /// Whether the certified upper bound is strictly within the given SLA ceiling (ms).
    ///
    /// Returns `true` only when the worst-case tail is provably enclosed below `sla_ms`.
    /// A `false` result means the SLA may be breached — not that it will be.
    #[inline]
    pub fn sla_guaranteed(&self, sla_ms: f64) -> bool {
        self.ball.upper() < sla_ms
    }
}

impl std::fmt::Display for LatencyBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.3} ± {:.3} ms  [lower={:.3}, upper={:.3}]",
            self.ball.mid,
            self.ball.rad,
            self.lower_ms(),
            self.upper_ms()
        )
    }
}

/// An ordered sequence of service hops forming a single synchronous call chain.
#[derive(Debug, Clone)]
pub struct HopChain {
    hops: Vec<ServiceHop>,
}

impl HopChain {
    /// Build a chain from a non-empty slice of hops.
    ///
    /// # Errors
    /// Returns [`SysperfError::EmptyHopChain`] when `hops` is empty.
    pub fn new(hops: Vec<ServiceHop>) -> Result<Self, SysperfError> {
        if hops.is_empty() {
            return Err(SysperfError::EmptyHopChain);
        }
        Ok(Self { hops })
    }

    /// Propagate latency jitter across all hops using Ball addition.
    ///
    /// Each addition inflates the radius by the sum of individual radii plus a
    /// machine-epsilon rounding guard, so the final interval rigorously encloses
    /// every possible accumulated delay.
    pub fn total_latency(&self) -> LatencyBound {
        // Accumulate starting from the first hop; Ball::add is not a fallible op.
        let accumulated = self
            .hops
            .iter()
            .skip(1)
            .fold(self.hops[0].latency_ms, |acc, hop| acc + hop.latency_ms);
        LatencyBound { ball: accumulated }
    }

    /// Verify the chain against a hard SLA ceiling (ms).
    ///
    /// Returns the [`LatencyBound`] so callers can inspect the full interval
    /// regardless of pass/fail status.
    pub fn verify_sla(&self, sla_ms: f64) -> (LatencyBound, bool) {
        let bound = self.total_latency();
        let passed = bound.sla_guaranteed(sla_ms);
        (bound, passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_hop_chain_containment() {
        // Mirrors the roadmap example: gateway(5±0.5) + auth(12±2.0) + db(25±5.0)
        let gateway = ServiceHop::new("API Gateway", 5.0, 0.5).unwrap();
        let auth = ServiceHop::new("Auth Service", 12.0, 2.0).unwrap();
        let db = ServiceHop::new("Read Replica", 25.0, 5.0).unwrap();

        let chain = HopChain::new(vec![gateway, auth, db]).unwrap();
        let bound = chain.total_latency();

        // Midpoint must equal the sum of all midpoints.
        assert!(
            (bound.ball.mid - 42.0).abs() < 1e-9,
            "midpoint should be 42.0 ms, got {}",
            bound.ball.mid
        );
        // Radius must be >= sum of raw jitters (7.5 ms) due to rounding inflation.
        assert!(bound.ball.rad >= 7.5, "radius must enclose raw jitter sum");
        // True worst case 49.5 ms is provably below SLA ceiling of 55.0 ms.
        assert!(bound.sla_guaranteed(55.0));
        // Fails when SLA is tighter than worst-case bound.
        assert!(!bound.sla_guaranteed(bound.upper_ms() - 0.001));
    }

    #[test]
    fn test_empty_chain_rejected() {
        assert!(matches!(
            HopChain::new(vec![]),
            Err(SysperfError::EmptyHopChain)
        ));
    }

    #[test]
    fn test_invalid_hop_rejected() {
        assert!(matches!(
            ServiceHop::new("bad", f64::NAN, 0.0),
            Err(SysperfError::InvalidBall(_))
        ));
        assert!(matches!(
            ServiceHop::new("neg-rad", 5.0, -1.0),
            Err(SysperfError::InvalidBall(_))
        ));
    }

    #[test]
    fn test_single_hop_chain() {
        let hop = ServiceHop::new("Solo", 30.0, 3.0).unwrap();
        let chain = HopChain::new(vec![hop]).unwrap();
        let bound = chain.total_latency();
        assert!((bound.ball.mid - 30.0).abs() < 1e-9);
        assert!((bound.ball.rad - 3.0).abs() < 1e-9);
    }
}
