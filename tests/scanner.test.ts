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

describe("Type Guards", () => {
  describe("isScoreTier", () => {
    it("accepts valid tier values", () => {
      expect(isScoreTier(0)).toBe(true);
      expect(isScoreTier(1)).toBe(true);
      expect(isScoreTier(2)).toBe(true);
      expect(isScoreTier(3)).toBe(true);
      expect(isScoreTier(4)).toBe(true);
    });

    it("rejects out-of-range numbers", () => {
      expect(isScoreTier(-1)).toBe(false);
      expect(isScoreTier(5)).toBe(false);
      expect(isScoreTier(100)).toBe(false);
    });

    it("rejects non-number types", () => {
      expect(isScoreTier("Critical")).toBe(false);
      expect(isScoreTier(null)).toBe(false);
      expect(isScoreTier(undefined)).toBe(false);
    });

    it("rejects floating-point numbers", () => {
      expect(isScoreTier(1.5)).toBe(false);
    });
  });

  describe("isValidTokenMetrics", () => {
    const validMetrics = {
      freshWalletBps: 5400,
      bundlerBps: 2800,
      top10HolderBps: 4300,
      smartMoneyCount: 2,
      devHoldingsBps: 800,
      lpLocked: false,
      mintRevoked: true,
      mcapLamports: 50_000_000_000n,
      volume1mLamports: 1_000_000_000n,
      holderCount: 150,
      volumeTrendUp: true,
    };

    it("accepts valid metrics", () => {
      expect(isValidTokenMetrics(validMetrics)).toBe(true);
    });

    it("rejects metrics with out-of-range bps", () => {
      expect(
        isValidTokenMetrics({ ...validMetrics, freshWalletBps: 10001 })
      ).toBe(false);
    });

    it("rejects metrics with wrong boolean type", () => {
      expect(
        isValidTokenMetrics({ ...validMetrics, lpLocked: 1 })
      ).toBe(false);
    });

    it("rejects null", () => {
      expect(isValidTokenMetrics(null)).toBe(false);
    });

    it("rejects missing fields", () => {
      expect(isValidTokenMetrics({ freshWalletBps: 5000 })).toBe(false);
    });
  });
});

describe("Scoring Weights Validation", () => {
  it("validates default weights sum to 10000", () => {
    expect(validateWeights(DEFAULT_SCORING_WEIGHTS)).toBe(true);
  });

  it("rejects weights that do not sum to 10000", () => {
    const badWeights = { ...DEFAULT_SCORING_WEIGHTS, freshWalletWeight: 9999 };
    expect(validateWeights(badWeights)).toBe(false);
  });

  it("rejects weights that sum to more than 10000", () => {
    const over = { ...DEFAULT_SCORING_WEIGHTS, freshWalletWeight: 2000 };
    expect(validateWeights(over)).toBe(false);
  });
});

describe("PDA Derivation", () => {
  it("derives deterministic config PDA", () => {
    const [pda1, bump1] = deriveConfigPda();
    const [pda2, bump2] = deriveConfigPda();

    expect(pda1.equals(pda2)).toBe(true);
    expect(bump1).toBe(bump2);
  });

  it("derives different scan record PDAs for different tokens", () => {
    const mintA = Keypair.generate().publicKey;
    const mintB = Keypair.generate().publicKey;

    const [pdaA] = deriveScanRecordPda(mintA);
    const [pdaB] = deriveScanRecordPda(mintB);

    expect(pdaA.equals(pdaB)).toBe(false);
  });

  it("derives different snapshot PDAs for different indices", () => {
    const mint = Keypair.generate().publicKey;

    const [pdaA] = deriveSnapshotPda(mint, 0);
    const [pdaB] = deriveSnapshotPda(mint, 1);

    expect(pdaA.equals(pdaB)).toBe(false);
  });

  it("derives different snapshot PDAs for different tokens at same index", () => {
    const mintA = Keypair.generate().publicKey;
    const mintB = Keypair.generate().publicKey;

    const [pdaA] = deriveSnapshotPda(mintA, 0);
    const [pdaB] = deriveSnapshotPda(mintB, 0);

    expect(pdaA.equals(pdaB)).toBe(false);
  });
});

describe("Constants", () => {
  it("program ID is a valid base58 public key", () => {
    expect(() => new PublicKey(KOVA_PROGRAM_ID.toBase58())).not.toThrow();
  });

  it("BPS_SCALE is 10000", () => {
    expect(BPS_SCALE).toBe(10_000);
  });

  it("account space calculations are positive", () => {
    expect(SCAN_CONFIG_SPACE).toBeGreaterThan(8);
    expect(SCAN_RECORD_SPACE).toBeGreaterThan(8);
    expect(TOKEN_SNAPSHOT_SPACE).toBeGreaterThan(8);
  });

  it("tier thresholds are in ascending order", () => {
    expect(TIER_THRESHOLDS.CRITICAL_MAX).toBeLessThan(TIER_THRESHOLDS.DANGEROUS_MAX);
    expect(TIER_THRESHOLDS.DANGEROUS_MAX).toBeLessThan(TIER_THRESHOLDS.CAUTION_MAX);
    expect(TIER_THRESHOLDS.CAUTION_MAX).toBeLessThan(TIER_THRESHOLDS.MODERATE_MAX);
  });
});

describe("KovaValidationError", () => {
  it("has the correct error name", () => {
    const err = new KovaValidationError("test");
    expect(err.name).toBe("KovaValidationError");
    expect(err.message).toBe("test");
  });

  it("is an instance of Error", () => {
    const err = new KovaValidationError("test");
    expect(err).toBeInstanceOf(Error);
  });
});
