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
