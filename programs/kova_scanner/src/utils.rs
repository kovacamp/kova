use anchor_lang::prelude::*;

use crate::errors::ScannerError;
use crate::state::{ScoringWeights, TokenMetrics};

/// Basis points denominator.
pub const BPS_SCALE: u64 = 10_000;

/// Maximum valid basis points value for a single metric.
pub const MAX_METRIC_BPS: u16 = 10_000;

/// Maximum score value (100).
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