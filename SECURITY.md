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

## Contact

Maintainer: Emmanuel Chukwunyere — emmanuelanalaba@gmail.com  
Organization: [Tx-wat](https://github.com/Tx-wat)
