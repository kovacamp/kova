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