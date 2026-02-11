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
