use anchor_lang::prelude::*;

use crate::contexts::RecordSnapshot;
use crate::errors::ScannerError;
use crate::state::TokenMetrics;
use crate::utils::{validate_metrics, validate_token_mint};

/// Records a point-in-time token metric snapshot.
///
/// The recorder must be the scanner authority. Each snapshot is stored in a
/// unique PDA derived from the token mint and a sequential index. The scan