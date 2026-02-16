use anchor_lang::prelude::*;

#[error_code]
pub enum ScannerError {
    /// Scoring weights total must equal exactly 10000 basis points.
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

    /// Token mint address cannot be the system program or default pubkey.
    #[msg("Invalid token mint address")]
    InvalidTokenMint,

    /// Minimum snapshot interval must be positive.
    #[msg("Snapshot interval must be at least 1 second")]
    InvalidSnapshotInterval,

    /// Snapshot index exceeds the maximum allowed per token.
    #[msg("Snapshot index overflow")]
    SnapshotIndexOverflow,

    /// Boolean metric field must be 0 or 1.
    #[msg("Boolean metric must be 0 or 1")]
    InvalidBooleanMetric,
}
