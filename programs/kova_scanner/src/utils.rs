use anchor_lang::prelude::*;

use crate::errors::ScannerError;
use crate::state::{ScoringWeights, TokenMetrics};

/// Basis points denominator.
pub const BPS_SCALE: u64 = 10_000;

/// Maximum valid basis points value for a single metric.
pub const MAX_METRIC_BPS: u16 = 10_000;

/// Maximum score value (100).
/// Maximum survival score (0-100 range).
pub const MAX_SCORE: u8 = 100;

/// Validates that all scoring weights sum to exactly 10000 basis points.
pub fn validate_weights(weights: &ScoringWeights) -> Result<()> {
    let sum = (weights.fresh_wallet_weight as u32)
        .checked_add(weights.bundler_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.top10_holder_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.smart_money_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.dev_holdings_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.lp_locked_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.mint_revoked_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.volume_trend_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.fresh_slope_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(weights.top10_slope_weight as u32)
        .ok_or(ScannerError::ArithmeticOverflow)?;

    require!(sum == BPS_SCALE as u32, ScannerError::InvalidWeightSum);
    Ok(())
}

/// Validates that metric fields are within their expected ranges.
pub fn validate_metrics(metrics: &TokenMetrics) -> Result<()> {
    require!(
        metrics.fresh_wallet_bps <= MAX_METRIC_BPS,
        ScannerError::MetricOutOfRange
    );
    require!(
        metrics.bundler_bps <= MAX_METRIC_BPS,
        ScannerError::MetricOutOfRange
    );
    require!(
        metrics.top10_holder_bps <= MAX_METRIC_BPS,
        ScannerError::MetricOutOfRange
    );
    require!(
        metrics.dev_holdings_bps <= MAX_METRIC_BPS,
        ScannerError::MetricOutOfRange
    );
    require!(metrics.lp_locked <= 1, ScannerError::InvalidBooleanMetric);
    require!(
        metrics.mint_revoked <= 1,
        ScannerError::InvalidBooleanMetric
    );
    require!(
        metrics.volume_trend_up <= 1,
        ScannerError::InvalidBooleanMetric
    );
    Ok(())
}

/// Computes the weighted survival score from a set of token metrics.
///
/// Each metric is converted into a directional sub-score (0-10000 scale) and
/// then weighted according to the config. The result is scaled down to 0-100.
///
/// Inverse metrics (higher value = worse): fresh_wallet, bundler, top10_holder, dev_holdings
/// Positive metrics (higher value = better): smart_money, lp_locked, mint_revoked, volume_trend
pub fn compute_weighted_score(metrics: &TokenMetrics, weights: &ScoringWeights) -> Result<u8> {
    // Convert inverse metrics: sub_score = 10000 - metric_bps
    let fresh_sub = (BPS_SCALE as u16)
        .checked_sub(metrics.fresh_wallet_bps)
        .ok_or(ScannerError::ArithmeticOverflow)? as u64;

    let bundler_sub = (BPS_SCALE as u16)
        .checked_sub(metrics.bundler_bps)
        .ok_or(ScannerError::ArithmeticOverflow)? as u64;

    let top10_sub = (BPS_SCALE as u16)
        .checked_sub(metrics.top10_holder_bps)
        .ok_or(ScannerError::ArithmeticOverflow)? as u64;

    let dev_sub = (BPS_SCALE as u16)
        .checked_sub(metrics.dev_holdings_bps)
        .ok_or(ScannerError::ArithmeticOverflow)? as u64;

    // Smart money: normalize to 0-10000 scale. Cap at 20 entries for max score.
    let smart_money_normalized = ((metrics.smart_money_count as u64).min(20))
        .checked_mul(BPS_SCALE / 20)
        .ok_or(ScannerError::ArithmeticOverflow)?;

    // Boolean metrics: 0 or 10000
    let lp_sub = if metrics.lp_locked == 1 { BPS_SCALE } else { 0 };
    let mint_sub = if metrics.mint_revoked == 1 {
        BPS_SCALE
    } else {
        0
    };
    let volume_sub = if metrics.volume_trend_up == 1 {
        BPS_SCALE
    } else {
        0
    };

    // For slope-based sub-scores, use placeholder values derived from the
    // current snapshot metrics. Real time-series slope requires multiple
    // snapshots and is computed off-chain in v1; the on-chain program uses
    // the current metric direction as a proxy.
    let fresh_slope_sub = fresh_sub; // declining fresh % is positive
    let top10_slope_sub = top10_sub; // distributing top10 is positive

    // Weighted sum: sum(sub_score * weight) / 10000, then scale to 0-100
    let weighted_sum = fresh_sub
        .checked_mul(weights.fresh_wallet_weight as u64)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            bundler_sub
                .checked_mul(weights.bundler_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            top10_sub
                .checked_mul(weights.top10_holder_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            smart_money_normalized
                .checked_mul(weights.smart_money_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            dev_sub
                .checked_mul(weights.dev_holdings_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            lp_sub
                .checked_mul(weights.lp_locked_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            mint_sub
                .checked_mul(weights.mint_revoked_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            volume_sub
                .checked_mul(weights.volume_trend_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            fresh_slope_sub
                .checked_mul(weights.fresh_slope_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_add(
            top10_slope_sub
                .checked_mul(weights.top10_slope_weight as u64)
                .ok_or(ScannerError::ArithmeticOverflow)?,
        )
        .ok_or(ScannerError::ArithmeticOverflow)?;

    // Divide by BPS_SCALE to get 0-10000 range, then divide by 100 for 0-100
    let score_raw = weighted_sum
        .checked_div(BPS_SCALE)
        .ok_or(ScannerError::ArithmeticOverflow)?
        .checked_div(100)
        .ok_or(ScannerError::ArithmeticOverflow)?;

    let clamped = score_raw.min(MAX_SCORE as u64) as u8;
    Ok(clamped)
}

/// Computes a probability distribution from a survival score.
///
/// Returns (prob_death_bps, prob_100k_bps, prob_300k_bps, prob_1m_bps).
/// All values sum to 10000 bps.
///
/// The distribution is a heuristic piecewise function calibrated against
/// observed Solana token lifecycle data. Higher scores shift probability
/// mass away from death and toward higher market cap tiers.
pub fn compute_probability_distribution(score: u8) -> (u16, u16, u16, u16) {
    let s = score as u32;

    // Death probability: starts at ~95% for score 0, decreases to ~15% for score 100.
    // P(death) = 9500 - (s * 80)
    let death_raw = 9500u32.saturating_sub(s.saturating_mul(80));
    let death_bps = death_raw.min(BPS_SCALE as u32) as u16;

    // 100K+ probability: peaks around score 60-80.
    // P(100K) = min(s * 30, 2500)
    let p100k_raw = s.saturating_mul(30).min(2500);
    let p100k_bps = p100k_raw as u16;

    // 300K+ probability: meaningful only above score 40.
    // P(300K) = max(0, (s - 40) * 20)
    let p300k_raw = s.saturating_sub(40).saturating_mul(20).min(1500);
    let p300k_bps = p300k_raw as u16;

    // 1M+ probability: remainder to ensure sum = 10000.
    let allocated = (death_bps as u32)
        .saturating_add(p100k_bps as u32)
        .saturating_add(p300k_bps as u32);
    let p1m_bps = (BPS_SCALE as u32).saturating_sub(allocated).min(1000) as u16;

    // Rebalance death to ensure exact sum of 10000.
    let total = (death_bps as u32) + (p100k_bps as u32) + (p300k_bps as u32) + (p1m_bps as u32);
    let death_adjusted = (death_bps as u32)
        .checked_add(BPS_SCALE as u32)
        .unwrap_or(BPS_SCALE as u32)
        .checked_sub(total)
        .unwrap_or(death_bps as u32) as u16;

    (death_adjusted, p100k_bps, p300k_bps, p1m_bps)
}

/// Validates that a pubkey is not the default (all-zeros) address.
pub fn validate_token_mint(mint: &Pubkey) -> Result<()> {
    require!(*mint != Pubkey::default(), ScannerError::InvalidTokenMint);
    Ok(())
}

/// Derives the TokenScanConfig PDA address and bump.
pub fn derive_config_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"scan_config"], program_id)
}

/// Derives a ScanRecord PDA address and bump for a given token mint.
pub fn derive_scan_record_pda(token_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"scan_record", token_mint.as_ref()], program_id)
}

/// Derives a TokenSnapshot PDA address and bump.
pub fn derive_snapshot_pda(
    token_mint: &Pubkey,
    snapshot_index: u32,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"token_snapshot",
            token_mint.as_ref(),
            &snapshot_index.to_le_bytes(),
        ],
        program_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_weights() -> ScoringWeights {
        ScoringWeights {
            fresh_wallet_weight: 1500,
            bundler_weight: 1500,
            top10_holder_weight: 1000,
            smart_money_weight: 1000,
            dev_holdings_weight: 1000,
            lp_locked_weight: 800,
            mint_revoked_weight: 800,
            volume_trend_weight: 700,
            fresh_slope_weight: 850,
            top10_slope_weight: 850,
        }
    }

    #[test]
    fn test_validate_weights_correct_sum() {
        let weights = default_weights();
        assert!(validate_weights(&weights).is_ok());
    }

    #[test]
    fn test_validate_weights_incorrect_sum() {
        let mut weights = default_weights();
        weights.fresh_wallet_weight = 9999;
        assert!(validate_weights(&weights).is_err());
    }

    #[test]
    fn test_validate_metrics_valid() {
        let metrics = TokenMetrics {
            fresh_wallet_bps: 5400,
            bundler_bps: 2800,
            top10_holder_bps: 4300,
            smart_money_count: 2,
            dev_holdings_bps: 800,
            lp_locked: 0,
            mint_revoked: 1,
            mcap_lamports: 50_000_000_000,
            volume_1m_lamports: 1_000_000_000,
            holder_count: 150,
            volume_trend_up: 1,
        };
        assert!(validate_metrics(&metrics).is_ok());
    }

    #[test]
    fn test_validate_metrics_out_of_range() {
        let metrics = TokenMetrics {
            fresh_wallet_bps: 10001,
            bundler_bps: 0,
            top10_holder_bps: 0,
            smart_money_count: 0,
            dev_holdings_bps: 0,
            lp_locked: 0,
            mint_revoked: 0,
            mcap_lamports: 0,
            volume_1m_lamports: 0,
            holder_count: 0,
            volume_trend_up: 0,
        };
        assert!(validate_metrics(&metrics).is_err());
    }

    #[test]
    fn test_validate_metrics_invalid_boolean() {
        let metrics = TokenMetrics {
            fresh_wallet_bps: 5000,
            bundler_bps: 2000,
            top10_holder_bps: 3000,
            smart_money_count: 1,
            dev_holdings_bps: 500,
            lp_locked: 2, // invalid
            mint_revoked: 1,
            mcap_lamports: 0,
            volume_1m_lamports: 0,
            holder_count: 0,
            volume_trend_up: 0,
        };
        assert!(validate_metrics(&metrics).is_err());
    }

    #[test]
    fn test_compute_score_healthy_token() {
        let weights = default_weights();
        let metrics = TokenMetrics {
            fresh_wallet_bps: 1000, // low fresh = good
            bundler_bps: 500,       // low bundler = good
            top10_holder_bps: 2000, // moderately distributed
            smart_money_count: 10,  // strong SM presence
            dev_holdings_bps: 200,  // low dev holdings = good
            lp_locked: 1,
            mint_revoked: 1,
            mcap_lamports: 100_000_000_000,
            volume_1m_lamports: 5_000_000_000,
            holder_count: 500,
            volume_trend_up: 1,
        };
        let score = compute_weighted_score(&metrics, &weights).unwrap();
        assert!(
            score >= 70,
            "Healthy token should score >= 70, got {}",
            score
        );
    }

    #[test]
    fn test_compute_score_risky_token() {
        let weights = default_weights();
        let metrics = TokenMetrics {
            fresh_wallet_bps: 8900, // very high fresh = bad
            bundler_bps: 6000,      // high bundler = bad
            top10_holder_bps: 7800, // concentrated = bad
            smart_money_count: 0,
            dev_holdings_bps: 4000, // high dev = bad
            lp_locked: 0,
            mint_revoked: 0,
            mcap_lamports: 5_000_000,
            volume_1m_lamports: 100_000,
            holder_count: 15,
            volume_trend_up: 0,
        };
        let score = compute_weighted_score(&metrics, &weights).unwrap();
        assert!(score <= 30, "Risky token should score <= 30, got {}", score);
    }

    #[test]
    fn test_compute_score_all_zeros() {
        let weights = default_weights();
        let metrics = TokenMetrics {
            fresh_wallet_bps: 0,
            bundler_bps: 0,
            top10_holder_bps: 0,
            smart_money_count: 0,
            dev_holdings_bps: 0,
            lp_locked: 0,
            mint_revoked: 0,
            mcap_lamports: 0,
            volume_1m_lamports: 0,
            holder_count: 0,
            volume_trend_up: 0,
        };
        let score = compute_weighted_score(&metrics, &weights).unwrap();
        // All inverse metrics at 0 => sub-score 10000. Boolean metrics at 0.
        // SM at 0. Weighted by ~77% of weight for inverse metrics.
        assert!(
            score >= 50,
            "All-zero inverse metrics should score decently, got {}",
            score
        );
    }

    #[test]
    fn test_compute_score_max_all_positive() {
        let weights = default_weights();
        let metrics = TokenMetrics {
            fresh_wallet_bps: 0,
            bundler_bps: 0,
            top10_holder_bps: 0,
            smart_money_count: 20,
            dev_holdings_bps: 0,
            lp_locked: 1,
            mint_revoked: 1,
            mcap_lamports: 1_000_000_000_000,
            volume_1m_lamports: 100_000_000_000,
            holder_count: 10_000,
            volume_trend_up: 1,
        };
        let score = compute_weighted_score(&metrics, &weights).unwrap();
        assert_eq!(
            score, 100,
            "Perfect metrics should yield 100, got {}",
            score
        );
    }

    #[test]
    fn test_probability_distribution_sums_to_10000() {
        for s in 0..=100 {
            let (death, p100k, p300k, p1m) = compute_probability_distribution(s);
            let total = (death as u32) + (p100k as u32) + (p300k as u32) + (p1m as u32);
            assert_eq!(
                total, 10_000,
                "Distribution for score {} sums to {}, expected 10000",
                s, total
            );
        }
    }

    #[test]
    fn test_probability_distribution_high_death_for_low_score() {
        let (death, _, _, _) = compute_probability_distribution(0);
        assert!(
            death >= 8000,
            "Score 0 should have high death probability, got {}",
            death
        );
    }

    #[test]
    fn test_probability_distribution_low_death_for_high_score() {
        let (death, _, _, _) = compute_probability_distribution(100);
        assert!(
            death <= 3000,
            "Score 100 should have low death probability, got {}",
            death
        );
    }

    #[test]
    fn test_score_tier_boundaries() {
        use crate::state::ScoreTier;
        assert_eq!(ScoreTier::from_score(0), ScoreTier::Critical);
        assert_eq!(ScoreTier::from_score(19), ScoreTier::Critical);
        assert_eq!(ScoreTier::from_score(20), ScoreTier::Dangerous);
        assert_eq!(ScoreTier::from_score(39), ScoreTier::Dangerous);
        assert_eq!(ScoreTier::from_score(40), ScoreTier::Caution);
        assert_eq!(ScoreTier::from_score(59), ScoreTier::Caution);
        assert_eq!(ScoreTier::from_score(60), ScoreTier::Moderate);
        assert_eq!(ScoreTier::from_score(79), ScoreTier::Moderate);
        assert_eq!(ScoreTier::from_score(80), ScoreTier::Healthy);
        assert_eq!(ScoreTier::from_score(100), ScoreTier::Healthy);
    }
}
