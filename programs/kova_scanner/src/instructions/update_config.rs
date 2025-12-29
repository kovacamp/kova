use anchor_lang::prelude::*;

use crate::contexts::UpdateConfig;
use crate::errors::ScannerError;
use crate::state::ScoringWeights;
use crate::utils::validate_weights;

/// Updates one or both scanner configuration parameters.
///