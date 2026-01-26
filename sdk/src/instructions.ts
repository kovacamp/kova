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
export function deriveScanRecordPda(
  tokenMint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SCAN_RECORD_SEED, tokenMint.toBuffer()],
    KOVA_PROGRAM_ID
  );
}

/** Derives the TokenSnapshot PDA address. */
export function deriveSnapshotPda(
  tokenMint: PublicKey,
  snapshotIndex: number
): [PublicKey, number] {
  const indexBuffer = Buffer.alloc(4);
  indexBuffer.writeUInt32LE(snapshotIndex, 0);
  return PublicKey.findProgramAddressSync(
    [TOKEN_SNAPSHOT_SEED, tokenMint.toBuffer(), indexBuffer],
    KOVA_PROGRAM_ID
  );
}

/** Encodes ScoringWeights into a buffer (10 x u16 = 20 bytes). */
function encodeWeights(weights: ScoringWeights): Buffer {
  const buf = Buffer.alloc(20);
  buf.writeUInt16LE(weights.freshWalletWeight, 0);
  buf.writeUInt16LE(weights.bundlerWeight, 2);
  buf.writeUInt16LE(weights.top10HolderWeight, 4);
  buf.writeUInt16LE(weights.smartMoneyWeight, 6);
  buf.writeUInt16LE(weights.devHoldingsWeight, 8);
  buf.writeUInt16LE(weights.lpLockedWeight, 10);
  buf.writeUInt16LE(weights.mintRevokedWeight, 12);
  buf.writeUInt16LE(weights.volumeTrendWeight, 14);
  buf.writeUInt16LE(weights.freshSlopeWeight, 16);
  buf.writeUInt16LE(weights.top10SlopeWeight, 18);
  return buf;
}
