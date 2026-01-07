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
