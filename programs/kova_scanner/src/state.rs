use anchor_lang::prelude::*;

/// Score tier thresholds for token survival classification.
/// Tier assignment is based on the computed survival probability score.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScoreTier {
    /// Score 0-19. Extremely high risk; likely rug or instant death.
    Critical,
    /// Score 20-39. High risk with multiple red flags.
    Dangerous,
    /// Score 40-59. Mixed signals; proceed with caution.
    Caution,
    /// Score 60-79. Moderately healthy indicators.
    Moderate,
    /// Score 80-100. Strong survival signals across all metrics.
    Healthy,
}

impl ScoreTier {
    /// Determines the tier from a score value (0-100).
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=19 => ScoreTier::Critical,
            20..=39 => ScoreTier::Dangerous,
            40..=59 => ScoreTier::Caution,
            60..=79 => ScoreTier::Moderate,
            _ => ScoreTier::Healthy,
        }
    }

    /// Returns a human-readable label for this tier.
    pub fn label(&self) -> &'static str {
        match self {
            ScoreTier::Critical => "Critical",
            ScoreTier::Dangerous => "Dangerous",
            ScoreTier::Caution => "Caution",
            ScoreTier::Moderate => "Moderate",
            ScoreTier::Healthy => "Healthy",
        }
    }
}

/// Weights applied to each metric dimension when computing the aggregate score.
/// All weights are in basis points (0-10000). Total should sum to 10000.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScoringWeights {
    /// Weight for fresh wallet percentage (inverse -- higher fresh = lower score).
    pub fresh_wallet_weight: u16,
    /// Weight for bundler percentage (inverse).
    pub bundler_weight: u16,
    /// Weight for top 10 holder concentration (inverse).
    pub top10_holder_weight: u16,
    /// Weight for smart money entry count (positive).
    pub smart_money_weight: u16,
    /// Weight for developer holdings percentage (inverse).
    pub dev_holdings_weight: u16,
    /// Weight for LP burn/lock status (positive).
    pub lp_locked_weight: u16,
    /// Weight for mint authority revocation (positive).
    pub mint_revoked_weight: u16,
    /// Weight for volume trend direction (positive).
    pub volume_trend_weight: u16,
    /// Weight for time-series slope of fresh wallet % (positive = declining).
    pub fresh_slope_weight: u16,
    /// Weight for time-series slope of top10 % (positive = distributing).
    pub top10_slope_weight: u16,
}

/// Raw token metrics collected from on-chain and off-chain data sources.
/// Stored as basis points (0-10000) where applicable, or raw counts.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct TokenMetrics {
    /// Fresh wallet percentage in basis points (0-10000).
    pub fresh_wallet_bps: u16,
    /// Bundler percentage in basis points (0-10000).
    pub bundler_bps: u16,
    /// Top 10 holder concentration in basis points (0-10000).
    pub top10_holder_bps: u16,
    /// Number of smart money wallets that entered.
    pub smart_money_count: u16,
    /// Developer holdings percentage in basis points (0-10000).
    pub dev_holdings_bps: u16,
    /// Whether LP is burned or locked (1 = yes, 0 = no).
    pub lp_locked: u8,
    /// Whether mint authority is revoked (1 = yes, 0 = no).
    pub mint_revoked: u8,
    /// Market cap in SOL lamports.
    pub mcap_lamports: u64,
    /// 1-minute volume in SOL lamports.
    pub volume_1m_lamports: u64,
    /// Total unique holder count.
    pub holder_count: u32,
    /// Volume trend signal: 1 = increasing, 0 = flat/decreasing.
    pub volume_trend_up: u8,
}

/// Global scanner configuration. Singleton PDA derived from ["scan_config"].
#[account]
pub struct TokenScanConfig {
    /// The authority pubkey that can update config and submit snapshots.
    pub authority: Pubkey,
    /// Scoring weights for each metric dimension.
    pub weights: ScoringWeights,
    /// Total number of snapshots recorded across all tokens.
    pub total_snapshots_recorded: u64,
    /// Total number of score calculations performed.
    pub total_scores_calculated: u64,
    /// Minimum interval between snapshots for the same token (seconds).
    pub min_snapshot_interval_secs: i64,
    /// Unix timestamp of the last config update.
    pub last_updated_at: i64,
    /// PDA bump seed for address derivation.
    pub bump: u8,
}

impl TokenScanConfig {
    /// Account discriminator (8) + pubkey (32) + ScoringWeights (10 * 2 = 20)
    /// + u64 (8) + u64 (8) + i64 (8) + i64 (8) + u8 (1)
    pub const SPACE: usize = 8 + 32 + 20 + 8 + 8 + 8 + 8 + 1;
    pub const SEED_PREFIX: &'static [u8] = b"scan_config";
}

/// Per-token snapshot storing a single point-in-time metric capture.
/// PDA derived from ["token_snapshot", token_mint, snapshot_index].
#[account]
pub struct TokenSnapshot {
    /// The token mint address this snapshot belongs to.
    pub token_mint: Pubkey,
    /// Sequential index for this token's snapshots (0-based).
    pub snapshot_index: u32,
    /// The captured metrics at this point in time.
    pub metrics: TokenMetrics,
    /// Unix timestamp when this snapshot was captured.
    pub captured_at: i64,
    /// The recorder (operator) who submitted this snapshot.
    pub recorder: Pubkey,
    /// PDA bump seed.
    pub bump: u8,
}

impl TokenSnapshot {
    /// 8 + 32 + 4 + TokenMetrics (2+2+2+2+2+1+1+8+8+4+1 = 33) + 8 + 32 + 1
    pub const SPACE: usize = 8 + 32 + 4 + 33 + 8 + 32 + 1;
    pub const SEED_PREFIX: &'static [u8] = b"token_snapshot";
}

/// Aggregated scan record containing the computed score and probability distribution.
/// PDA derived from ["scan_record", token_mint].
#[account]
pub struct ScanRecord {
    /// The token mint address this record belongs to.
    pub token_mint: Pubkey,
    /// Computed survival probability score (0-100).
    pub score: u8,
    /// Assigned score tier based on the computed score.
    pub tier: ScoreTier,
    /// Probability of death before graduation, in basis points (0-10000).
    pub prob_death_bps: u16,
    /// Probability of reaching 100K+ market cap, in basis points.
    pub prob_100k_bps: u16,
    /// Probability of reaching 300K+ (runner), in basis points.
    pub prob_300k_bps: u16,
    /// Probability of reaching 1M+ (mega run), in basis points.
    pub prob_1m_bps: u16,
    /// Number of snapshots used to compute this score.
    pub snapshots_used: u32,
    /// Unix timestamp of the most recent snapshot included.
    pub latest_snapshot_at: i64,
    /// Unix timestamp of when this score was computed.
    pub scored_at: i64,
    /// PDA bump seed.
    pub bump: u8,
}

impl ScanRecord {
    /// 8 + 32 + 1 + 1 (tier) + 2 + 2 + 2 + 2 + 4 + 8 + 8 + 1
    pub const SPACE: usize = 8 + 32 + 1 + 1 + 2 + 2 + 2 + 2 + 4 + 8 + 8 + 1;
    pub const SEED_PREFIX: &'static [u8] = b"scan_record";
}
// TODO: add Display impl for ScoreTier
