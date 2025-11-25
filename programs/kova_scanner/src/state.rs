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