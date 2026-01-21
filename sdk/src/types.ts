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