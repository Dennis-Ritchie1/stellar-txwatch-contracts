# Issue #273: Count by contract after concurrent owner operations (scaffold)

**Status:** blocked. `get_alert_count_for_contract` is not in the compiled contract.

## Why it is blocked

The issue tests the accuracy of `get_alert_count_for_contract`. That function is not in the compiled
`contracts/alert-registry/src/lib.rs`, so there is no count to check. The other count helpers that are
compiled (`get_alert_count`, `get_active_alert_count`) are covered in #272.

## What is needed first

1. Add `get_alert_count_for_contract(target)` to `lib.rs`. It counts live alerts in the contract index
   for `target`, skipping expired entries.
2. Add a `transfer_alert_ownership` or `update_target_contract` path that keeps the index in step, if
   those scenarios are to be tested.

## Test scenarios (from the issue)

1. Register from two owners to one contract: the count rises with each registration.
2. Remove by one owner: the count falls, and the other owner's alert is unchanged.
3. Ownership transfer: the count for the contract does not change.
4. Retarget an alert to another contract: the old count falls and the new count rises.
5. Admin removal: the count falls.

## Not done

No function, no tests.
