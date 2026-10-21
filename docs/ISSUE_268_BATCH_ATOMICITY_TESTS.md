# Issue #268: Batch registration is atomic (scaffold)

**Status:** blocked. `batch_register_alerts` is not in the compiled contract, so there is no batch to test.

## Why it is blocked

The issue depends on #215, which adds `batch_register_alerts`. The function does not appear in the
compiled `contracts/alert-registry/src/lib.rs`. Without it, an atomicity test has nothing to call.

## What is needed first

1. Implement `batch_register_alerts` (issue #215) in `lib.rs`, with a size cap checked before any write.
2. Make every per-alert check run before the first write, so a failure leaves no partial state.

## Test scenarios (from the issue)

1. All valid: every alert is registered and every ID is returned.
2. Last alert invalid: none is registered.
3. First alert invalid: none is registered.
4. Per-owner limit exceeded midway: none is registered.
5. Batch too large: refused before any write.

After each failed call, check that `get_alert_count` and the owner's active count are unchanged.

## Not done

No batch function, no tests.
