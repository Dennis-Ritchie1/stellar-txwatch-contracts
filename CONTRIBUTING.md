# Contributing

## Prerequisites

Install the Rust toolchain and Soroban target:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

Install the Stellar CLI:

```bash
cargo install --locked stellar-cli --features opt
```

## Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

## Test

```bash
cargo test
```

Tests run natively (no WASM target needed). Each contract has a `#[cfg(test)]` module covering happy paths, unauthorized rejections, and edge cases.

## Deploy to Testnet

1. Set up a funded testnet identity:

```bash
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet
```

2. Run the deploy script:

```bash
bash scripts/deploy.sh
```

3. Update `DEPLOYMENTS.md` with the printed contract addresses.

## Adding a New Function to an Existing Contract

1. Add the function inside the `#[contractimpl]` block in `contracts/<name>/src/lib.rs`.
2. If it mutates state, call `<caller>.require_auth()` as the first line.
3. Add at least one test in the `#[cfg(test)]` module covering the happy path and any auth rejection.
4. Run `cargo test` to confirm everything passes.
5. Update the relevant doc in `docs/`.

## Branch Protection & Merge Process

The `main` branch is protected to keep contract code safe from accidental force pushes and unreviewed merges. The following rules are enforced in GitHub settings:

- **Pull request required.** All changes land on `main` through a pull request; direct pushes to `main` are restricted.
- **At least 1 approval.** Every PR needs at least one approving review before it can be merged.
- **Required status checks.** A PR cannot merge until these checks pass:
  - `CI`
  - `clippy`
  - `fmt-check`
  - `doc-check`
- **Up-to-date branches.** The PR branch must be up to date with `main` before merging (rebase or merge `main` in if it has moved ahead).
- **Signed commits (recommended).** Signing commits is strongly encouraged for contract code so authorship and integrity can be verified.

### Why these rules exist

Smart contract code is immutable once deployed, so mistakes are costly. Requiring review, passing CI, and an up-to-date branch ensures every change is validated against the latest `main` before it ships, and blocking force pushes prevents history from being rewritten out from under reviewers and deployers.

### Merge workflow

1. Create a feature branch off `main`.
2. Open a pull request and request at least one review.
3. Ensure `CI`, `clippy`, `fmt-check`, and `doc-check` are green.
4. If `main` has advanced, update your branch so it is up to date.
5. Merge once approved and all checks pass.

## Sister Repos

- **Core engine:** https://github.com/Tx-wat/stellar-txwatch-core
- **Web dashboard:** https://github.com/Tx-wat/stellar-txwatch-web
