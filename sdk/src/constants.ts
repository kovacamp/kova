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