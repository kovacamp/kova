use anchor_lang::prelude::*;

use crate::state::{ScanRecord, TokenScanConfig, TokenSnapshot};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = TokenScanConfig::SPACE,
        seeds = [TokenScanConfig::SEED_PREFIX],
        bump,
    )]
    pub scan_config: Account<'info, TokenScanConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(token_mint: Pubkey, metrics: crate::state::TokenMetrics)]
pub struct RecordSnapshot<'info> {
    #[account(mut)]
    pub recorder: Signer<'info>,

    #[account(
        mut,
        seeds = [TokenScanConfig::SEED_PREFIX],
        bump = scan_config.bump,
        constraint = recorder.key() == scan_config.authority @ crate::errors::ScannerError::UnauthorizedAuthority,
    )]
    pub scan_config: Account<'info, TokenScanConfig>,

    /// The scan record for this token. Used to read the current snapshot count
    /// and derive the next snapshot index. Initialized separately or via
    /// calculate_score if it does not exist yet.
    #[account(
        mut,
        seeds = [ScanRecord::SEED_PREFIX, token_mint.as_ref()],
        bump = scan_record.bump,
    )]
    pub scan_record: Account<'info, ScanRecord>,

    #[account(
        init,
        payer = recorder,
        space = TokenSnapshot::SPACE,
        seeds = [
            TokenSnapshot::SEED_PREFIX,
            token_mint.as_ref(),
            &scan_record.snapshots_used.to_le_bytes(),
        ],
        bump,
    )]
    pub token_snapshot: Account<'info, TokenSnapshot>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CalculateScore<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    #[account(
        mut,
        seeds = [TokenScanConfig::SEED_PREFIX],
        bump = scan_config.bump,
    )]
    pub scan_config: Account<'info, TokenScanConfig>,

    #[account(
        mut,
        seeds = [ScanRecord::SEED_PREFIX, scan_record.token_mint.as_ref()],
        bump = scan_record.bump,
    )]
    pub scan_record: Account<'info, ScanRecord>,

    /// The most recent token snapshot. The operator must supply the correct
    /// snapshot account matching the latest index.
    pub latest_snapshot: Account<'info, TokenSnapshot>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        constraint = authority.key() == scan_config.authority @ crate::errors::ScannerError::UnauthorizedAuthority,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [TokenScanConfig::SEED_PREFIX],
        bump = scan_config.bump,
    )]
    pub scan_config: Account<'info, TokenScanConfig>,
}
