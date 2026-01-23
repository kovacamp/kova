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