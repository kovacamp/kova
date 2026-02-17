import { Keypair, PublicKey } from "@solana/web3.js";
import {
  KovaClient,
  KovaValidationError,
  ScoreTier,
  BPS_SCALE,
  DEFAULT_SCORING_WEIGHTS,
} from "../sdk/src";
import {
  buildInitializeInstruction,
  buildRecordSnapshotInstruction,
  buildCalculateScoreInstruction,
  buildUpdateConfigInstruction,
} from "../sdk/src/instructions";
import type { TokenMetrics } from "../sdk/src";

describe("Instruction Builders", () => {
  const authority = Keypair.generate();
  const tokenMint = Keypair.generate().publicKey;

  describe("buildInitializeInstruction", () => {
    it("produces an instruction with correct account count", () => {
      const ix = buildInitializeInstruction(authority.publicKey, {
        scoringWeights: DEFAULT_SCORING_WEIGHTS,
        minSnapshotIntervalSecs: 5n,
      });

      expect(ix.keys).toHaveLength(3);
      expect(ix.keys[0].pubkey.equals(authority.publicKey)).toBe(true);
      expect(ix.keys[0].isSigner).toBe(true);
      expect(ix.keys[0].isWritable).toBe(true);
    });

    it("encodes weights and interval into instruction data", () => {
      const ix = buildInitializeInstruction(authority.publicKey, {
        scoringWeights: DEFAULT_SCORING_WEIGHTS,
        minSnapshotIntervalSecs: 10n,
      });

      // Data layout: 8 (disc) + 20 (weights) + 8 (interval) = 36 bytes
      expect(ix.data.length).toBe(36);

      // Read first weight at offset 8
      const freshWeight = ix.data.readUInt16LE(8);
      expect(freshWeight).toBe(DEFAULT_SCORING_WEIGHTS.freshWalletWeight);

      // Read interval at offset 28
      const interval = ix.data.readBigInt64LE(28);
      expect(interval).toBe(10n);
    });
  });

  describe("buildRecordSnapshotInstruction", () => {
    const metrics: TokenMetrics = {
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

    it("produces an instruction with 5 accounts", () => {
      const ix = buildRecordSnapshotInstruction(
        authority.publicKey,
        { tokenMint, metrics },
        0
      );

      expect(ix.keys).toHaveLength(5);
    });

    it("encodes token mint and metrics into instruction data", () => {
      const ix = buildRecordSnapshotInstruction(
        authority.publicKey,
        { tokenMint, metrics },
        0
      );

      // 8 (disc) + 32 (mint) + 33 (metrics) = 73
      expect(ix.data.length).toBe(73);

      // Read mint at offset 8
      const mintBytes = ix.data.subarray(8, 40);
      expect(new PublicKey(mintBytes).equals(tokenMint)).toBe(true);

      // Read freshWalletBps at offset 40
      const freshBps = ix.data.readUInt16LE(40);
      expect(freshBps).toBe(5400);

      // Read lpLocked boolean at offset 50
      expect(ix.data.readUInt8(50)).toBe(0); // false

      // Read mintRevoked boolean at offset 51
      expect(ix.data.readUInt8(51)).toBe(1); // true
    });
  });

  describe("buildCalculateScoreInstruction", () => {
    it("produces an instruction with 4 accounts", () => {
      const ix = buildCalculateScoreInstruction(
        authority.publicKey,
        tokenMint,
        0
      );

      expect(ix.keys).toHaveLength(4);
    });

    it("has only the discriminator in data (8 bytes)", () => {
      const ix = buildCalculateScoreInstruction(
        authority.publicKey,
        tokenMint,
        0
      );
      expect(ix.data.length).toBe(8);
    });
  });

  describe("buildUpdateConfigInstruction", () => {
    it("encodes Some weights correctly", () => {
      const ix = buildUpdateConfigInstruction(authority.publicKey, {
        newWeights: DEFAULT_SCORING_WEIGHTS,
        newMinInterval: 15n,
      });

      expect(ix.keys).toHaveLength(2);
      // 8 (disc) + 1 + 20 (weights) + 1 + 8 (interval) = 38
      expect(ix.data.length).toBe(38);
    });

    it("encodes None values with zero option tags", () => {
      const ix = buildUpdateConfigInstruction(authority.publicKey, {
        newWeights: null,
        newMinInterval: null,
      });

      // Option tags at offsets 8 and 29 should be 0
      expect(ix.data.readUInt8(8)).toBe(0);
      expect(ix.data.readUInt8(29)).toBe(0);
    });
  });
});

describe("KovaClient Score Computation", () => {
  const client = new KovaClient({
    rpcEndpoint: "https://api.devnet.solana.com",
  });

  it("computes a high score for healthy metrics", () => {
    const metrics: TokenMetrics = {
      freshWalletBps: 1000,
      bundlerBps: 500,
      top10HolderBps: 2000,
      smartMoneyCount: 10,
      devHoldingsBps: 200,
      lpLocked: true,
      mintRevoked: true,
      mcapLamports: 100_000_000_000n,
      volume1mLamports: 5_000_000_000n,
      holderCount: 500,
      volumeTrendUp: true,
    };

    const result = client.computeScore(metrics);
    expect(result.score).toBeGreaterThanOrEqual(70);
    expect(result.tier).toBe(ScoreTier.Healthy);
  });

  it("computes a low score for risky metrics", () => {
    const metrics: TokenMetrics = {
      freshWalletBps: 8900,
      bundlerBps: 6000,
      top10HolderBps: 7800,
      smartMoneyCount: 0,
      devHoldingsBps: 4000,
      lpLocked: false,
      mintRevoked: false,
      mcapLamports: 5_000_000n,
      volume1mLamports: 100_000n,
      holderCount: 15,
      volumeTrendUp: false,
    };

    const result = client.computeScore(metrics);
    expect(result.score).toBeLessThanOrEqual(30);
    expect(
      result.tier === ScoreTier.Critical || result.tier === ScoreTier.Dangerous
    ).toBe(true);
  });

  it("produces probability distribution summing to 10000", () => {
    const metrics: TokenMetrics = {
      freshWalletBps: 5000,
      bundlerBps: 3000,
      top10HolderBps: 4000,
      smartMoneyCount: 3,
      devHoldingsBps: 1000,
      lpLocked: true,
      mintRevoked: false,
      mcapLamports: 20_000_000_000n,
      volume1mLamports: 2_000_000_000n,
      holderCount: 200,
      volumeTrendUp: true,
    };

    const result = client.computeScore(metrics);
    const dist = result.distribution;
    const total = dist.deathBps + dist.reach100kBps + dist.reach300kBps + dist.reach1mBps;

    expect(total).toBe(BPS_SCALE);
  });

  it("computes score of 100 for perfect metrics", () => {
    const metrics: TokenMetrics = {
      freshWalletBps: 0,
      bundlerBps: 0,
      top10HolderBps: 0,
      smartMoneyCount: 20,
      devHoldingsBps: 0,
      lpLocked: true,
      mintRevoked: true,
      mcapLamports: 1_000_000_000_000n,
      volume1mLamports: 100_000_000_000n,
      holderCount: 10_000,
      volumeTrendUp: true,
    };

    const result = client.computeScore(metrics);
    expect(result.score).toBe(100);
    expect(result.tier).toBe(ScoreTier.Healthy);
  });
});

describe("KovaClient Signal Generation", () => {
  const client = new KovaClient({
    rpcEndpoint: "https://api.devnet.solana.com",
  });

  it("generates warning for high fresh wallet %", () => {
    const metrics: TokenMetrics = {
      freshWalletBps: 7000,
      bundlerBps: 1000,
      top10HolderBps: 3000,
      smartMoneyCount: 2,
      devHoldingsBps: 300,
      lpLocked: true,
      mintRevoked: true,
      mcapLamports: 50_000_000_000n,
      volume1mLamports: 2_000_000_000n,
      holderCount: 300,
      volumeTrendUp: true,
    };

    const signals = client.generateSignals(metrics);
    const freshWarning = signals.find(
      (s) => s.metricKey === "freshWalletBps" && s.type === "warning"
    );
    expect(freshWarning).toBeDefined();
  });

  it("generates positive signal for smart money", () => {
    const metrics: TokenMetrics = {
      freshWalletBps: 3000,
      bundlerBps: 1000,
      top10HolderBps: 3000,
      smartMoneyCount: 5,
      devHoldingsBps: 100,
      lpLocked: true,
      mintRevoked: true,
      mcapLamports: 100_000_000_000n,
      volume1mLamports: 5_000_000_000n,
      holderCount: 500,
      volumeTrendUp: true,
    };

    const signals = client.generateSignals(metrics);
    const smSignal = signals.find(
      (s) => s.metricKey === "smartMoneyCount" && s.type === "positive"
    );
    expect(smSignal).toBeDefined();
    expect(smSignal?.metricValue).toBe(5);
  });

  it("generates warning for unrevoked mint authority", () => {
    const metrics: TokenMetrics = {
      freshWalletBps: 3000,
      bundlerBps: 1000,
      top10HolderBps: 3000,
      smartMoneyCount: 2,
      devHoldingsBps: 100,
      lpLocked: true,
      mintRevoked: false,
      mcapLamports: 50_000_000_000n,
      volume1mLamports: 2_000_000_000n,
      holderCount: 300,
      volumeTrendUp: true,
    };

    const signals = client.generateSignals(metrics);
    const mintWarning = signals.find(
      (s) => s.metricKey === "mintRevoked" && s.type === "warning"
    );
    expect(mintWarning).toBeDefined();
  });
});

describe("KovaClient Validation", () => {
  const client = new KovaClient({
    rpcEndpoint: "https://api.devnet.solana.com",
  });
  const keypair = Keypair.generate();

  it("rejects invalid weights on initialize", async () => {
    const badWeights = { ...DEFAULT_SCORING_WEIGHTS, freshWalletWeight: 9999 };
    await expect(
      client.initialize(keypair, {
        scoringWeights: badWeights,
        minSnapshotIntervalSecs: 5n,
      })
    ).rejects.toThrow(KovaValidationError);
  });

  it("rejects zero snapshot interval on initialize", async () => {
    await expect(
      client.initialize(keypair, {
        scoringWeights: DEFAULT_SCORING_WEIGHTS,
        minSnapshotIntervalSecs: 0n,
      })
    ).rejects.toThrow(KovaValidationError);
  });

  it("rejects invalid metrics on record snapshot", async () => {
    const badMetrics = {
      freshWalletBps: 15000, // out of range
      bundlerBps: 0,
      top10HolderBps: 0,
      smartMoneyCount: 0,
      devHoldingsBps: 0,
      lpLocked: false,
      mintRevoked: false,
      mcapLamports: 0n,
      volume1mLamports: 0n,
      holderCount: 0,
      volumeTrendUp: false,
    };

    await expect(
      client.recordSnapshot(
        keypair,
        { tokenMint: Keypair.generate().publicKey, metrics: badMetrics },
        0
      )
    ).rejects.toThrow(KovaValidationError);
  });

  it("rejects invalid weights on update config", async () => {
    const badWeights = { ...DEFAULT_SCORING_WEIGHTS, bundlerWeight: 5000 };
    await expect(
      client.updateConfig(keypair, {
        newWeights: badWeights,
        newMinInterval: null,
      })
    ).rejects.toThrow(KovaValidationError);
  });

  it("rejects zero interval on update config", async () => {
    await expect(
      client.updateConfig(keypair, {
        newWeights: null,
        newMinInterval: 0n,
      })
    ).rejects.toThrow(KovaValidationError);
  });
});
