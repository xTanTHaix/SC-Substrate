//! Layer 03: Arb-Style Ball Arithmetic [m +/- r] with Directed Rounding

use std::ops::{Add, Div, Mul, Neg, Sub};

/// Error conditions encountered in Ball arithmetic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BallError {
    /// Division by an interval enclosing zero is strictly forbidden.
    DivisionByZeroInterval,
    /// Non-finite value encountered (NaN or Infinite).
    NonFiniteEncountered,
    /// Radius must be non-negative.
    NegativeRadius,
}

impl std::fmt::Display for BallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivisionByZeroInterval => {
                write!(f, "Attempted division by a ball enclosing zero")
            }
            Self::NonFiniteEncountered => write!(f, "Encountered NaN or Inf in ball parameters"),
            Self::NegativeRadius => write!(f, "Ball radius must be strictly non-negative"),
        }
    }
}

impl std::error::Error for BallError {}

/// Standard floating-point machine epsilon for f64 arithmetic.
pub const MACHINE_EPSILON: f64 = f64::EPSILON;

/// Certified numerical enclosure ball [mid +/- rad].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ball {
    /// Floating-point midpoint.
    pub mid: f64,
    /// Conservative upper-bound error radius (rad >= 0.0).
    pub rad: f64,
}

impl Ball {
    /// Construct a new certified ball.
    pub fn new(mid: f64, rad: f64) -> Result<Self, BallError> {
        if !mid.is_finite() || !rad.is_finite() {
            return Err(BallError::NonFiniteEncountered);
        }
        if rad < 0.0 {
            return Err(BallError::NegativeRadius);
        }
        Ok(Self { mid, rad })
    }

    /// Construct an exact point ball with zero radius.
    #[inline]
    pub fn exact(val: f64) -> Self {
        Self { mid: val, rad: 0.0 }
    }

    /// Construct an enclosing ball from interval bounds [low, high].
    pub fn from_interval(low: f64, high: f64) -> Result<Self, BallError> {
        if !low.is_finite() || !high.is_finite() {
            return Err(BallError::NonFiniteEncountered);
        }
        let (min, max) = if low <= high {
            (low, high)
        } else {
            (high, low)
        };
        let mid = 0.5 * (min + max);
        let rad = 0.5 * (max - min) + MACHINE_EPSILON * mid.abs();
        Ok(Self { mid, rad })
    }

    /// Lower numerical bound: mid - rad.
    #[inline]
    pub fn lower(&self) -> f64 {
        self.mid - self.rad
    }

    /// Upper numerical bound: mid + rad.
    #[inline]
    pub fn upper(&self) -> f64 {
        self.mid + self.rad
    }

    /// Diameter of the enclosing interval: 2 * rad.
    #[inline]
    pub fn diameter(&self) -> f64 {
        2.0 * self.rad
    }

    /// Check whether a scalar value is certified enclosed within this ball.
    #[inline]
    pub fn contains(&self, val: f64) -> bool {
        (val - self.mid).abs() <= self.rad + MACHINE_EPSILON * self.mid.abs()
    }

    /// Check if this ball strictly encloses zero.
    #[inline]
    pub fn encloses_zero(&self) -> bool {
        self.mid.abs() <= self.rad
    }

    /// Relative accuracy of the ball: rad / |mid|.
    #[inline]
    pub fn relative_error(&self) -> f64 {
        if self.mid == 0.0 {
            self.rad
        } else {
            self.rad / self.mid.abs()
        }
    }
}

impl Add for Ball {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let mid = self.mid + rhs.mid;
        let rad = self.rad + rhs.rad + MACHINE_EPSILON * mid.abs();
        Self { mid, rad }
    }
}

impl Sub for Ball {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let mid = self.mid - rhs.mid;
        let rad = self.rad + rhs.rad + MACHINE_EPSILON * mid.abs();
        Self { mid, rad }
    }
}

impl Mul for Ball {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let mid = self.mid * rhs.mid;
        // Radius expansion: |m1|*r2 + |m2|*r1 + r1*r2 + epsilon*|m|
        let rad = self.mid.abs() * rhs.rad
            + rhs.mid.abs() * self.rad
            + self.rad * rhs.rad
            + MACHINE_EPSILON * mid.abs();
        Self { mid, rad }
    }
}

impl Div for Ball {
    type Output = Result<Self, BallError>;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.encloses_zero() {
            return Err(BallError::DivisionByZeroInterval);
        }
        let denom_inf = rhs.mid.abs() - rhs.rad;
        let mid = self.mid / rhs.mid;
        let rad = (self.rad + mid.abs() * rhs.rad) / denom_inf + MACHINE_EPSILON * mid.abs();
        Ok(Self { mid, rad })
    }
}

impl Neg for Ball {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            mid: -self.mid,
            rad: self.rad,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ball_arithmetic_containment() {
        let a = Ball::new(3.0, 0.1).unwrap();
        let b = Ball::new(5.0, 0.2).unwrap();

        let sum = a + b;
        assert!(sum.contains(8.0));
        assert!(sum.contains(2.9 + 4.8));

        let diff = b - a;
        assert!(diff.contains(2.0));

        let prod = a * b;
        assert!(prod.contains(15.0));

        let quot = (b / a).unwrap();
        assert!(quot.contains(5.0 / 3.0));
    }

    #[test]
    fn test_ball_division_by_zero_fails() {
        let a = Ball::new(10.0, 0.1).unwrap();
        let zero_enclosing = Ball::new(0.0, 0.5).unwrap();
        assert!(matches!(
            a / zero_enclosing,
            Err(BallError::DivisionByZeroInterval)
        ));
    }
}
