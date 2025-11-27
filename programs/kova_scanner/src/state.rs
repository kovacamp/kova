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