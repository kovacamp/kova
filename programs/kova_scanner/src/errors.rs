use anchor_lang::prelude::*;

#[error_code]
pub enum ScannerError {
    /// Scoring weight total must equal exactly 10000 basis points.
    #[msg("Scoring weights must sum to 10000 basis points")]
    InvalidWeightSum,

    /// Arithmetic overflow during score calculation.
    #[msg("Arithmetic overflow in score calculation")]
    ArithmeticOverflow,
