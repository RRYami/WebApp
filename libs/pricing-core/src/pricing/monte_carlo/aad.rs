//! Reverse-mode Automatic Differentiation (AAD) for Monte Carlo Greeks.
//!
//! This module implements a tape-based reverse-mode AD system.
//! All operations are recorded on a tape, then backward pass computes derivatives.

/// A node in the computation graph
#[derive(Debug, Clone)]
pub struct ADNode {
    /// Value at this node
    pub value: f64,
    /// Adjoint (derivative of output w.r.t. this node)
    pub adjoint: f64,
    /// Operation that produced this node
    pub op: Operation,
    /// Parent node indices
    pub parents: [usize; 2],
}

/// Operations supported by the AD system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    /// Variable (input)
    Variable,
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Exponential
    Exp,
    /// Natural logarithm
    Ln,
    /// Square root
    Sqrt,
    /// Constant
    Constant,
}

/// AD tape for recording computations
#[derive(Debug, Clone)]
pub struct ADTape {
    nodes: Vec<ADNode>,
}

impl ADTape {
    /// Create new empty tape
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a variable to the tape (input with derivative 1.0)
    pub fn variable(&mut self, value: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Variable,
            parents: [idx, idx], // Self-referential for variables
        });
        idx
    }

    /// Add a constant to the tape
    pub fn constant(&mut self, value: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Constant,
            parents: [idx, idx],
        });
        idx
    }

    /// Addition: a + b
    pub fn add(&mut self, a: usize, b: usize) -> usize {
        let value = self.nodes[a].value + self.nodes[b].value;
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Add,
            parents: [a, b],
        });
        idx
    }

    /// Subtraction: a - b
    pub fn sub(&mut self, a: usize, b: usize) -> usize {
        let value = self.nodes[a].value - self.nodes[b].value;
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Sub,
            parents: [a, b],
        });
        idx
    }

    /// Multiplication: a * b
    pub fn mul(&mut self, a: usize, b: usize) -> usize {
        let value = self.nodes[a].value * self.nodes[b].value;
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Mul,
            parents: [a, b],
        });
        idx
    }

    /// Division: a / b
    pub fn div(&mut self, a: usize, b: usize) -> usize {
        let value = self.nodes[a].value / self.nodes[b].value;
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Div,
            parents: [a, b],
        });
        idx
    }

    /// Exponential: exp(a)
    pub fn exp(&mut self, a: usize) -> usize {
        let value = self.nodes[a].value.exp();
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Exp,
            parents: [a, a],
        });
        idx
    }

    /// Natural log: ln(a)
    pub fn ln(&mut self, a: usize) -> usize {
        let value = self.nodes[a].value.ln();
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Ln,
            parents: [a, a],
        });
        idx
    }

    /// Square root: sqrt(a)
    pub fn sqrt(&mut self, a: usize) -> usize {
        let value = self.nodes[a].value.sqrt();
        let idx = self.nodes.len();
        self.nodes.push(ADNode {
            value,
            adjoint: 0.0,
            op: Operation::Sqrt,
            parents: [a, a],
        });
        idx
    }

    /// Reverse-mode automatic differentiation (backward pass)
    /// Computes derivatives of output w.r.t. all inputs
    pub fn reverse(&mut self, output_idx: usize) {
        // Initialize adjoint of output to 1.0
        self.nodes[output_idx].adjoint = 1.0;

        // Sweep backward through the tape
        for i in (0..self.nodes.len()).rev() {
            let adjoint = self.nodes[i].adjoint;
            let node = &self.nodes[i];

            // Skip if adjoint is zero (no contribution)
            if adjoint == 0.0 {
                continue;
            }

            match node.op {
                Operation::Variable | Operation::Constant => {
                    // No parents to propagate to
                }
                Operation::Add => {
                    // d(a+b)/da = 1, d(a+b)/db = 1
                    let (p0, p1) = (node.parents[0], node.parents[1]);
                    self.nodes[p0].adjoint += adjoint;
                    self.nodes[p1].adjoint += adjoint;
                }
                Operation::Sub => {
                    // d(a-b)/da = 1, d(a-b)/db = -1
                    let (p0, p1) = (node.parents[0], node.parents[1]);
                    self.nodes[p0].adjoint += adjoint;
                    self.nodes[p1].adjoint -= adjoint;
                }
                Operation::Mul => {
                    // d(a*b)/da = b, d(a*b)/db = a
                    let (p0, p1) = (node.parents[0], node.parents[1]);
                    let a_val = self.nodes[p0].value;
                    let b_val = self.nodes[p1].value;
                    self.nodes[p0].adjoint += adjoint * b_val;
                    self.nodes[p1].adjoint += adjoint * a_val;
                }
                Operation::Div => {
                    // d(a/b)/da = 1/b, d(a/b)/db = -a/b²
                    let (p0, p1) = (node.parents[0], node.parents[1]);
                    let a_val = self.nodes[p0].value;
                    let b_val = self.nodes[p1].value;
                    self.nodes[p0].adjoint += adjoint / b_val;
                    self.nodes[p1].adjoint -= adjoint * a_val / (b_val * b_val);
                }
                Operation::Exp => {
                    // d(exp(a))/da = exp(a)
                    let p = node.parents[0];
                    self.nodes[p].adjoint += adjoint * node.value;
                }
                Operation::Ln => {
                    // d(ln(a))/da = 1/a
                    let p = node.parents[0];
                    self.nodes[p].adjoint += adjoint / self.nodes[p].value;
                }
                Operation::Sqrt => {
                    // d(sqrt(a))/da = 1/(2*sqrt(a))
                    let p = node.parents[0];
                    self.nodes[p].adjoint += adjoint / (2.0 * node.value);
                }
            }
        }
    }

    /// Get adjoint (derivative) of a variable
    pub fn get_adjoint(&self, idx: usize) -> f64 {
        self.nodes[idx].adjoint
    }

    /// Get value of a node
    pub fn get_value(&self, idx: usize) -> f64 {
        self.nodes[idx].value
    }

    /// Reset adjoints for new computation
    pub fn reset_adjoints(&mut self) {
        for node in &mut self.nodes {
            node.adjoint = 0.0;
        }
    }

    /// Clear the tape
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

