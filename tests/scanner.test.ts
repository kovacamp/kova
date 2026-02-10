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
  describe("tierFromScore", () => {
    it("assigns Critical for score 0", () => {
      expect(tierFromScore(0)).toBe(ScoreTier.Critical);
    });

    it("assigns Critical for score 19", () => {
      expect(tierFromScore(19)).toBe(ScoreTier.Critical);
    });

    it("assigns Dangerous at score 20 boundary", () => {
      expect(tierFromScore(20)).toBe(ScoreTier.Dangerous);
    });

    it("assigns Caution at score 40 boundary", () => {
      expect(tierFromScore(40)).toBe(ScoreTier.Caution);
    });

    it("assigns Moderate at score 60 boundary", () => {
      expect(tierFromScore(60)).toBe(ScoreTier.Moderate);
    });

    it("assigns Healthy at score 80 boundary", () => {
      expect(tierFromScore(80)).toBe(ScoreTier.Healthy);
    });

    it("assigns Healthy for score 100", () => {
      expect(tierFromScore(100)).toBe(ScoreTier.Healthy);
    });
  });

  describe("tierLabel", () => {
    it("returns correct labels for all tiers", () => {
      expect(tierLabel(ScoreTier.Critical)).toBe("Critical");
      expect(tierLabel(ScoreTier.Dangerous)).toBe("Dangerous");
      expect(tierLabel(ScoreTier.Caution)).toBe("Caution");
      expect(tierLabel(ScoreTier.Moderate)).toBe("Moderate");
      expect(tierLabel(ScoreTier.Healthy)).toBe("Healthy");
    });
  });
});
