# Issue #276: Regenerate `@tx-wat/alert-registry-bindings` (plan)

**Status:** not done. Regenerating needs a WASM build and a package publish, which this change does not run.

## Steps the issue asks for

1. Build the compiled WASM: `cargo build --release --target wasm32-unknown-unknown`.
2. Regenerate the TypeScript bindings with `make bindings`.
3. Check that the generated `src/index.ts` includes the functions the compiled `lib.rs` defines. Do not
   expect functions that exist only in the uncompiled `contract.rs` (webhook propose and confirm,
   `renew_alert_ttl`), because they are not in the WASM.
4. Bump the version in `bindings/alert-registry/package.json`, and publish.

## Blockers

- The WASM build and the publish need the maintainers' release access. They are not run here.
- `bindings/alert-registry/src/` is generated and is not in the repository, so there is no file to check
  against.

## Done in this change

Nothing is regenerated and nothing is published. This plan records the steps and the blockers.
