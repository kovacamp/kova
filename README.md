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

## Scoring Weights

All weights are in basis points (sum = 10,000 = 100%).

| Factor | Default Weight | bps |
|--------|---------------|-----|
| Fresh Wallet % | 20% | 2000 |
| Bundler % | 15% | 1500 |
| Top 10 Holder % | 15% | 1500 |
| Smart Money | 10% | 1000 |
| Dev Holdings | 10% | 1000 |
| LP Locked | 10% | 1000 |
| Mint Revoked | 5% | 500 |
| Volume Trend | 5% | 500 |
| Fresh Wallet Slope | 5% | 500 |
| Top 10 Slope | 5% | 500 |

Weights are on-chain and adjustable via `update_config` by the config authority.

## Probability Distribution

Given a score, Kova maps it to a four-bucket distribution (basis points,
sum = 10,000):

| Bucket | Formula |
|--------|---------|
| Death (rug / fade) | `max(0, 9500 - score * 80)` |
| Reach 100K mcap | `min(score * 30, 2500)` |
| Reach 300K mcap | `min(max(0, (score - 40) * 20), 1500)` |
| Reach 1M+ mcap | remainder to 10,000 |

Example at score 72:

```
Death     17.4%  ████████▋
100K      21.6%  ██████████▊
300K      6.4%   ███▏
1M+       54.6%  ███████████████████████████▎
```

## Data Pipeline

```mermaid
flowchart TB
    subgraph Input ["Raw Metrics"]
        M1["Holder Distribution\n(fresh %, bundler %, top10 %)"]
        M2["Token Config\n(LP locked, mint revoked, dev %)"]
        M3["Market Activity\n(smart money count, volume trend)"]
    end

    VALIDATE["Metric Validation\n- Range check (0-10000 bps)\n- Boolean check (0 or 1)\n- Token mint validation"]

    SNAPSHOT["TokenSnapshot PDA\n(immutable, sequential index)\n- 11 metric fields\n- timestamp + recorder"]

    subgraph Scoring ["Score Computation"]
        INVERT["Inverse Metrics\n10000 - value\n(fresh, bundler, top10, dev)"]
        DIRECT["Positive Metrics\ndirect value\n(smart money, LP, mint, volume)"]
        DERIVED["Derived Metrics\nslope proxies\n(fresh slope, top10 slope)"]
    end

    WEIGHTED["Weighted Sum\nsum(sub_score * weight) / weight_total\n-> normalized to 0-100"]

    TIER["Tier Classification\nCritical | Dangerous | Caution\nModerate | Healthy"]

    DIST["Probability Distribution\n4 buckets in basis points\nDeath | 100K | 300K | 1M+"]

    RECORD["ScanRecord PDA\n- score (u8)\n- tier (enum)\n- distribution (4 x u16)\n- snapshots_used\n- timestamps"]

    M1 --> VALIDATE
    M2 --> VALIDATE
    M3 --> VALIDATE
    VALIDATE --> SNAPSHOT
    SNAPSHOT --> INVERT
    SNAPSHOT --> DIRECT
    SNAPSHOT --> DERIVED
    INVERT --> WEIGHTED
    DIRECT --> WEIGHTED
    DERIVED --> WEIGHTED
    WEIGHTED --> TIER
    WEIGHTED --> DIST
    TIER --> RECORD
    DIST --> RECORD
```
