import { Keypair, PublicKey } from "@solana/web3.js";
import {
  ScoreTier,
  tierFromScore,
  tierLabel,
  isScoreTier,
  isValidTokenMetrics,
  validateWeights,
  KOVA_PROGRAM_ID,
  BPS_SCALE,
  SCAN_CONFIG_SPACE,
  SCAN_RECORD_SPACE,
  TOKEN_SNAPSHOT_SPACE,
  DEFAULT_SCORING_WEIGHTS,
  TIER_THRESHOLDS,
} from "../sdk/src";
import {
  deriveConfigPda,
  deriveScanRecordPda,
  deriveSnapshotPda,
} from "../sdk/src/instructions";
import { KovaValidationError } from "../sdk/src/client";

describe("ScoreTier", () => {