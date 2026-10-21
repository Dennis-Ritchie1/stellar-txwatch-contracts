# Issue #261: Two-step webhook rotation tests (scaffold)

**Status:** blocked. The compiled contract has no propose/confirm webhook flow, so there is nothing to
test yet.

## Why it is blocked

`propose_webhook` and `confirm_webhook` are defined only in `contracts/alert-registry/src/contract.rs`.
That file is not declared as a module in `lib.rs`, so it is not compiled into the deployed contract. The
compiled `lib.rs` has `update_webhook`, which changes the hash in one step.

## What is needed first

1. Port `propose_webhook` and `confirm_webhook` into `lib.rs`, with a pending-hash key family.
2. Add the `NoPendingWebhook` error to the compiled enum, matching the discriminant used in `contract.rs`.

## Test scenarios (from the issue)

1. Propose, then confirm: the new hash is stored and the pending entry is cleared.
2. Confirm with no proposal: `NoPendingWebhook`.
3. Propose twice, then confirm: the second proposal is the one applied.
4. Confirm by a non-owner: `Unauthorized`.
5. Propose, remove the alert, then confirm: `AlertNotFound`.
6. Events: the propose and confirm events match the shapes in `docs/events.md`.

## Not done

No port, no tests, no pending-hash key.
