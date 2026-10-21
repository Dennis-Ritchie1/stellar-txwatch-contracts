# Contributing Contracts

This guide covers how to contribute to the Soroban contracts in this repository.

## Getting Started

1. Fork the repository and create a feature branch.
2. Install the Rust toolchain and `cargo`.
3. Run the existing test suite before making changes.

## General Contract Contributions

- Keep changes focused and minimal.
- Match existing module structure, naming, and error handling.
- Add or update tests for any behavior change.
- Update documentation when the public surface changes.

## Adding a New Rule Descriptor

Rule descriptors (for example `rule:invoke`, `rule:burn`) are the string tags that
contracts and the off-chain watcher use to identify a rule's semantics. Adding a
new descriptor touches several layers, so follow the full process below.

1. **Propose** — open an issue describing the descriptor name, its semantics, and
the use case it enables. Get agreement on the name and meaning before writing code.
2. **Contract change** — add the descriptor to the `validate_rules` allow-list in
the contract so it is accepted during validation.
3. **Watcher change** — implement the matching logic for the descriptor in
`stellar-txwatch-core` so the off-chain watcher can act on it.
4. **Docs** — update the rule descriptor table in `docs/alert-registry.md` to
document the new descriptor.
5. **Test** — add a unit test in the contract covering validation of the new
descriptor, and an integration test in the core crate covering the watcher logic.
6. **TypeScript** — regenerate the TypeScript bindings if the type surface changes.

### Example PR Template for New Rule Descriptors

When opening a PR that adds a new rule descriptor, include the following checklist
in the PR description:

```markdown
## New Rule Descriptor: `rule:<name>`

- [ ] Proposal issue linked (name, semantics, use case)
- [ ] Descriptor added to `validate_rules` allow-list in the contract
- [ ] Matching logic implemented in `stellar-txwatch-core`
- [ ] `docs/alert-registry.md` rule descriptor table updated
- [ ] Contract unit test added
- [ ] Core integration test added
- [ ] TypeScript bindings regenerated (if the type surface changed)
```
