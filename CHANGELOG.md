# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-06-15

### Changed

- Pivoted from MEV protection to token survival probability scanning.
- Replaced ShieldConfig, UserProfile, BundleRecord with TokenScanConfig, TokenSnapshot, ScanRecord.
- Replaced five-tier staking system with five-tier scoring system (Critical, Dangerous, Caution, Moderate, Healthy).
- Replaced instructions (initialize, register_user, submit_bundle, update_config, claim_fees) with (initialize, record_snapshot, calculate_score, update_config).
- Rewrote kova-math from AMM/fee primitives to scoring/statistics primitives (weighted_score, probability_distribution, time_series_slope, rate_of_change, exponential_moving_average, z_score_normalize, std_deviation).
- Updated CLI from shield/status/tier commands to scan/monitor/graveyard/stats commands.
- Updated TypeScript SDK types, client, instructions, and constants for scanner context.
- Updated all tests for the new scanner architecture.

## [0.1.0] - 2025-06-01

### Added

- Initial release with MEV protection protocol (now superseded by v0.2.0).
