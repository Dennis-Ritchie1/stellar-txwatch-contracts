# Issue #278: Export `AlertConfig` and `AlertInput` from the bindings (blocked)

**Status:** blocked. The export has to be added to the generated `bindings/alert-registry/src/index.ts`,
and that file is not in the repository.

## Why it is blocked

`bindings/alert-registry/src/` is produced at build time from the WASM. It is not committed, so there is
no file to edit. A hand-written change would be lost the next time the bindings are generated.

## What is needed first

1. Regenerate the bindings (see [`ISSUE_276_BINDINGS_REGEN_PLAN.md`](./ISSUE_276_BINDINGS_REGEN_PLAN.md)).
2. Add `export type { AlertConfig, AlertInput, ContractError };` to the generator's template, so the export
   survives regeneration. Editing the generated file alone is not enough.

## Not done

No export added, no file changed.
