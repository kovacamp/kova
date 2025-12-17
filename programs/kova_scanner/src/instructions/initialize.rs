use anchor_lang::prelude::*;

use crate::contexts::Initialize;
use crate::errors::ScannerError;
use crate::state::ScoringWeights;
use crate::utils::validate_weights;

/// Sets up the global TokenScanConfig account with initial scoring weights.
///
/// This must be called exactly once. The signer becomes the scanner authority
/// with exclusive rights to update config and record snapshots.
pub fn handle(
    ctx: Context<Initialize>,
    scoring_weights: ScoringWeights,
    min_snapshot_interval_secs: i64,
) -> Result<()> {
    validate_weights(&scoring_weights)?;
    require!(
        min_snapshot_interval_secs >= 1,
        ScannerError::InvalidSnapshotInterval
    );

    let config = &mut ctx.accounts.scan_config;
    config.authority = ctx.accounts.authority.key();
    config.weights = scoring_weights;
    config.total_snapshots_recorded = 0;
    config.total_scores_calculated = 0;
    config.min_snapshot_interval_secs = min_snapshot_interval_secs;
    config.last_updated_at = Clock::get()?.unix_timestamp;
    config.bump = ctx.bumps.scan_config;

    msg!(
        "Kova Scanner initialized: min_interval={}s, authority={}",
        min_snapshot_interval_secs,
        config.authority,
    );

    Ok(())
}
