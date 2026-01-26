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

/** Builds the initialize instruction. */
export function buildInitializeInstruction(
  authority: PublicKey,
  params: InitializeParams
): TransactionInstruction {
  const [configPda] = deriveConfigPda();

  // 8 (disc) + 20 (weights) + 8 (interval) = 36
  const data = Buffer.alloc(36);
  let offset = 0;

  INITIALIZE_DISCRIMINATOR.copy(data, offset);
  offset += 8;

  encodeWeights(params.scoringWeights).copy(data, offset);
  offset += 20;

  data.writeBigInt64LE(params.minSnapshotIntervalSecs, offset);

  return new TransactionInstruction({
    programId: KOVA_PROGRAM_ID,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: configPda, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });
}

/** Builds the record_snapshot instruction. */
export function buildRecordSnapshotInstruction(
  recorder: PublicKey,
  params: RecordSnapshotParams,
  snapshotIndex: number
): TransactionInstruction {
  const [configPda] = deriveConfigPda();
  const [scanRecordPda] = deriveScanRecordPda(params.tokenMint);
  const [snapshotPda] = deriveSnapshotPda(params.tokenMint, snapshotIndex);

  const m = params.metrics;

  // 8 (disc) + 32 (token_mint) + metrics (2+2+2+2+2+1+1+8+8+4+1 = 33) = 73
  const data = Buffer.alloc(73);
  let offset = 0;

  RECORD_SNAPSHOT_DISCRIMINATOR.copy(data, offset);
  offset += 8;

  params.tokenMint.toBuffer().copy(data, offset);
  offset += 32;

  data.writeUInt16LE(m.freshWalletBps, offset);
  offset += 2;
  data.writeUInt16LE(m.bundlerBps, offset);
  offset += 2;
  data.writeUInt16LE(m.top10HolderBps, offset);
  offset += 2;
  data.writeUInt16LE(m.smartMoneyCount, offset);
  offset += 2;
  data.writeUInt16LE(m.devHoldingsBps, offset);
  offset += 2;
  data.writeUInt8(m.lpLocked ? 1 : 0, offset);
  offset += 1;
  data.writeUInt8(m.mintRevoked ? 1 : 0, offset);
  offset += 1;
  data.writeBigUInt64LE(m.mcapLamports, offset);
  offset += 8;
  data.writeBigUInt64LE(m.volume1mLamports, offset);
  offset += 8;
  data.writeUInt32LE(m.holderCount, offset);
  offset += 4;
  data.writeUInt8(m.volumeTrendUp ? 1 : 0, offset);

  return new TransactionInstruction({
    programId: KOVA_PROGRAM_ID,
    keys: [
      { pubkey: recorder, isSigner: true, isWritable: true },
      { pubkey: configPda, isSigner: false, isWritable: true },
      { pubkey: scanRecordPda, isSigner: false, isWritable: true },
      { pubkey: snapshotPda, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });
}
