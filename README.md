# kova-core

<p align="center">
  <img src="kovagh.png" alt="kova" width="480" />
</p>

<p align="center">
  Token survival probability scanner for Solana.
</p>

<p align="center">
  <a href="https://kova.camp"><img src="https://img.shields.io/badge/website-kova.camp-5EEAD4?style=for-the-badge&logo=googlechrome&logoColor=white" alt="Website" /></a>
  <a href="https://x.com/kovadotcamp"><img src="https://img.shields.io/badge/X-@kovadotcamp-000000?style=for-the-badge&logo=x&logoColor=white" alt="X" /></a>
</p>

<p align="center">
  <a href="https://github.com/kovacamp/kova/actions"><img src="https://github.com/kovacamp/kova/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <img src="https://img.shields.io/badge/rust-1.75%2B-orange" alt="Rust" />
  <img src="https://img.shields.io/badge/anchor-0.29.0-blue" alt="Anchor" />
  <img src="https://img.shields.io/badge/solana-1.17.0-9945FF" alt="Solana" />
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License" />
</p>

---

Kova quantifies how likely a Solana token is to survive using a multi-factor
scoring algorithm. It captures on-chain metrics, computes a weighted survival
score (0--100), and derives a probability distribution across four market-cap
outcomes.

Everything runs on integer arithmetic with overflow checks. No floating point
anywhere in the stack.

## Feature Engineering

Kova evaluates tokens across **10 weighted sub-scores** derived from real-time
on-chain metrics:

| Factor | Direction | What It Measures |
|--------|-----------|-----------------|
| Fresh Wallet % | Inverse | Proportion of holders with new wallets |
| Bundler % | Inverse | Proportion flagged as sniper/bundler activity |
| Top 10 Holder % | Inverse | Concentration in the top 10 wallets |
| Dev Holdings % | Inverse | Developer wallet retention |
| Smart Money Count | Positive | Number of tracked smart-money wallets holding |
| LP Locked | Positive | Whether liquidity pool is locked |
| Mint Revoked | Positive | Whether mint authority is revoked |
| Volume Trend | Positive | Directional volume movement |
| Fresh Wallet Slope | Derived | Rate of change in fresh wallet concentration |
| Top 10 Slope | Derived | Rate of change in top-holder concentration |

Inverse factors penalize high values. Positive factors reward them. Derived
factors track time-series momentum.

## Architecture

```mermaid
graph TB
    subgraph Program ["On-Chain Program &mdash; kova_scanner (Anchor)"]
        ENTRY["Program Entry\nlib.rs"]
        ENTRY --> INIT["initialize\n(setup global config)"]
        ENTRY --> RECORD_S["record_snapshot\n(capture metrics)"]
        ENTRY --> CALC["calculate_score\n(compute survival %)"]
        ENTRY --> UPDATE["update_config\n(adjust weights)"]

        INIT --> CONFIG["TokenScanConfig PDA\n(singleton, authority + weights)"]
        RECORD_S --> SNAP["TokenSnapshot PDA\n(per-token, sequential index)"]
        CALC --> SCANREC["ScanRecord PDA\n(per-token, score + distribution)"]
        UPDATE --> CONFIG
    end

    subgraph Math ["kova-math (Rust, integer-only)"]
        WS["weighted_score\n(10 sub-scores * weights)"]
        PD["probability_distribution\n(score -> 4 outcome buckets)"]
        SLOPE["time_series_slope\n(linear regression)"]
        STAT["ema / z_score / std_dev\n(statistical primitives)"]
        WS --> PD
    end

    subgraph SDK ["@kova-protocol/sdk (TypeScript)"]
        TYPES["Types + Constants\n(ScoreTier, TokenMetrics, weights)"]
        INSTR["Instruction Builders\n(PDA derivation + Borsh encoding)"]
        CLIENT["KovaClient\n(high-level API)"]
        SIGNALS["Signal Generation\n(warning / positive)"]
        TYPES --> INSTR
        INSTR --> CLIENT
        SIGNALS --> CLIENT
    end

    subgraph CLI ["kova-cli (Rust)"]
        CLI_SCAN["scan\n(one-shot token check)"]
        CLI_MON["monitor\n(real-time polling)"]
        CLI_GY["graveyard\n(dead token list)"]
        CLI_STAT["stats\n(config diagnostics)"]
    end

    CALC --> |"calls"| WS
    CALC --> |"calls"| PD
    CLIENT --> |"build & send txs"| ENTRY
    CLI_SCAN --> |"RPC read"| SCANREC
    CLI_MON --> |"RPC poll"| SCANREC
    CLI_SCAN --> |"local compute"| WS
```

## Score Tiers

| Score | Tier | Label |
|-------|------|-------|
| 0 -- 19 | Critical | Extreme risk |
| 20 -- 39 | Dangerous | High risk |
| 40 -- 59 | Caution | Moderate risk |
| 60 -- 79 | Moderate | Lower risk |
| 80 -- 100 | Healthy | Lowest observed risk |
