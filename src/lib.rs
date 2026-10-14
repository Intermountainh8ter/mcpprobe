//! Core types and estimators for mcpprobe.
//!
//! The harness runs `n` independent trials of each task and reports both pass@k
//! (any k out of n trials succeeds) and pass^k (all k out of n trials succeed),
//! using the unbiased combinatorial estimators. See `docs/PREREGISTRATION.md`
//! for the full statistical plan this crate commits to; this file implements it.

use serde::{Deserialize, Serialize};

/// Outcome of one trial of one task.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Outcome {
    Pass,
    Fail,
    /// The server refused to answer or returned an unparseable response.
    /// Counted as fail for pass@k but tracked separately for the audit.
    Refuse,
}

impl Outcome {
    pub fn is_pass(self) -> bool {
        matches!(self, Outcome::Pass)
    }
}

/// Combinatorial estimator for pass@k: the probability that at least one of k
/// trials drawn without replacement from n is a pass. See Chen et al. (2021).
pub fn pass_at_k(n: usize, c: usize, k: usize) -> f64 {
    assert!(k <= n, "k must be <= n");
    if n - c < k {
        return 1.0;
    }
    let mut p = 1.0f64;
    for i in 0..k {
        p *= (n - c - i) as f64 / (n - i) as f64;
    }
    1.0 - p
}

/// pass^k: the probability that all k trials drawn without replacement from n
/// succeed. The complement of "at least one fails".
pub fn pass_power_k(n: usize, c: usize, k: usize) -> f64 {
    assert!(k <= n, "k must be <= n");
    if c < k {
        return 0.0;
    }
    let mut p = 1.0f64;
    for i in 0..k {
        p *= (c - i) as f64 / (n - i) as f64;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_at_k_matches_binomial_identity() {
        // k=1 is just c/n.
        assert!((pass_at_k(8, 3, 1) - 3.0 / 8.0).abs() < 1e-12);
        // k=n is 1 iff c>=1.
        assert!((pass_at_k(8, 1, 8) - 1.0).abs() < 1e-12);
        assert!((pass_at_k(8, 0, 8) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn pass_power_k_matches_definition() {
        // c=n means pass^k=1 for any k<=n.
        assert!((pass_power_k(8, 8, 4) - 1.0).abs() < 1e-12);
        // c<k means pass^k=0.
        assert_eq!(pass_power_k(8, 2, 4), 0.0);
        // k=1 is c/n.
        assert!((pass_power_k(8, 3, 1) - 3.0 / 8.0).abs() < 1e-12);
    }

    #[test]
    fn outcome_pass_is_pass_fail_is_not() {
        assert!(Outcome::Pass.is_pass());
        assert!(!Outcome::Fail.is_pass());
        assert!(!Outcome::Refuse.is_pass());
    }
}
