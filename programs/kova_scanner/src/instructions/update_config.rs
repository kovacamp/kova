use anchor_lang::prelude::*;

use crate::contexts::UpdateConfig;
use crate::errors::ScannerError;
use crate::state::ScoringWeights;
use crate::utils::validate_weights;

/// Updates one or both scanner configuration parameters.
///
/// Only the current authority can call this. Each parameter is optional --
/// passing `None` leaves the existing value unchanged. Weights are validated
/// against the 10000 bps sum requirement before writing.
pub fn handle(
    ctx: Context<UpdateConfig>,
    new_weights: Option<ScoringWeights>,
    new_min_interval: Option<i64>,
) -> Result<()> {
    let config = &mut ctx.accounts.scan_config;

    if let Some(weights) = new_weights {
        validate_weights(&weights)?;
        config.weights = weights;
    }

    if let Some(interval) = new_min_interval {
        require!(interval >= 1, ScannerError::InvalidSnapshotInterval);
        config.min_snapshot_interval_secs = interval;
    }

    config.last_updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "Config updated: min_interval={}s",
        config.min_snapshot_interval_secs,
    );

    Ok(())
}
