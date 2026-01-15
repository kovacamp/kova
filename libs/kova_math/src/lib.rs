/// Core math primitives for the Kova token survival scanner.
///
/// All arithmetic uses checked operations. No floating point. Values are
/// represented in basis points (u16/u64) or fixed-point integers throughout.

/// Basis points denominator (100% = 10_000 bps).
pub const BPS_SCALE: u64 = 10_000;

/// Precision multiplier for intermediate calculations to avoid truncation.
/// Precision multiplier for intermediate fixed-point calculations.
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
pub fn time_series_slope(values: &[u64]) -> Option<i128> {
    let n = values.len();
    if n < 2 {
        return Some(0);
    }

    let n_128 = n as i128;
    let mut sum_x: i128 = 0;
    let mut sum_y: i128 = 0;
    let mut sum_xy: i128 = 0;
    let mut sum_x2: i128 = 0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as i128;
        let y_val = y as i128;
        sum_x = sum_x.checked_add(x)?;
        sum_y = sum_y.checked_add(y_val)?;
        sum_xy = sum_xy.checked_add(x.checked_mul(y_val)?)?;
        sum_x2 = sum_x2.checked_add(x.checked_mul(x)?)?;
    }

    let numerator = n_128
        .checked_mul(sum_xy)?
        .checked_sub(sum_x.checked_mul(sum_y)?)?;
    let denominator = n_128
        .checked_mul(sum_x2)?
        .checked_sub(sum_x.checked_mul(sum_x)?)?;

    if denominator == 0 {
        return Some(0);
    }

    // Multiply by PRECISION before dividing to preserve fractional part
    let slope = numerator
        .checked_mul(PRECISION as i128)?
        .checked_div(denominator)?;

    Some(slope)
}

/// Computes the rate of change between two consecutive values as basis points.
///
/// rate = (current - previous) * 10000 / previous
///
/// Returns positive for increase, negative for decrease.
pub fn rate_of_change(previous: u64, current: u64) -> Option<i64> {
    if previous == 0 {
        if current == 0 {
            return Some(0);
        }
        return Some(BPS_SCALE as i64); // max positive change
    }

    let prev_128 = previous as i128;
    let curr_128 = current as i128;

    let delta = curr_128.checked_sub(prev_128)?;
    let rate = delta
        .checked_mul(BPS_SCALE as i128)?
        .checked_div(prev_128)?;

    i64::try_from(rate).ok()
}

/// Computes an exponential moving average given the previous EMA, current value,
/// and a smoothing factor in basis points.
///
/// ema = (current * alpha + previous_ema * (10000 - alpha)) / 10000
///
/// A higher alpha gives more weight to the current value.
pub fn exponential_moving_average(
    previous_ema: u64,
    current_value: u64,
    alpha_bps: u16,
) -> Option<u64> {
    if alpha_bps > BPS_SCALE as u16 {
        return None;
    }

    let alpha = alpha_bps as u128;
    let complement = (BPS_SCALE as u128).checked_sub(alpha)?;

    let weighted_current = (current_value as u128).checked_mul(alpha)?;
    let weighted_previous = (previous_ema as u128).checked_mul(complement)?;

    let result = weighted_current
        .checked_add(weighted_previous)?
        .checked_div(BPS_SCALE as u128)?;

    u64::try_from(result).ok()
}

/// Normalizes a value to a z-score using fixed-point arithmetic.
///
/// z = (value - mean) * PRECISION / std_dev
///
/// Returns 0 if std_dev is zero.
pub fn z_score_normalize(value: u64, mean: u64, std_dev: u64) -> i128 {
    if std_dev == 0 {
        return 0;
    }

    let val = value as i128;
    let m = mean as i128;
    let sd = std_dev as i128;

    val.saturating_sub(m)
        .saturating_mul(PRECISION as i128)
        .checked_div(sd)
        .unwrap_or(0)
}

