use anchor_lang::prelude::*;

use crate::contexts::Initialize;
use crate::errors::ScannerError;
use crate::state::ScoringWeights;
use crate::utils::validate_weights;

/// Sets up the global TokenScanConfig account with initial scoring weights.
///