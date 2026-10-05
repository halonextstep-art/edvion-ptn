//! Rasch (1PL) Item Response Theory model — pure math, no I/O, fully unit-testable. Used to
//! compute "Estimasi IRT", a platform-calibrated ability estimate shown as a SEPARATE, honestly
//! labeled score alongside the existing simple percentage-based "Skor Instan" (see
//! `domain::tryout::compute_score`).
//!
//! IMPORTANT — what this is NOT: this is not, and cannot be, a replication of the official
//! UTBK/SNBT IRT scoring used by SNPMB. That method's item pool, calibration data, and exact
//! scaling constants are not publicly disclosed, so no platform outside SNPMB can reproduce it
//! exactly. What this module DOES provide is a genuine, from-scratch IRT estimate — item
//! difficulties calibrated from this platform's own historical attempt data, and a student's
//! ability (theta) estimated from their own response pattern against those difficulties. That
//! is an honest, real IRT calculation; it is just not a copy of SNPMB's specific numbers.
//!
//! Model choice: Rasch (1PL) rather than 2PL/3PL — only one parameter (difficulty) needs
//! calibrating per item, which needs far less historical data to be trustworthy and is far
//! simpler to implement correctly without a working compiler in this environment (see
//! CLAUDE.md: no reliable `cargo build` here — changes must be manually/subagent verified).
//!
//! Calibration uses a simplified empirical-logit estimator (not the textbook-exact PROX/Cohen
//! method) — deliberately chosen to minimize numerical-edge-case risk while remaining a
//! legitimate difficulty estimate: `b = -ln(p / (1 - p))` from the empirical proportion-correct
//! `p`, clamped away from 0/1 to avoid infinities.

/// An item's difficulty is only trusted once at least this many historical answers exist for
/// it — below this, `calibrate_difficulty` honestly returns `None` (honest-zero convention)
/// instead of a noisy estimate from a handful of responses. The caller (`IrtService::recalibrate`)
/// is the one that checks a question's total against this constant before calling
/// `calibrate_difficulty`.
pub const MIN_ITEM_SAMPLE_SIZE: i64 = 20;

/// An attempt's IRT score is only shown once at least this many of its questions have a
/// trusted calibration — below this, too few data points exist to estimate ability
/// meaningfully, so `irt_score` on the result honestly stays `None`.
pub const MIN_CALIBRATED_ITEMS_FOR_SCORE: usize = 5;

/// theta (student ability, in logits) is clamped to this range — a Rasch MLE diverges to
/// +/-infinity for an all-correct or all-incorrect response pattern, so an explicit bound is
/// required for the estimate to stay a finite, displayable number.
const THETA_MIN: f64 = -4.0;
const THETA_MAX: f64 = 4.0;

/// Empirical-logit item difficulty from historical proportion-correct. `total` should already
/// have been checked by the caller against `MIN_ITEM_SAMPLE_SIZE` — this function only guards
/// the literal zero-total case (division by zero), it does not itself enforce the minimum
/// sample size, so the two distinct "not enough data" reasons (too few responses vs. exactly
/// zero) stay visible at the call site.
pub fn calibrate_difficulty(correct: i64, total: i64) -> Option<f64> {
    if total <= 0 {
        return None;
    }
    let p = (correct as f64 / total as f64).clamp(0.02, 0.98);
    Some(-(p / (1.0 - p)).ln())
}

fn probability_correct(theta: f64, b: f64) -> f64 {
    1.0 / (1.0 + (-(theta - b)).exp())
}

