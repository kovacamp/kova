use anchor_lang::prelude::*;

#[error_code]
pub enum ScannerError {
    /// Scoring weight total must equal exactly 10000 basis points.
    #[msg("Scoring weights must sum to 10000 basis points")]
    InvalidWeightSum,

    /// Arithmetic overflow during score calculation.
    #[msg("Arithmetic overflow in score calculation")]
    ArithmeticOverflow,

    /// Metric value exceeds the valid basis points range (0-10000).
    #[msg("Metric value exceeds maximum of 10000 basis points")]
    MetricOutOfRange,

    /// The signer does not match the scanner authority.
    #[msg("Signer is not the scanner authority")]
    UnauthorizedAuthority,

    /// Snapshot interval has not elapsed since the last capture for this token.
    #[msg("Minimum snapshot interval has not elapsed")]
    SnapshotTooFrequent,

    /// No snapshot data exists to compute a score from.
    #[msg("No snapshot data available for score calculation")]
    NoSnapshotData,
