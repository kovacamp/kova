use anchor_lang::prelude::*;

use crate::contexts::RecordSnapshot;
use crate::errors::ScannerError;
use crate::state::TokenMetrics;
use crate::utils::{validate_metrics, validate_token_mint};

/// Records a point-in-time token metric snapshot.
///
/// The recorder must be the scanner authority. Each snapshot is stored in a
/// unique PDA derived from the token mint and a sequential index. The scan
/// record's snapshot counter is incremented to track the next available index.
pub fn handle(
    ctx: Context<RecordSnapshot>,
    token_mint: Pubkey,
    metrics: TokenMetrics,
) -> Result<()> {
    validate_token_mint(&token_mint)?;
    validate_metrics(&metrics)?;

    let clock = Clock::get()?;
    let scan_record = &ctx.accounts.scan_record;

    // Enforce minimum snapshot interval if this is not the first snapshot.
    if scan_record.snapshots_used > 0 {
        let elapsed = clock
            .unix_timestamp
            .checked_sub(scan_record.latest_snapshot_at)
            .ok_or(ScannerError::ArithmeticOverflow)?;
        require!(
            elapsed >= ctx.accounts.scan_config.min_snapshot_interval_secs,
            ScannerError::SnapshotTooFrequent
        );
    }

    let snapshot = &mut ctx.accounts.token_snapshot;
    snapshot.token_mint = token_mint;
    snapshot.snapshot_index = scan_record.snapshots_used;
    snapshot.metrics = metrics;
    snapshot.captured_at = clock.unix_timestamp;
    snapshot.recorder = ctx.accounts.recorder.key();
    snapshot.bump = ctx.bumps.token_snapshot;

    // Update scan record counters
    let scan_record = &mut ctx.accounts.scan_record;
    scan_record.snapshots_used = scan_record
        .snapshots_used
        .checked_add(1)
        .ok_or(ScannerError::SnapshotIndexOverflow)?;
    scan_record.latest_snapshot_at = clock.unix_timestamp;

    // Update global counter
    let config = &mut ctx.accounts.scan_config;
    config.total_snapshots_recorded = config
        .total_snapshots_recorded
        .checked_add(1)
        .ok_or(ScannerError::ArithmeticOverflow)?;

    msg!(
        "Snapshot #{} recorded for token {} at slot {}",
        snapshot.snapshot_index,
        token_mint,
        clock.slot
    );

    Ok(())
}
