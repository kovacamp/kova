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

  /** Initializes the scanner config. Must be called once by the authority. */
  async initialize(
    authority: Keypair,
    params: InitializeParams
  ): Promise<string> {
    if (!validateWeights(params.scoringWeights)) {
      throw new KovaValidationError("Scoring weights must sum to 10000 bps");
    }
    if (params.minSnapshotIntervalSecs < BigInt(MIN_SNAPSHOT_INTERVAL_SECS)) {
      throw new KovaValidationError(
        `Snapshot interval must be at least ${MIN_SNAPSHOT_INTERVAL_SECS} second`
      );
    }

    const instruction = buildInitializeInstruction(
      authority.publicKey,
      params
    );
    return this.sendTransaction([authority], instruction);
  }

  /** Records a token metric snapshot. Authority only. */
  async recordSnapshot(
    recorder: Keypair,
    params: RecordSnapshotParams,
    snapshotIndex: number
  ): Promise<string> {
    if (!isValidTokenMetrics(params.metrics)) {
      throw new KovaValidationError("Invalid token metrics");
    }

    const instruction = buildRecordSnapshotInstruction(
      recorder.publicKey,
      params,
      snapshotIndex
    );
    return this.sendTransaction([recorder], instruction);
  }

  /** Triggers score calculation for a token. */
  async calculateScore(
    operator: Keypair,
    tokenMint: PublicKey,
    latestSnapshotIndex: number
  ): Promise<string> {
    const instruction = buildCalculateScoreInstruction(
      operator.publicKey,
      tokenMint,
      latestSnapshotIndex
    );
    return this.sendTransaction([operator], instruction);
  }

  /** Updates the scanner config. Authority only. */
  async updateConfig(
    authority: Keypair,
    params: UpdateConfigParams
  ): Promise<string> {
    if (params.newWeights !== null && !validateWeights(params.newWeights)) {
      throw new KovaValidationError("Scoring weights must sum to 10000 bps");
    }
    if (
      params.newMinInterval !== null &&
      params.newMinInterval < BigInt(MIN_SNAPSHOT_INTERVAL_SECS)
    ) {
      throw new KovaValidationError(
        `Snapshot interval must be at least ${MIN_SNAPSHOT_INTERVAL_SECS} second`
      );
    }

    const instruction = buildUpdateConfigInstruction(
      authority.publicKey,
      params
    );
    return this.sendTransaction([authority], instruction);
  }

  /** Fetches and deserializes the TokenScanConfig account. */
  async fetchConfig(): Promise<TokenScanConfig | null> {
    const [configPda] = deriveConfigPda();
    const accountInfo = await this.connection.getAccountInfo(configPda);
    if (!accountInfo) return null;

    return this.deserializeConfig(accountInfo.data);
  }

  /** Fetches and deserializes a ScanRecord account. */
  async fetchScanRecord(tokenMint: PublicKey): Promise<ScoreResult | null> {
    const [recordPda] = deriveScanRecordPda(tokenMint);
    const accountInfo = await this.connection.getAccountInfo(recordPda);
    if (!accountInfo) return null;

    return this.deserializeScanRecord(accountInfo.data);
  }

  /**
   * Computes a survival score from token metrics using the weighted scoring
   * algorithm. Does not require an on-chain transaction -- runs entirely
   * client-side for quick estimations.
   */
  computeScore(
    metrics: TokenMetrics,
    weights: ScoringWeights = DEFAULT_SCORING_WEIGHTS
  ): { score: number; tier: ScoreTier; distribution: ProbabilityDistribution } {
    const freshSub = BPS_SCALE - metrics.freshWalletBps;
    const bundlerSub = BPS_SCALE - metrics.bundlerBps;
    const top10Sub = BPS_SCALE - metrics.top10HolderBps;
    const devSub = BPS_SCALE - metrics.devHoldingsBps;
    const smartMoneySub = Math.min(metrics.smartMoneyCount, SMART_MONEY_CAP)
      * (BPS_SCALE / SMART_MONEY_CAP);
    const lpSub = metrics.lpLocked ? BPS_SCALE : 0;
    const mintSub = metrics.mintRevoked ? BPS_SCALE : 0;
    const volumeSub = metrics.volumeTrendUp ? BPS_SCALE : 0;

    const weightedSum =
      freshSub * weights.freshWalletWeight +
      bundlerSub * weights.bundlerWeight +
      top10Sub * weights.top10HolderWeight +
      smartMoneySub * weights.smartMoneyWeight +
      devSub * weights.devHoldingsWeight +
      lpSub * weights.lpLockedWeight +
      mintSub * weights.mintRevokedWeight +
      volumeSub * weights.volumeTrendWeight +
      freshSub * weights.freshSlopeWeight +
      top10Sub * weights.top10SlopeWeight;

    const scoreRaw = Math.floor(weightedSum / BPS_SCALE / 100);
    const score = Math.min(scoreRaw, 100);
    const tier = tierFromScore(score);
    const distribution = this.computeDistribution(score);

    return { score, tier, distribution };
  }
