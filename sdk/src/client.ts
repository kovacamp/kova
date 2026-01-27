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

/** Error thrown when an RPC call or transaction confirmation fails. */
export class KovaRpcError extends Error {
  public readonly cause: unknown;
  constructor(message: string, cause?: unknown) {
    super(message);
    this.name = "KovaRpcError";
    this.cause = cause;
  }
}

/** Configuration for the KovaClient. */
export interface KovaClientConfig {
  rpcEndpoint: string;
  commitment?: Commitment;
}

/**
 * High-level client for interacting with the Kova Scanner program.
 *
 * Wraps instruction building, transaction assembly, scoring computation,
 * and signal generation into a single ergonomic interface.
 */
export class KovaClient {
  private readonly connection: Connection;
  private readonly commitment: Commitment;

  constructor(config: KovaClientConfig) {
    this.commitment = config.commitment ?? "confirmed";
    this.connection = new Connection(config.rpcEndpoint, this.commitment);
  }

  /** Returns the underlying Solana connection. */
  getConnection(): Connection {
    return this.connection;
  }