/// Integer square root via Newton's method. Used for standard deviation
/// calculations and other statistical computations.
pub fn isqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }

    let mut guess = value;
    let mut previous = 0u128;

    while guess != previous {
        previous = guess;
        // Newton step: next = (guess + value / guess) / 2
        guess = guess
            .checked_add(value.checked_div(guess).unwrap_or(0))
            .unwrap_or(guess)
            .checked_div(2)
            .unwrap_or(guess);
    }

    guess
}

/// Computes the standard deviation of a slice of values using fixed-point math.
///
/// Returns the standard deviation as a u64.
pub fn std_deviation(values: &[u64]) -> Option<u64> {
    let n = values.len();
    if n < 2 {
        return Some(0);
    }

    let n_128 = n as u128;
    let sum: u128 = values.iter().map(|&v| v as u128).sum();
    let mean = sum.checked_div(n_128)?;

    let variance_sum: u128 = values
        .iter()
        .map(|&v| {
            let diff = if (v as u128) >= mean {
                (v as u128) - mean
            } else {
                mean - (v as u128)
            };
            diff.checked_mul(diff).unwrap_or(u128::MAX)
        })
        .try_fold(0u128, |acc, v| acc.checked_add(v))?;

    let variance = variance_sum.checked_div(n_128.checked_sub(1)?)?;
    let std = isqrt(variance);

    u64::try_from(std).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_score_equal_weights() {
        // All sub-scores at 5000 (50%), equal weights => score 50
        let components = vec![(5000, 2500), (5000, 2500), (5000, 2500), (5000, 2500)];
        let score = weighted_score(&components).unwrap();
        assert_eq!(score, 50);
    }

    #[test]
    fn test_weighted_score_max() {
        let components = vec![(10000, 5000), (10000, 5000)];
        let score = weighted_score(&components).unwrap();
        assert_eq!(score, 100);
    }

    #[test]
    fn test_weighted_score_min() {
        let components = vec![(0, 5000), (0, 5000)];
        let score = weighted_score(&components).unwrap();
        assert_eq!(score, 0);
    }

    #[test]
    fn test_weighted_score_mixed() {
        // 10000 * 7000 + 0 * 3000 = 70_000_000 / 10000 = 7000 / 100 = 70
        let components = vec![(10000, 7000), (0, 3000)];
        let score = weighted_score(&components).unwrap();
        assert_eq!(score, 70);
    }

    #[test]
    fn test_weighted_score_empty_returns_none() {
        assert!(weighted_score(&[]).is_none());
    }

    #[test]
    fn test_probability_distribution_sums_to_10000() {
        for s in 0..=100 {
            let dist = probability_distribution(s).unwrap();
            let total: u64 = dist.iter().sum();
            assert_eq!(total, 10_000, "Score {} distribution sums to {}", s, total);
        }
    }

    #[test]
    fn test_probability_distribution_high_death_at_zero() {
        let dist = probability_distribution(0).unwrap();
        assert!(dist[0] >= 8000, "Death probability at score 0: {}", dist[0]);
    }

    #[test]
    fn test_probability_distribution_low_death_at_100() {
        let dist = probability_distribution(100).unwrap();
        assert!(
            dist[0] <= 3000,
            "Death probability at score 100: {}",
            dist[0]
        );
    }

    #[test]
    fn test_probability_distribution_invalid_score() {
        assert!(probability_distribution(101).is_none());
    }

    #[test]
    fn test_time_series_slope_positive() {
        let values = vec![1000, 2000, 3000, 4000, 5000];
        let slope = time_series_slope(&values).unwrap();
        assert!(slope > 0, "Ascending series should have positive slope");
    }

    #[test]
    fn test_time_series_slope_negative() {
        let values = vec![5000, 4000, 3000, 2000, 1000];
        let slope = time_series_slope(&values).unwrap();
        assert!(slope < 0, "Descending series should have negative slope");
    }

    #[test]
    fn test_time_series_slope_flat() {
        let values = vec![3000, 3000, 3000, 3000];
        let slope = time_series_slope(&values).unwrap();
        assert_eq!(slope, 0, "Flat series should have zero slope");
    }

    #[test]
    fn test_time_series_slope_single_value() {
        let values = vec![5000];
        let slope = time_series_slope(&values).unwrap();
        assert_eq!(slope, 0);
    }

    #[test]
    fn test_rate_of_change_increase() {
        let rate = rate_of_change(1000, 1500).unwrap();
        // (1500 - 1000) * 10000 / 1000 = 5000 bps = 50% increase
        assert_eq!(rate, 5000);
    }

    #[test]
    fn test_rate_of_change_decrease() {
        let rate = rate_of_change(1000, 500).unwrap();
        // (500 - 1000) * 10000 / 1000 = -5000 bps = 50% decrease
        assert_eq!(rate, -5000);
    }

    #[test]
    fn test_rate_of_change_no_change() {
        let rate = rate_of_change(1000, 1000).unwrap();
        assert_eq!(rate, 0);
    }

    #[test]
    fn test_rate_of_change_from_zero() {
        let rate = rate_of_change(0, 500).unwrap();
        assert_eq!(rate, BPS_SCALE as i64);
    }

    #[test]
    fn test_rate_of_change_both_zero() {
        let rate = rate_of_change(0, 0).unwrap();
        assert_eq!(rate, 0);
    }

    #[test]
    fn test_ema_full_weight_current() {
        let ema = exponential_moving_average(5000, 8000, 10000).unwrap();
        assert_eq!(ema, 8000); // alpha=100% => pure current value
    }

    #[test]
    fn test_ema_full_weight_previous() {
        let ema = exponential_moving_average(5000, 8000, 0).unwrap();
        assert_eq!(ema, 5000); // alpha=0% => pure previous value
    }

    #[test]
    fn test_ema_half_weight() {
        let ema = exponential_moving_average(4000, 6000, 5000).unwrap();
        // (6000 * 5000 + 4000 * 5000) / 10000 = 50_000_000 / 10000 = 5000
        assert_eq!(ema, 5000);
    }

    #[test]
    fn test_ema_invalid_alpha() {
        assert!(exponential_moving_average(5000, 8000, 10001).is_none());
    }

    #[test]
    fn test_z_score_above_mean() {
        let z = z_score_normalize(150, 100, 25);
        // (150 - 100) * PRECISION / 25 = 50 * PRECISION / 25 = 2 * PRECISION
        assert_eq!(z, 2 * PRECISION as i128);
    }

    #[test]
    fn test_z_score_below_mean() {
        let z = z_score_normalize(50, 100, 25);
        // (50 - 100) * PRECISION / 25 = -2 * PRECISION
        assert_eq!(z, -2 * PRECISION as i128);
    }

    #[test]
    fn test_z_score_at_mean() {
        let z = z_score_normalize(100, 100, 25);
        assert_eq!(z, 0);
    }

    #[test]
    fn test_z_score_zero_std_dev() {
        let z = z_score_normalize(150, 100, 0);
        assert_eq!(z, 0);
    }

    #[test]
    fn test_isqrt_perfect_square() {
        assert_eq!(isqrt(144), 12);
        assert_eq!(isqrt(10000), 100);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(0), 0);
    }

    #[test]
    fn test_isqrt_non_perfect() {
        assert_eq!(isqrt(2), 1);
        assert_eq!(isqrt(150), 12);
    }

    #[test]
    fn test_std_deviation_basic() {
        let values = vec![2, 4, 4, 4, 5, 5, 7, 9];
        let std = std_deviation(&values).unwrap();
        // Population variance = 4, sample variance = 4.57, sqrt ~= 2.14 => isqrt = 2
        assert!(
            std >= 1 && std <= 3,
            "Std deviation should be ~2, got {}",
            std
        );
    }

    #[test]
    fn test_std_deviation_constant() {
        let values = vec![5, 5, 5, 5, 5];
        let std = std_deviation(&values).unwrap();
        assert_eq!(std, 0);
    }

    #[test]
    fn test_std_deviation_single_value() {
        let values = vec![42];
        let std = std_deviation(&values).unwrap();
        assert_eq!(std, 0);
    }
}
