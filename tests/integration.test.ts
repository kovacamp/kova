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
