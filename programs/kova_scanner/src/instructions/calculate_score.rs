use anchor_lang::prelude::*;

use crate::contexts::CalculateScore;
use crate::errors::ScannerError;
use crate::state::ScoreTier;
use crate::utils::{compute_probability_distribution, compute_weighted_score};

/// Calculates the survival probability score from the latest snapshot.
///