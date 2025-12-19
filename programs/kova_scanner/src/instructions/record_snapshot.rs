use anchor_lang::prelude::*;

use crate::contexts::RecordSnapshot;
use crate::errors::ScannerError;
use crate::state::TokenMetrics;
use crate::utils::{validate_metrics, validate_token_mint};

/// Records a point-in-time token metric snapshot.
///