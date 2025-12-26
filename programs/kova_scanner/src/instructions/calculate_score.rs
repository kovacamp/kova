use anchor_lang::prelude::*;

use crate::contexts::CalculateScore;
use crate::errors::ScannerError;
use crate::state::ScoreTier;
use crate::utils::{compute_probability_distribution, compute_weighted_score};

/// Calculates the survival probability score from the latest snapshot.
///
/// Reads the most recent TokenSnapshot for the target token, applies the
/// weighted scoring algorithm using the current config weights, computes
/// the probability distribution, and writes the results to the ScanRecord.
pub fn handle(ctx: Context<CalculateScore>) -> Result<()> {
    let snapshot = &ctx.accounts.latest_snapshot;
    let config = &ctx.accounts.scan_config;
    let scan_record = &ctx.accounts.scan_record;

    // Verify the snapshot belongs to the same token as the scan record.
    require!(
        snapshot.token_mint == scan_record.token_mint,
        ScannerError::InvalidTokenMint
    );

    // Verify snapshot data exists.
    require!(
        scan_record.snapshots_used > 0,
        ScannerError::NoSnapshotData
    );

    let score = compute_weighted_score(&snapshot.metrics, &config.weights)?;
    let tier = ScoreTier::from_score(score);
    let (prob_death, prob_100k, prob_300k, prob_1m) = compute_probability_distribution(score);

    let clock = Clock::get()?;

    let record = &mut ctx.accounts.scan_record;
    record.score = score;
    record.tier = tier;
    record.prob_death_bps = prob_death;
    record.prob_100k_bps = prob_100k;
    record.prob_300k_bps = prob_300k;
    record.prob_1m_bps = prob_1m;
    record.scored_at = clock.unix_timestamp;

    // Update global counter
    let config = &mut ctx.accounts.scan_config;
    config.total_scores_calculated = config
        .total_scores_calculated
        .checked_add(1)
        .ok_or(ScannerError::ArithmeticOverflow)?;

    msg!(
        "Score calculated: {} ({}) for token {}",
        score,
        tier.label(),
        record.token_mint
    );

    Ok(())
}
