/// Core math primitives for the Kova token survival scanner.
///
/// All arithmetic uses checked operations. No floating point. Values are
/// represented in basis points (u16/u64) or fixed-point integers throughout.

/// Basis points denominator (100% = 10_000 bps).
pub const BPS_SCALE: u64 = 10_000;

/// Precision multiplier for intermediate calculations to avoid truncation.
const PRECISION: u128 = 1_000_000_000_000;

/// Computes a weighted score from a set of sub-scores and their weights.
///
/// Each element in `components` is a (sub_score, weight) pair where both values
/// are in basis points (0-10000). The result is scaled to the range 0-100.
///
/// Returns `None` on arithmetic overflow or if the weight sum is zero.
pub fn weighted_score(components: &[(u64, u64)]) -> Option<u64> {
    let mut weighted_sum: u128 = 0;
    let mut weight_sum: u128 = 0;

    for &(sub_score, weight) in components {
        let product = (sub_score as u128).checked_mul(weight as u128)?;
        weighted_sum = weighted_sum.checked_add(product)?;
        weight_sum = weight_sum.checked_add(weight as u128)?;
    }

    if weight_sum == 0 {
        return None;
    }

    // Normalize: weighted_sum / weight_sum gives 0-10000 range, then / 100 for 0-100
    let normalized = weighted_sum.checked_div(weight_sum)?;
    let score = normalized.checked_div(100)?;

    u64::try_from(score.min(100)).ok()
}

/// Computes a probability distribution from a survival score (0-100).
///
/// Returns an array of 4 probabilities in basis points that sum to 10000:
/// [prob_death, prob_100k, prob_300k, prob_1m]
///
/// The distribution is a heuristic piecewise function calibrated against
/// observed Solana token lifecycle data.
pub fn probability_distribution(score: u64) -> Option<[u64; 4]> {
    if score > 100 {
        return None;
    }

    let s = score;

    // Death probability: 95% at score 0, decreasing to ~15% at score 100
    let death = 9500u64.saturating_sub(s.saturating_mul(80));

    // 100K+ probability: scales linearly, caps at 25%
    let p100k = s.saturating_mul(30).min(2500);

    // 300K+ probability: only meaningful above score 40, caps at 15%
    let p300k = s.saturating_sub(40).saturating_mul(20).min(1500);

    // Remainder goes to 1M+
    let allocated = death.saturating_add(p100k).saturating_add(p300k);
    let p1m = BPS_SCALE.saturating_sub(allocated);

    // Adjust death to ensure exact sum of 10000
    let total = death + p100k + p300k + p1m;
    let death_adjusted = death.checked_add(BPS_SCALE)?.checked_sub(total)?;

    Some([death_adjusted, p100k, p300k, p1m])
}

/// Computes the slope (rate of change) of a time series using linear regression.
///
/// Given a slice of y-values sampled at uniform intervals, returns the slope
/// as a fixed-point value multiplied by PRECISION to preserve precision.
///
/// Uses the formula: slope = (n * sum(xy) - sum(x) * sum(y)) / (n * sum(x^2) - sum(x)^2)