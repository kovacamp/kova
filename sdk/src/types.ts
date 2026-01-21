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