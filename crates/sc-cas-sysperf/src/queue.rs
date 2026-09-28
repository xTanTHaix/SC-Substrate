//! Queueing theory: Little's Law (L = λW) and M/M/c steady-state metrics.
//!
//! All capacity figures are computed with `f64` here because queue parameters
//! (arrival and service rates) are inherently statistical; certified Ball bounds
//! on SLA latency live in `latency.rs`.

use crate::error::SysperfError;

/// Steady-state metrics for an M/M/c queue under stable load.
#[derive(Debug, Clone, Copy)]
pub struct QueueMetrics {
    /// Server utilisation ρ = λ / (c · μ). Must be < 1.0 for stability.
    pub utilisation: f64,
    /// Mean number of customers in the system (Little's Law: L = λ · W).
    pub mean_customers: f64,
    /// Mean time in system W = L / λ (seconds).
    pub mean_sojourn_s: f64,
    /// Erlang-C blocking probability P(queuing).
    pub erlang_c: f64,
    /// Mean queueing delay before service (seconds).
    pub mean_queue_delay_s: f64,
}

/// M/M/c queue model: Poisson arrivals, exponential service, `c` servers.
#[derive(Debug, Clone, Copy)]
pub struct MmcQueue {
    /// Arrival rate λ (customers per second).
    pub lambda: f64,
    /// Per-server service rate μ (customers per second per server).
    pub mu: f64,
    /// Number of parallel servers c.
    pub c: u32,
}

impl MmcQueue {
    /// Construct and validate queue parameters.
    ///
    /// # Errors
    /// Returns [`SysperfError::InvalidQueueParameters`] when the queue is
    /// unstable (λ ≥ c·μ) or any parameter is non-positive.
    pub fn new(lambda: f64, mu: f64, c: u32) -> Result<Self, SysperfError> {
        let capacity = c as f64 * mu;
        if lambda <= 0.0 || mu <= 0.0 || c == 0 || lambda >= capacity {
            return Err(SysperfError::InvalidQueueParameters { lambda, mu, c });
        }
        Ok(Self { lambda, mu, c })
    }

    /// Compute steady-state metrics via the Erlang-C formula.
    ///
    /// The Erlang-C probability is:
    ///   P_c = (ρ·c)^c / c! · 1/(1-ρ)  /  [Σ_{k=0}^{c-1} (ρ·c)^k/k! + above]
    /// where ρ = λ/(c·μ).
    pub fn metrics(&self) -> QueueMetrics {
        let rho = self.lambda / (self.c as f64 * self.mu);
        let a = self.lambda / self.mu; // offered load (Erlangs)

        // Erlang-C numerator term: a^c / c! · 1/(1-ρ)
        let c_usize = self.c as usize;
        let erlang_c = erlang_c_probability(a, c_usize, rho);

        // Mean queueing delay: W_q = C(c,a) / (c·μ·(1-ρ))
        let mean_queue_delay_s = erlang_c / (self.c as f64 * self.mu * (1.0 - rho));

        // Mean sojourn time: W = W_q + 1/μ
        let mean_sojourn_s = mean_queue_delay_s + 1.0 / self.mu;

        // Little's Law: L = λ · W
        let mean_customers = self.lambda * mean_sojourn_s;

        QueueMetrics {
            utilisation: rho,
            mean_customers,
            mean_sojourn_s,
            erlang_c,
            mean_queue_delay_s,
        }
    }
}

/// Compute the Erlang-C blocking probability using log-space accumulation to
/// avoid factorial overflow for large `c`.
///
/// Returns the probability that an arriving customer must queue (all servers busy).
fn erlang_c_probability(a: f64, c: usize, rho: f64) -> f64 {
    // Sum Σ_{k=0}^{c-1} a^k / k! in log space, then convert back.
    let mut log_sum_terms: Vec<f64> = Vec::with_capacity(c);
    let mut log_term = 0.0_f64; // log(a^0 / 0!) = 0
    for k in 0..c {
        log_sum_terms.push(log_term);
        // Next term: a^(k+1)/(k+1)! = prev * a/(k+1)
        log_term += a.ln() - ((k + 1) as f64).ln();
    }
    // Numerator in log space: a^c / c! · 1/(1-ρ)
    let log_numerator = log_term - (1.0 - rho).ln();
    let log_max = log_numerator
        .max(log_sum_terms.iter().copied().fold(f64::NEG_INFINITY, f64::max));

    let numerator = (log_numerator - log_max).exp();
    let denominator: f64 = log_sum_terms
        .iter()
        .map(|&lt| (lt - log_max).exp())
        .sum::<f64>()
        + numerator;

    numerator / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_server_mm1_erlang_c() {
        // M/M/1: Erlang-C should equal ρ for c=1.
        let q = MmcQueue::new(0.6, 1.0, 1).unwrap();
        let m = q.metrics();
        // ρ = 0.6, so erlang_c ≈ 0.6 (within 1e-6).
        assert!(
            (m.erlang_c - 0.6).abs() < 1e-6,
            "Erlang-C for M/M/1 with ρ=0.6 should equal 0.6, got {}",
            m.erlang_c
        );
        // Little's Law: L = λW; verify sojourn time W = 1/(μ-λ) = 2.5 s.
        assert!(
            (m.mean_sojourn_s - 2.5).abs() < 1e-6,
            "M/M/1 sojourn should be 2.5 s, got {}",
            m.mean_sojourn_s
        );
    }

    #[test]
    fn test_unstable_queue_rejected() {
        // λ = c·μ → ρ = 1.0, unstable.
        assert!(matches!(
            MmcQueue::new(3.0, 1.0, 3),
            Err(SysperfError::InvalidQueueParameters { .. })
        ));
        // λ > c·μ
        assert!(matches!(
            MmcQueue::new(10.0, 1.0, 3),
            Err(SysperfError::InvalidQueueParameters { .. })
        ));
    }

    #[test]
    fn test_multi_server_utilisation() {
        // 4 servers, λ=2, μ=1 → ρ = 2/4 = 0.5
        let q = MmcQueue::new(2.0, 1.0, 4).unwrap();
        let m = q.metrics();
        assert!((m.utilisation - 0.5).abs() < 1e-9);
        // Erlang-C < 1 for stable queue.
        assert!(m.erlang_c > 0.0 && m.erlang_c < 1.0);
    }

    #[test]
    fn test_littles_law_consistency() {
        let q = MmcQueue::new(0.8, 1.0, 2).unwrap();
        let m = q.metrics();
        // Little's Law: L = λ · W must hold to within floating-point precision.
        let expected_l = q.lambda * m.mean_sojourn_s;
        assert!(
            (m.mean_customers - expected_l).abs() < 1e-9,
            "Little's Law violated: L={} but λW={}",
            m.mean_customers,
            expected_l
        );
    }
}
