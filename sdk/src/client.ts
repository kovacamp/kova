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

  /** Generates signals (warnings and positive indicators) from token metrics. */
  generateSignals(metrics: TokenMetrics): Signal[] {
    const signals: Signal[] = [];

    if (metrics.freshWalletBps > 5000) {
      signals.push({
        type: "warning",
        message: `Fresh wallet ${(metrics.freshWalletBps / 100).toFixed(0)}%`,
        metricKey: "freshWalletBps",
        metricValue: metrics.freshWalletBps,
      });
    }

    if (metrics.bundlerBps > 2000) {
      signals.push({
        type: "warning",
        message: `Bundler ratio ${(metrics.bundlerBps / 100).toFixed(0)}%`,
        metricKey: "bundlerBps",
        metricValue: metrics.bundlerBps,
      });
    }

    if (metrics.top10HolderBps > 5000) {
      signals.push({
        type: "warning",
        message: `Top 10 holds ${(metrics.top10HolderBps / 100).toFixed(0)}%`,
        metricKey: "top10HolderBps",
        metricValue: metrics.top10HolderBps,
      });
    }

    if (metrics.devHoldingsBps > 500) {
      signals.push({
        type: "warning",
        message: `Dev still holds ${(metrics.devHoldingsBps / 100).toFixed(1)}%`,
        metricKey: "devHoldingsBps",
        metricValue: metrics.devHoldingsBps,
      });
    }

    if (metrics.smartMoneyCount > 0) {
      signals.push({
        type: "positive",
        message: `Smart money: ${metrics.smartMoneyCount} entered`,
        metricKey: "smartMoneyCount",
        metricValue: metrics.smartMoneyCount,
      });
    }

    if (metrics.lpLocked) {
      signals.push({
        type: "positive",
        message: "LP burned/locked",
        metricKey: "lpLocked",
        metricValue: 1,
      });
    }

    if (metrics.mintRevoked) {
      signals.push({
        type: "positive",
        message: "Mint authority revoked",
        metricKey: "mintRevoked",
        metricValue: 1,
      });
    }

    if (!metrics.mintRevoked) {
      signals.push({
        type: "warning",
        message: "Mint authority NOT revoked",
        metricKey: "mintRevoked",
        metricValue: 0,
      });
    }

    if (metrics.volumeTrendUp) {
      signals.push({
        type: "positive",
        message: "Volume trending up",
        metricKey: "volumeTrendUp",
        metricValue: 1,
      });
    }

    return signals;
  }

  /** Computes the probability distribution for a given score. */
  private computeDistribution(score: number): ProbabilityDistribution {
    const s = score;
    const deathRaw = Math.max(0, 9500 - s * 80);
    const reach100kRaw = Math.min(s * 30, 2500);
    const reach300kRaw = Math.min(Math.max(0, (s - 40) * 20), 1500);
    const allocated = deathRaw + reach100kRaw + reach300kRaw;
    const reach1mRaw = BPS_SCALE - allocated;
    const total = deathRaw + reach100kRaw + reach300kRaw + reach1mRaw;
    const deathBps = deathRaw + (BPS_SCALE - total);

    return {
      deathBps,
      reach100kBps: reach100kRaw,
      reach300kBps: reach300kRaw,
      reach1mBps: reach1mRaw,
    };
  }

  /** Assembles, signs, and sends a transaction with a single instruction. */
  private async sendTransaction(
    signers: Keypair[],
    ...instructions: TransactionInstruction[]
  ): Promise<string> {
    const transaction = new Transaction();
    for (const ix of instructions) {
      transaction.add(ix);
    }

    try {
      const signature = await sendAndConfirmTransaction(
        this.connection,
        transaction,
        signers,
        { commitment: this.commitment }
      );
      return signature;
    } catch (err) {
      throw new KovaRpcError("Transaction failed", err);
    }
  }

  /** Deserializes raw account data into a TokenScanConfig. */
  private deserializeConfig(data: Buffer): TokenScanConfig {
    let offset = 8; // skip discriminator

    const authority = new PublicKey(data.subarray(offset, offset + 32));
    offset += 32;

    const weights: ScoringWeights = {
      freshWalletWeight: data.readUInt16LE(offset),
      bundlerWeight: data.readUInt16LE(offset + 2),
      top10HolderWeight: data.readUInt16LE(offset + 4),
      smartMoneyWeight: data.readUInt16LE(offset + 6),
      devHoldingsWeight: data.readUInt16LE(offset + 8),
      lpLockedWeight: data.readUInt16LE(offset + 10),
      mintRevokedWeight: data.readUInt16LE(offset + 12),
      volumeTrendWeight: data.readUInt16LE(offset + 14),
      freshSlopeWeight: data.readUInt16LE(offset + 16),
      top10SlopeWeight: data.readUInt16LE(offset + 18),
    };
    offset += 20;

    const totalSnapshotsRecorded = data.readBigUInt64LE(offset);
    offset += 8;

    const totalScoresCalculated = data.readBigUInt64LE(offset);
    offset += 8;

    const minSnapshotIntervalSecs = data.readBigInt64LE(offset);
    offset += 8;

    const lastUpdatedAt = data.readBigInt64LE(offset);
    offset += 8;

    const bump = data.readUInt8(offset);

    return {
      authority,
      weights,
      totalSnapshotsRecorded,
      totalScoresCalculated,
      minSnapshotIntervalSecs,
      lastUpdatedAt,
      bump,
    };
  }

  /** Deserializes raw account data into a ScoreResult. */
  private deserializeScanRecord(data: Buffer): ScoreResult {
    let offset = 8; // skip discriminator

    const tokenMint = new PublicKey(data.subarray(offset, offset + 32));
    offset += 32;

    const score = data.readUInt8(offset);
    offset += 1;

    const tierByte = data.readUInt8(offset);
    const tier: ScoreTier = tierByte;
    offset += 1;

    const probDeathBps = data.readUInt16LE(offset);
    offset += 2;

    const prob100kBps = data.readUInt16LE(offset);
    offset += 2;

    const prob300kBps = data.readUInt16LE(offset);
    offset += 2;

    const prob1mBps = data.readUInt16LE(offset);
    offset += 2;

    const snapshotsUsed = data.readUInt32LE(offset);
    offset += 4;

    const latestSnapshotAt = data.readBigInt64LE(offset);
    offset += 8;

    const scoredAt = data.readBigInt64LE(offset);

    return {
      tokenMint,
      score,
      tier,
      distribution: {
        deathBps: probDeathBps,
        reach100kBps: prob100kBps,
        reach300kBps: prob300kBps,
        reach1mBps: prob1mBps,
      },
      snapshotsUsed,
      latestSnapshotAt,
      scoredAt,
    };
  }
}
