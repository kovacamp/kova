import { PublicKey } from "@solana/web3.js";

/** Kova Scanner program ID on Solana mainnet and devnet. */
export const KOVA_PROGRAM_ID = new PublicKey(
  "KovA5cAnNeR7xQwK8rY2NjmEv3bUnDL4sHfT9pRs1Wz"
);

/** PDA seed for the global TokenScanConfig account. */
export const SCAN_CONFIG_SEED = Buffer.from("scan_config");

/** PDA seed prefix for ScanRecord accounts. */
export const SCAN_RECORD_SEED = Buffer.from("scan_record");

/** PDA seed prefix for TokenSnapshot accounts. */
export const TOKEN_SNAPSHOT_SEED = Buffer.from("token_snapshot");

/** Basis points denominator (100% = 10000). */
export const BPS_SCALE = 10_000;

/** Maximum valid basis points for a single metric. */
export const MAX_METRIC_BPS = 10_000;

/** Maximum survival score (0-100 scale). */
export const MAX_SCORE = 100;

/** Account discriminator length in bytes (Anchor standard). */
export const DISCRIMINATOR_LENGTH = 8;

/** TokenScanConfig account space in bytes. */
export const SCAN_CONFIG_SPACE = DISCRIMINATOR_LENGTH + 32 + 20 + 8 + 8 + 8 + 8 + 1;

/** TokenSnapshot account space in bytes. */
export const TOKEN_SNAPSHOT_SPACE = DISCRIMINATOR_LENGTH + 32 + 4 + 33 + 8 + 32 + 1;

/** ScanRecord account space in bytes. */
export const SCAN_RECORD_SPACE = DISCRIMINATOR_LENGTH + 32 + 1 + 1 + 2 + 2 + 2 + 2 + 4 + 8 + 8 + 1;

/** Default scoring weights matching the concept document (v1 heuristic). */
export const DEFAULT_SCORING_WEIGHTS = {
  freshWalletWeight: 1500,
  bundlerWeight: 1500,
  top10HolderWeight: 1000,
  smartMoneyWeight: 1000,
  devHoldingsWeight: 1000,
  lpLockedWeight: 800,
  mintRevokedWeight: 800,
  volumeTrendWeight: 700,
  freshSlopeWeight: 850,
  top10SlopeWeight: 850,
} as const;

/** Score tier thresholds. */
export const TIER_THRESHOLDS = {
  CRITICAL_MAX: 19,
  DANGEROUS_MAX: 39,
  CAUTION_MAX: 59,
  MODERATE_MAX: 79,
} as const;

/** Minimum snapshot interval in seconds. */
export const MIN_SNAPSHOT_INTERVAL_SECS = 1;

/** Smart money cap for normalization (20 entries = max sub-score). */
export const SMART_MONEY_CAP = 20;
