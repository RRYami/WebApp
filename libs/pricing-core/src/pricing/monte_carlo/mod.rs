//! Monte Carlo option pricing with AAD (Automatic Differentiation).

pub mod aad;
pub mod core;

pub use aad::{ADNode, ADTape, Operation};
pub use core::{GreeksWithUncertainty, MonteCarlo, MonteCarloResult, VarianceStats};
