# Contributing to Kova

Contributions are welcome. This document describes the development workflow and expectations for pull requests.

## Development Setup

```
git clone <repo-url> && cd kova-core
```

Install Rust 1.75+, Solana CLI 1.17.0, and Anchor CLI 0.29.0. Then:

```
anchor build
cargo test --workspace
```

For SDK and integration tests, install Node.js 20+ and run:

```
cd sdk && npm install && cd ..
anchor test
```

## Pull Request Guidelines

- One logical change per PR.
- All CI checks must pass: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `anchor test`.
- New instructions require corresponding test coverage in both Rust unit tests and TypeScript integration tests.
- Use `checked_add`, `checked_sub`, `checked_mul`, `checked_div` for all arithmetic on-chain. No unchecked math.
- Add doc comments (`///`) to all public Rust functions and types.
- Add JSDoc comments (`/** */`) to all exported TypeScript functions and types.
- Keep PR descriptions focused on *why* the change is needed, not just *what* changed.

## Code Style

- Rust: enforced by `rustfmt.toml` (run `cargo fmt`).
- TypeScript: strict mode enabled in `tsconfig.json`.
- Naming: use domain-specific names. Prefer `fresh_wallet_bps` over `amount` or `val`.
- Comments: only on non-obvious logic. The code should be self-documenting where possible.

## Security

- All on-chain authority checks must use Anchor's `constraint` attribute or `require!` macro.
- PDA seeds and bumps must be validated in every account context.
- New error variants go in `errors.rs` with descriptive `#[msg()]` attributes.
- Metric validation must occur before any state mutation.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