impl Default for ADTape {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_derivative() {
        let mut tape = ADTape::new();
        let a = tape.variable(2.0);
        let b = tape.variable(3.0);
        let c = tape.add(a, b); // c = a + b = 5

        tape.reverse(c);

        assert_eq!(tape.get_adjoint(a), 1.0); // dc/da = 1
        assert_eq!(tape.get_adjoint(b), 1.0); // dc/db = 1
    }

    #[test]
    fn test_mul_derivative() {
        let mut tape = ADTape::new();
        let a = tape.variable(2.0);
        let b = tape.variable(3.0);
        let c = tape.mul(a, b); // c = a * b = 6

        tape.reverse(c);

        assert_eq!(tape.get_adjoint(a), 3.0); // dc/da = b = 3
        assert_eq!(tape.get_adjoint(b), 2.0); // dc/db = a = 2
    }

    #[test]
    fn test_exp_derivative() {
        let mut tape = ADTape::new();
        let a = tape.variable(1.0);
        let b = tape.exp(a); // b = exp(1) ≈ 2.718

        tape.reverse(b);

        // db/da = exp(1) ≈ 2.718
        assert!((tape.get_adjoint(a) - 2.718281828).abs() < 1e-6);
    }

    #[test]
    fn test_chain_rule() {
        // Test: f(x) = exp(x² + 1)
        // f'(x) = exp(x² + 1) * 2x
        // At x = 1: f(1) = exp(2) ≈ 7.389, f'(1) = exp(2) * 2 ≈ 14.778

        let mut tape = ADTape::new();
        let x = tape.variable(1.0);
        let one = tape.constant(1.0);
        let x2 = tape.mul(x, x); // x²
        let x2_plus_1 = tape.add(x2, one); // x² + 1
        let result = tape.exp(x2_plus_1); // exp(x² + 1)

        tape.reverse(result);

        let expected = 7.389056099 * 2.0; // exp(2) * 2
        assert!((tape.get_adjoint(x) - expected).abs() < 1e-3);
    }

    #[test]
    fn test_gbm_price_derivative() {
        // Test GBM price derivative w.r.t. spot
        // S_T = S_0 * exp((r - 0.5σ²)T + σ√T Z)
        // dS_T/dS_0 = S_T / S_0

        let spot = 100.0;
        let drift = 0.03;
        let diffusion = 0.2;
        let z = 0.5;

        let mut tape = ADTape::new();
        let s0 = tape.variable(spot);
        let drift_term = tape.constant(drift);
        let diff_term = tape.constant(diffusion * z);
        let sum = tape.add(drift_term, diff_term);
        let exp_term = tape.exp(sum);
        let st = tape.mul(s0, exp_term);

        tape.reverse(st);

        let expected_derivative = tape.get_value(st) / spot;
        assert!((tape.get_adjoint(s0) - expected_derivative).abs() < 1e-10);
    }
}
