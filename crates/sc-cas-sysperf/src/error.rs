//! Unified error type for the `sc-cas-sysperf` crate.

/// All failure modes surfaced by sysperf computations.
#[derive(Debug, Clone, PartialEq)]
pub enum SysperfError {
    /// Ball construction failed (NaN/Inf input or negative radius).
    InvalidBall(String),
    /// A hop chain must contain at least one service hop.
    EmptyHopChain,
    /// SLO uptime must be in the half-open range (0.0, 1.0].
    InvalidUptimePercentage(f64),
    /// M/M/c arrival rate must be strictly positive and less than `c * μ`.
    InvalidQueueParameters {
        /// Arrival rate λ.
        lambda: f64,
        /// Per-server service rate μ.
        mu: f64,
        /// Number of parallel servers c.
        c: u32,
    },
}

impl std::fmt::Display for SysperfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBall(msg) => write!(f, "Ball construction error: {msg}"),
            Self::EmptyHopChain => write!(f, "Hop chain must have at least one service"),
            Self::InvalidUptimePercentage(p) => {
                write!(f, "Uptime {p:.6} is outside (0.0, 1.0]")
            }
            Self::InvalidQueueParameters { lambda, mu, c } => write!(
                f,
                "Queue is unstable: λ={lambda:.4} ≥ c*μ={}",
                *c as f64 * mu
            ),
        }
    }
}

impl std::error::Error for SysperfError {}
