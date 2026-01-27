import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  type Commitment,
} from "@solana/web3.js";
import {
  BPS_SCALE,
  DEFAULT_SCORING_WEIGHTS,
  SMART_MONEY_CAP,
  MIN_SNAPSHOT_INTERVAL_SECS,
} from "./constants";
import {
  buildInitializeInstruction,
  buildRecordSnapshotInstruction,
  buildCalculateScoreInstruction,
  buildUpdateConfigInstruction,
  deriveConfigPda,
  deriveScanRecordPda,
} from "./instructions";
import type {
  InitializeParams,
  RecordSnapshotParams,
  UpdateConfigParams,
  TokenScanConfig,
  ScoreResult,
  TokenMetrics,
  ProbabilityDistribution,
  Signal,
  ScoringWeights,
} from "./types";
import {
  ScoreTier,
  tierFromScore,
  isValidTokenMetrics,
  validateWeights,
} from "./types";

/** Error thrown when SDK validation fails before sending a transaction. */
export class KovaValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "KovaValidationError";
  }
}
