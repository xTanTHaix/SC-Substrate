//! SLO / Error Budget: maps uptime percentage targets to permissible downtime windows.
//!
//! Provides exact rational-style arithmetic using f64 for the SLA fractions, with
//! fail-fast guards on domain validity.  All duration results are in seconds so
//! callers can convert to preferred units.

use crate::error::SysperfError;

/// Number of seconds in common rolling windows.
const SECONDS_PER_MINUTE: f64 = 60.0;
const SECONDS_PER_HOUR: f64 = 3600.0;
const SECONDS_PER_DAY: f64 = 86_400.0;
const SECONDS_PER_WEEK: f64 = 604_800.0;
const SECONDS_PER_MONTH_30: f64 = 30.0 * SECONDS_PER_DAY;
const SECONDS_PER_YEAR: f64 = 365.25 * SECONDS_PER_DAY;

/// A validated SLO target expressed as an uptime fraction in (0, 1].
///
/// Construct via [`SloTarget::from_percentage`] or [`SloTarget::from_nines`].
#[derive(Debug, Clone, Copy)]
pub struct SloTarget {
    /// Validated uptime fraction: 0.0 < uptime_fraction ≤ 1.0.
    uptime_fraction: f64,
}

impl SloTarget {
    /// Construct from an explicit decimal fraction (e.g. `0.9999` for 99.99%).
    ///
    /// # Errors
    /// Returns [`SysperfError::InvalidUptimePercentage`] when `fraction` is
    /// outside `(0.0, 1.0]` or non-finite.
    pub fn from_fraction(fraction: f64) -> Result<Self, SysperfError> {
        if !fraction.is_finite() || fraction <= 0.0 || fraction > 1.0 {
            return Err(SysperfError::InvalidUptimePercentage(fraction));
        }
        Ok(Self { uptime_fraction: fraction })
    }

    /// Construct from a percentage string representation (e.g. `99.99`).
    ///
    /// # Errors
    /// Propagates [`SysperfError::InvalidUptimePercentage`] for out-of-range values.
    pub fn from_percentage(percent: f64) -> Result<Self, SysperfError> {
        Self::from_fraction(percent / 100.0)
    }

    /// Convenience constructor: build from a "number of nines" (e.g. `3` → 99.9%).
    ///
    /// # Errors
    /// Returns an error when `nines` is zero or so large the fraction rounds to 1.0.
    pub fn from_nines(nines: u8) -> Result<Self, SysperfError> {
        let fraction = 1.0 - 10_f64.powi(-(nines as i32));
        Self::from_fraction(fraction)
    }

    /// Raw uptime fraction (0, 1].
    #[inline]
    pub fn uptime_fraction(&self) -> f64 {
        self.uptime_fraction
    }

    /// Downtime fraction: 1.0 - uptime_fraction.
    #[inline]
    pub fn downtime_fraction(&self) -> f64 {
        1.0 - self.uptime_fraction
    }

    /// Uptime expressed as a percentage string, rounded to 4 decimal places.
    pub fn as_percentage_str(&self) -> String {
        format!("{:.4}%", self.uptime_fraction * 100.0)
    }
}

/// Permissible downtime windows derived from an [`SloTarget`] over standard rolling periods.
#[derive(Debug, Clone, Copy)]
pub struct ErrorBudget {
    /// Permissible downtime per minute (seconds).
    pub per_minute_s: f64,
    /// Permissible downtime per hour (seconds).
    pub per_hour_s: f64,
    /// Permissible downtime per day (seconds).
    pub per_day_s: f64,
    /// Permissible downtime per week (seconds).
    pub per_week_s: f64,
    /// Permissible downtime per 30-day month (seconds).
    pub per_month_s: f64,
    /// Permissible downtime per 365.25-day year (seconds).
    pub per_year_s: f64,
}

impl ErrorBudget {
    /// Derive the complete error budget from a validated [`SloTarget`].
    pub fn from_slo(target: &SloTarget) -> Self {
        let df = target.downtime_fraction();
        Self {
            per_minute_s: df * SECONDS_PER_MINUTE,
            per_hour_s: df * SECONDS_PER_HOUR,
            per_day_s: df * SECONDS_PER_DAY,
            per_week_s: df * SECONDS_PER_WEEK,
            per_month_s: df * SECONDS_PER_MONTH_30,
            per_year_s: df * SECONDS_PER_YEAR,
        }
    }
}

impl std::fmt::Display for ErrorBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error Budget:")?;
        writeln!(f, "  per minute : {:.4} s", self.per_minute_s)?;
        writeln!(f, "  per hour   : {:.4} s", self.per_hour_s)?;
        writeln!(f, "  per day    : {:.4} s ({:.2} min)", self.per_day_s, self.per_day_s / 60.0)?;
        writeln!(f, "  per week   : {:.4} s ({:.2} min)", self.per_week_s, self.per_week_s / 60.0)?;
        writeln!(
            f,
            "  per month  : {:.4} s ({:.2} min)",
            self.per_month_s,
            self.per_month_s / 60.0
        )?;
        write!(
            f,
            "  per year   : {:.4} s ({:.2} min)",
            self.per_year_s,
            self.per_year_s / 60.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_nines_error_budget() {
        // 99.99% → downtime fraction = 0.0001
        let slo = SloTarget::from_nines(4).unwrap();
        let budget = ErrorBudget::from_slo(&slo);

        // Per-year downtime: 365.25 * 86400 * 0.0001 ≈ 3155.76 s ≈ 52.6 min
        let expected_year_s = 0.0001 * SECONDS_PER_YEAR;
        assert!(
            (budget.per_year_s - expected_year_s).abs() < 0.01,
            "4-nines yearly budget: expected {expected_year_s:.4} s, got {:.4} s",
            budget.per_year_s
        );
    }

    #[test]
    fn test_three_nines_from_percentage() {
        let slo = SloTarget::from_percentage(99.9).unwrap();
        assert!((slo.downtime_fraction() - 0.001).abs() < 1e-9);
    }

    #[test]
    fn test_invalid_uptime_rejected() {
        assert!(matches!(
            SloTarget::from_fraction(0.0),
            Err(SysperfError::InvalidUptimePercentage(_))
        ));
        assert!(matches!(
            SloTarget::from_fraction(1.001),
            Err(SysperfError::InvalidUptimePercentage(_))
        ));
        assert!(matches!(
            SloTarget::from_fraction(f64::NAN),
            Err(SysperfError::InvalidUptimePercentage(_))
        ));
    }

    #[test]
    fn test_perfect_uptime_accepted() {
        let slo = SloTarget::from_fraction(1.0).unwrap();
        assert_eq!(slo.downtime_fraction(), 0.0);
        let budget = ErrorBudget::from_slo(&slo);
        assert_eq!(budget.per_year_s, 0.0);
    }
}
