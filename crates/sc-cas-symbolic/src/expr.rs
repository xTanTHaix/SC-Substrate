//! Exact Symbolic Expression AST & Canonical Simplifier (Alg 02: Phi_Omni-Min)

use sc_cas_types::Ball;
use std::collections::HashMap;

/// Symbolic evaluation error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprError {
    /// Undefined variable during evaluation.
    UndefinedVariable(String),
    /// Division by zero in symbolic evaluation.
    DivisionByZero,
}

impl std::fmt::Display for ExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndefinedVariable(v) => write!(f, "Variable '{v}' is not bound in environment"),
            Self::DivisionByZero => write!(f, "Division by zero in expression evaluation"),
        }
    }
}

impl std::error::Error for ExprError {}

/// Abstract Syntax Tree for exact symbolic and holonomic expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Constant numeric value enclosed in an Arb Ball.
    Constant(Ball),
    /// Named symbolic variable.
    Variable(String),
    /// Sum of terms.
    Add(Vec<Expr>),
    /// Product of terms.
    Mul(Vec<Expr>),
    /// Negation of an expression.
    Neg(Box<Expr>),
    /// Exponentiation: base ^ exp.
    Pow(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Create a numeric constant node.
    pub fn constant(val: f64) -> Self {
        Self::Constant(Ball::exact(val))
    }

    /// Create a variable node.
    pub fn var(name: impl Into<String>) -> Self {
        Self::Variable(name.into())
    }

    /// Complexity cost function for Phi_Omni-Min optimization.
    pub fn complexity(&self) -> usize {
        match self {
            Self::Constant(_) => 1,
            Self::Variable(_) => 1,
            Self::Add(terms) => 1 + terms.iter().map(|t| t.complexity()).sum::<usize>(),
            Self::Mul(terms) => 2 + terms.iter().map(|t| t.complexity()).sum::<usize>(),
            Self::Neg(inner) => 1 + inner.complexity(),
            Self::Pow(base, exp) => 3 + base.complexity() + exp.complexity(),
        }
    }

    /// Numerical evaluation under a variable assignment environment.
    pub fn eval(&self, env: &HashMap<String, Ball>) -> Result<Ball, ExprError> {
        match self {
            Self::Constant(b) => Ok(*b),
            Self::Variable(name) => env
                .get(name)
                .copied()
                .ok_or_else(|| ExprError::UndefinedVariable(name.clone())),
            Self::Add(terms) => {
                let mut sum = Ball::exact(0.0);
                for term in terms {
                    sum = sum + term.eval(env)?;
                }
                Ok(sum)
            }
            Self::Mul(terms) => {
                let mut prod = Ball::exact(1.0);
                for term in terms {
                    prod = prod * term.eval(env)?;
                }
                Ok(prod)
            }
            Self::Neg(inner) => Ok(-inner.eval(env)?),
            Self::Pow(base, exp) => {
                let b = base.eval(env)?;
                let e = exp.eval(env)?;
                let mid = b.mid.powf(e.mid);
                let rad = b.rad * e.mid.abs() * b.mid.powf(e.mid - 1.0).abs() + e.rad * mid.abs();
                Ball::new(mid, rad).map_err(|_| ExprError::DivisionByZero)
            }
        }
    }

    /// Canonical simplification reducing trivial zero-additions and one-multiplications.
    pub fn canonicalize(&self) -> Self {
        match self {
            Self::Add(terms) => {
                let mut simplified: Vec<Expr> = terms
                    .iter()
                    .map(|t| t.canonicalize())
                    .filter(|t| !matches!(t, Self::Constant(b) if b.mid == 0.0 && b.rad == 0.0))
                    .collect();
                if simplified.is_empty() {
                    Self::constant(0.0)
                } else if simplified.len() == 1 {
                    simplified.pop().unwrap()
                } else {
                    Self::Add(simplified)
                }
            }
            Self::Mul(terms) => {
                let mut simplified: Vec<Expr> = terms.iter().map(|t| t.canonicalize()).collect();
                if simplified
                    .iter()
                    .any(|t| matches!(t, Self::Constant(b) if b.mid == 0.0 && b.rad == 0.0))
                {
                    return Self::constant(0.0);
                }
                simplified
                    .retain(|t| !matches!(t, Self::Constant(b) if b.mid == 1.0 && b.rad == 0.0));
                if simplified.is_empty() {
                    Self::constant(1.0)
                } else if simplified.len() == 1 {
                    simplified.pop().unwrap()
                } else {
                    Self::Mul(simplified)
                }
            }
            Self::Neg(inner) => {
                let simp = inner.canonicalize();
                if let Self::Constant(b) = simp {
                    Self::Constant(-b)
                } else {
                    Self::Neg(Box::new(simp))
                }
            }
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_evaluation_and_canonicalization() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Ball::exact(3.0));

        let expr = Expr::Add(vec![
            Expr::var("x"),
            Expr::constant(0.0),
            Expr::constant(5.0),
        ]);

        let val = expr.eval(&env).unwrap();
        assert_eq!(val.mid, 8.0);

        let canon = expr.canonicalize();
        assert!(canon.complexity() < expr.complexity());
    }
}
