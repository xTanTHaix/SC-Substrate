//! `sc-cas-sysperf` — Certified SLA, Latency Enclosure & Capacity Planning
//!
//! Provides three main computation surfaces for systems engineers:
//! - **End-to-end latency bounds**: propagates Arb ball jitter across microservice hop chains
//! - **Queueing theory**: Little's Law (L = λW) and M/M/c queue load thresholds
//! - **SLO / Error budget**: maps uptime SLA percentages to permissible downtime windows
//!
//! Every latency result is a certified `Ball` interval — the worst-case tail is provably
//! enclosed, not merely estimated.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod latency;
pub mod queue;
pub mod slo;

pub use error::SysperfError;
pub use latency::{HopChain, LatencyBound, ServiceHop};
pub use queue::{MmcQueue, QueueMetrics};
pub use slo::{ErrorBudget, SloTarget};
