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