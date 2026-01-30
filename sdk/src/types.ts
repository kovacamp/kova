import { PublicKey } from "@solana/web3.js";

/** Score tier classifications for token survival assessment. */
export enum ScoreTier {
  Critical = 0,
  Dangerous = 1,
  Caution = 2,
  Moderate = 3,
  Healthy = 4,
}

/** Raw token metrics collected from on-chain and off-chain data sources. */
export interface TokenMetrics {
  /** Fresh wallet percentage in basis points (0-10000). */
  freshWalletBps: number;
  /** Bundler percentage in basis points (0-10000). */
  bundlerBps: number;
  /** Top 10 holder concentration in basis points (0-10000). */
  top10HolderBps: number;
  /** Number of smart money wallets that entered. */
  smartMoneyCount: number;
  /** Developer holdings percentage in basis points (0-10000). */
  devHoldingsBps: number;
  /** Whether LP is burned or locked. */
  lpLocked: boolean;
  /** Whether mint authority is revoked. */
  mintRevoked: boolean;
  /** Market cap in SOL lamports. */
  mcapLamports: bigint;
  /** 1-minute volume in SOL lamports. */
  volume1mLamports: bigint;
  /** Total unique holder count. */
  holderCount: number;
  /** Whether volume trend is increasing. */
  volumeTrendUp: boolean;
}

/** Result of a token survival probability scan. */
export interface ScoreResult {
  /** Token contract address. */
  tokenMint: PublicKey;
  /** Survival probability score (0-100). */
  score: number;
  /** Assigned score tier. */
  tier: ScoreTier;
  /** Probability distribution across outcome buckets. */
  distribution: ProbabilityDistribution;
  /** Number of snapshots used. */
  snapshotsUsed: number;
  /** Unix timestamp of the latest snapshot. */
  latestSnapshotAt: bigint;
  /** Unix timestamp of when this score was computed. */
  scoredAt: bigint;
}

/** Probability distribution across token outcome categories. */
export interface ProbabilityDistribution {
  /** Probability of death before graduation (bps). */
  deathBps: number;
  /** Probability of reaching 100K+ market cap (bps). */
  reach100kBps: number;
  /** Probability of reaching 300K+ as a runner (bps). */
  reach300kBps: number;
  /** Probability of reaching 1M+ mega run (bps). */
  reach1mBps: number;
}

/** A signal extracted from metric analysis (warning or positive indicator). */
export interface Signal {
  /** Signal type: "warning" for red flags, "positive" for green flags. */
  type: "warning" | "positive";
  /** Human-readable description of the signal. */
  message: string;
  /** The metric key that triggered this signal. */
  metricKey: string;
  /** The metric value that triggered this signal. */
  metricValue: number;
}

/** A single point in a time-series data feed. */
export interface TimeSeriesPoint {
  /** Unix timestamp in milliseconds. */
  timestamp: number;
  /** The metric value at this timestamp. */
  value: number;
}

/** Scoring weights configuration matching the on-chain ScoringWeights struct. */
export interface ScoringWeights {
  freshWalletWeight: number;
  bundlerWeight: number;
  top10HolderWeight: number;
  smartMoneyWeight: number;
  devHoldingsWeight: number;
  lpLockedWeight: number;
  mintRevokedWeight: number;
  volumeTrendWeight: number;
  freshSlopeWeight: number;
  top10SlopeWeight: number;
}

/** Deserialized TokenScanConfig account data. */
export interface TokenScanConfig {
  authority: PublicKey;
  weights: ScoringWeights;
  totalSnapshotsRecorded: bigint;
  totalScoresCalculated: bigint;
  minSnapshotIntervalSecs: bigint;
  lastUpdatedAt: bigint;
  bump: number;
}

/** Parameters for the initialize instruction. */
export interface InitializeParams {
  scoringWeights: ScoringWeights;
  minSnapshotIntervalSecs: bigint;
}

/** Parameters for the record_snapshot instruction. */
export interface RecordSnapshotParams {
  tokenMint: PublicKey;
  metrics: TokenMetrics;
}

/** Parameters for the update_config instruction. */
export interface UpdateConfigParams {
  newWeights: ScoringWeights | null;
  newMinInterval: bigint | null;
}

/** Type guard: checks if a value is a valid ScoreTier. */
export function isScoreTier(value: unknown): value is ScoreTier {
  return (
    typeof value === "number" &&
    Number.isInteger(value) &&
    value >= ScoreTier.Critical &&
    value <= ScoreTier.Healthy
  );
}

/** Type guard: validates a TokenMetrics object has all required fields in range. */
export function isValidTokenMetrics(value: unknown): value is TokenMetrics {
  if (typeof value !== "object" || value === null) return false;

  const m = value as Record<string, unknown>;

  if (typeof m.freshWalletBps !== "number" || m.freshWalletBps < 0 || m.freshWalletBps > 10000)
    return false;
  if (typeof m.bundlerBps !== "number" || m.bundlerBps < 0 || m.bundlerBps > 10000) return false;
  if (typeof m.top10HolderBps !== "number" || m.top10HolderBps < 0 || m.top10HolderBps > 10000)
    return false;
  if (typeof m.smartMoneyCount !== "number" || m.smartMoneyCount < 0) return false;
  if (typeof m.devHoldingsBps !== "number" || m.devHoldingsBps < 0 || m.devHoldingsBps > 10000)
    return false;
  if (typeof m.lpLocked !== "boolean") return false;
  if (typeof m.mintRevoked !== "boolean") return false;
  if (typeof m.holderCount !== "number" || m.holderCount < 0) return false;
  if (typeof m.volumeTrendUp !== "boolean") return false;

  return true;
}

/** Determines the ScoreTier for a given score value (0-100). */
export function tierFromScore(score: number): ScoreTier {
  if (score < 20) return ScoreTier.Critical;
  if (score < 40) return ScoreTier.Dangerous;
  if (score < 60) return ScoreTier.Caution;
  if (score < 80) return ScoreTier.Moderate;
  return ScoreTier.Healthy;
}

/** Returns a human-readable label for a ScoreTier. */
export function tierLabel(tier: ScoreTier): string {
  switch (tier) {
    case ScoreTier.Critical:
      return "Critical";
    case ScoreTier.Dangerous:
      return "Dangerous";
    case ScoreTier.Caution:
      return "Caution";
    case ScoreTier.Moderate:
      return "Moderate";
    case ScoreTier.Healthy:
      return "Healthy";
  }
}

/** Validates that scoring weights sum to exactly 10000. */
export function validateWeights(weights: ScoringWeights): boolean {
  const sum =
    weights.freshWalletWeight +
    weights.bundlerWeight +
    weights.top10HolderWeight +
    weights.smartMoneyWeight +
    weights.devHoldingsWeight +
    weights.lpLockedWeight +
    weights.mintRevokedWeight +
    weights.volumeTrendWeight +
    weights.freshSlopeWeight +
    weights.top10SlopeWeight;

  return sum === 10_000;
}