/// Newton-Raphson MLE of a single student's ability (theta) under the Rasch model, given their
/// correct/incorrect pattern against each answered item's calibrated difficulty. `responses`
/// entries are `(is_correct, difficulty_b)` — items without a trusted calibration must already
/// be filtered out by the caller before this is called (this function has no way to tell "item
/// wasn't calibrated" apart from "item was genuinely easy/hard"). Returns `None` if fewer than
/// `MIN_CALIBRATED_ITEMS_FOR_SCORE` responses are supplied.
pub fn estimate_theta(responses: &[(bool, f64)]) -> Option<f64> {
    if responses.len() < MIN_CALIBRATED_ITEMS_FOR_SCORE {
        return None;
    }
    let mut theta = 0.0_f64;
    for _ in 0..50 {
        let mut gradient = 0.0_f64;
        let mut curvature = 0.0_f64;
        for &(is_correct, b) in responses {
            let p = probability_correct(theta, b);
            let u = if is_correct { 1.0 } else { 0.0 };
            gradient += u - p;
            curvature += p * (1.0 - p);
        }
        if curvature.abs() < 1e-9 {
            break;
        }
        let step = gradient / curvature;
        theta += step;
        // Clamp mid-iteration too (not just the final result) so a pathological all-correct/
        // all-incorrect pattern can't blow up to +/-infinity across iterations before the final
        // clamp is applied.
        theta = theta.clamp(THETA_MIN - 1.0, THETA_MAX + 1.0);
        if step.abs() < 1e-6 {
            break;
        }
    }
    Some(theta.clamp(THETA_MIN, THETA_MAX))
}

/// Maps theta (logits, roughly -4..4) onto a display scale that sits in the same rough numeric
/// neighborhood as a real UTBK subtest score (typical range ~300-600, national average ~500 —
/// see module doc comment for the source). This is this platform's own disclosed linear
/// transform (mean 500, +/-100 per theta unit), not a claim of matching SNPMB's real scaling
/// constants.
pub fn theta_to_display_score(theta: f64) -> f64 {
    (500.0 + theta * 100.0).clamp(100.0, 900.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibrate_difficulty_handles_extremes_without_infinity() {
        assert!(calibrate_difficulty(100, 100).unwrap().is_finite());
        assert!(calibrate_difficulty(0, 100).unwrap().is_finite());
        assert_eq!(calibrate_difficulty(0, 0), None);
    }

    #[test]
    fn calibrate_difficulty_harder_item_has_higher_b() {
        let easy = calibrate_difficulty(90, 100).unwrap();
        let hard = calibrate_difficulty(10, 100).unwrap();
        assert!(hard > easy);
    }

    #[test]
    fn estimate_theta_none_below_minimum_items() {
        let responses = vec![(true, 0.0); MIN_CALIBRATED_ITEMS_FOR_SCORE - 1];
        assert_eq!(estimate_theta(&responses), None);
    }

    #[test]
    fn estimate_theta_higher_for_more_correct_answers() {
        let difficulties: Vec<f64> = vec![0.0; 10];
        let mostly_correct: Vec<(bool, f64)> = difficulties.iter().enumerate().map(|(i, &b)| (i < 8, b)).collect();
        let mostly_wrong: Vec<(bool, f64)> = difficulties.iter().enumerate().map(|(i, &b)| (i < 2, b)).collect();
        let theta_high = estimate_theta(&mostly_correct).unwrap();
        let theta_low = estimate_theta(&mostly_wrong).unwrap();
        assert!(theta_high > theta_low);
        assert!(theta_high.is_finite() && theta_low.is_finite());
    }

    #[test]
    fn estimate_theta_all_correct_stays_bounded() {
        let responses: Vec<(bool, f64)> = vec![(true, 0.0); 10];
        let theta = estimate_theta(&responses).unwrap();
        assert!(theta <= THETA_MAX);
    }

    #[test]
    fn estimate_theta_all_wrong_stays_bounded() {
        let responses: Vec<(bool, f64)> = vec![(false, 0.0); 10];
        let theta = estimate_theta(&responses).unwrap();
        assert!(theta >= THETA_MIN);
    }

    #[test]
    fn theta_to_display_score_monotonic_and_clamped() {
        assert_eq!(theta_to_display_score(0.0), 500.0);
        assert!(theta_to_display_score(4.0) > theta_to_display_score(0.0));
        assert_eq!(theta_to_display_score(100.0), 900.0);
        assert_eq!(theta_to_display_score(-100.0), 100.0);
    }
}
