import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  KOVA_PROGRAM_ID,
  SCAN_CONFIG_SEED,
  SCAN_RECORD_SEED,
  TOKEN_SNAPSHOT_SEED,
} from "./constants";
import type {
  InitializeParams,
  RecordSnapshotParams,
  UpdateConfigParams,
  ScoringWeights,
} from "./types";

/** Anchor instruction discriminator: sha256("global:initialize")[0..8] */
const INITIALIZE_DISCRIMINATOR = Buffer.from([
  175, 175, 109, 31, 13, 152, 155, 237,
]);

/** Anchor instruction discriminator: sha256("global:record_snapshot")[0..8] */
const RECORD_SNAPSHOT_DISCRIMINATOR = Buffer.from([
  58, 212, 83, 180, 117, 9, 41, 142,
]);

/** Anchor instruction discriminator: sha256("global:calculate_score")[0..8] */
const CALCULATE_SCORE_DISCRIMINATOR = Buffer.from([
  199, 42, 116, 248, 91, 63, 207, 14,
]);

/** Anchor instruction discriminator: sha256("global:update_config")[0..8] */
const UPDATE_CONFIG_DISCRIMINATOR = Buffer.from([
  29, 158, 252, 191, 10, 83, 219, 99,
]);

/** Derives the TokenScanConfig PDA address. */
export function deriveConfigPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SCAN_CONFIG_SEED],
    KOVA_PROGRAM_ID
  );
}

/** Derives the ScanRecord PDA address for a given token mint. */