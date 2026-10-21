# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report security issues through either of the following private channels:

- **Email:** emmanuelanalaba@gmail.com
- **GitHub private advisory:** open a draft advisory via the repository's
  [Security tab](https://github.com/Tx-wat/security/advisories/new)

Include as much detail as possible:
- A description of the vulnerability and its potential impact
- Steps to reproduce or a proof-of-concept
- Affected contract(s): `alert-registry`, `watcher-registry`, or both
- Any suggested mitigations

### What to expect

| Timeline | Action |
|----------|--------|
| Within 48 hours | Acknowledgement of your report |
| Within 7 days | Initial assessment and severity classification |
| Within 30 days | Patch or mitigation plan communicated to reporter |
| After fix is deployed | Public disclosure coordinated with reporter |

We follow responsible disclosure: we ask that you give us reasonable time to address the issue before any public disclosure.

## Severity Definitions

| Severity | Description | Example |
|----------|-------------|---------|
| Critical | Direct loss of funds or complete contract takeover | Admin key compromise vector |
| High | Unauthorized state mutation | Bypassing `require_auth` |
| Medium | Data integrity violation | Index corruption |
| Low | Informational / DoS | TTL manipulation |

## Response SLAs

Once a report is classified, we commit to the following timelines:

| Severity | Acknowledge within | Patch within |
|----------|--------------------|--------------|
| Critical | 24 hours | 7 days |
| High | 48 hours | 14 days |
| Medium / Low | 7 days | 30 days |

## Scope

The following are in scope:

- Logic errors in `AlertRegistry` or `WatcherRegistry` contract functions
- Authorization bypass (e.g., circumventing `require_auth()`)
- Storage manipulation or data corruption vectors
- Denial-of-service via resource exhaustion on-chain

The following are **out of scope**:

- Admin key security (operator responsibility)
- Off-chain watcher behavior
- Issues in third-party dependencies (report those upstream)
- Stellar protocol-level vulnerabilities (report to the [Stellar Development Foundation](https://stellar.org/bug-bounty))
- Issues in off-chain infrastructure not part of this repository

## `require_auth()` Placement Audit

Soroban's `require_auth()` must be the **very first instruction** in every mutating
function. If it runs after a storage read (e.g. an ownership lookup) or after any
conditional logic, an attacker can observe whether that read succeeded before
authorization is enforced, leaking state and allowing side effects to occur
pre-auth. The following audit verifies that no storage reads or conditional logic
precede `require_auth()` in any mutating entry point.

### AlertRegistry

| Function | `require_auth()` first? | Notes |
|----------|-------------------------|-------|
| `register_alert` | ✅ | Auth on caller before any storage access |
| `update_alert` | ✅ | Auth on caller before ownership/state reads |
| `update_webhook` | ✅ | Auth on caller before webhook read |
| `propose_webhook` | ✅ | Auth on caller before proposal write |
| `confirm_webhook` | ✅ | Auth on caller before confirmation read |
| `renew_alert_ttl` | ✅ | Auth on caller before TTL read |
| `remove_alert` | ✅ | Auth on caller before removal |
| `remove_alert_by_admin` | ✅ | Auth on admin before removal |
| `initialize` | ✅ | Auth on admin before init guard |
| `transfer_admin` | ✅ | Auth on current admin before transfer |
| `set_per_owner_alert_limit` | ✅ | Auth on admin before limit write |

### WatcherRegistry

| Function | `require_auth()` first? | Notes |
|----------|-------------------------|-------|
| `initialize` | ✅ | Auth on admin before init guard |
| `add_admin` | ✅ | Auth on admin before admin write |
| `remove_admin` | ✅ | Auth on admin before admin removal |
| `transfer_admin` | ✅ | Auth on current admin before transfer |
| `register_watcher` | ✅ | Auth on caller before registration |
| `remove_watcher` | ✅ | Auth on caller before removal |
| `clear_all_watchers` | ✅ | Auth on admin before clearing |

### Guidance for Contributors

When adding or modifying a mutating function:

1. Call `require_auth()` on the relevant address as the **first statement** of the
   function body — before any `env.storage()` access, ownership check, or `if`.
2. Perform ownership/role validation only *after* `require_auth()` has succeeded.
3. Never gate `require_auth()` behind a conditional or a prior storage read.

## Contact

Maintainer: Emmanuel Chukwunyere — emmanuelanalaba@gmail.com  
Organization: [Tx-wat](https://github.com/Tx-wat)
