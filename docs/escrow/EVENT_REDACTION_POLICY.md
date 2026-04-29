# Event Redaction Policy for Escrow Contract

## Overview

This document defines the authoritative rules for emitting events from the Escrow smart contract. The primary goal is to prevent sensitive off-chain metadata from being permanently written to the public ledger.

All events emitted by this contract have been reviewed against this policy. Any new event must pass the same review before merging.

---

## Redaction Rules

1. **No PII**: Events must never include names, email addresses, phone numbers, or any other personally identifiable information.
2. **No raw off-chain data**: Data that lives off-chain (work descriptions, evidence documents, full contract terms) must never be emitted in raw form.
3. **Hashes and IDs only**: If off-chain data must be referenced, include only its cryptographic hash (e.g., SHA-256) or a numeric identifier.
4. **Minimal payloads**: Emit only the fields strictly necessary for off-chain indexers and frontends to track contract state.

---

## Approved Event Payloads

### `contract_cancelled`

| Field | Type | Justification |
|---|---|---|
| `contract_id` (topic) | `u32` | Non-sensitive internal identifier |
| `caller` | `Address` | Cancelling party; required for audit trail |
| `status` | `ContractStatus` | Final status enum; required for state tracking |
| `timestamp` | `u64` | Ledger timestamp; standard blockchain metadata |

**Excluded**: deposited amounts, milestone details, cancellation reason text, contract terms.

---

## Sensitive Fields — Never Emitted

| Field | Risk |
|---|---|
| Raw contract terms | Full text of legal agreements; PII risk |
| Milestone amounts breakdown | Detailed financial structure; not needed in events |
| `total_deposited` balance | Aggregate balance; not needed per-event |
| Work evidence content | May contain URLs, file paths, or descriptions of private work |

---

## Guidelines for Future Events

When adding new events, apply these rules:

- **Safe to emit**: public `Address` values, numeric IDs (`u32`), token amounts (`i128`) when required for financial auditing, status enums, ledger timestamps, cryptographic hashes (`Bytes`).
- **Never emit**: raw strings describing off-chain content, full contract terms, work evidence content, PII of any kind.
- Add a `// REDACTION POLICY:` comment at each `env.events().publish()` call documenting what is included and what is excluded.
- Add a corresponding test in `contracts/escrow/src/test/event_redaction.rs` asserting the exact topic and data payload.
- Update this document with the new event's approved payload table.

---

## Why This Policy Exists

- **Privacy**: Protects users from having sensitive data permanently written to a public, immutable ledger.
- **Compliance**: Supports GDPR and similar regulations that restrict permanent storage of PII.
- **Efficiency**: Smaller event payloads reduce on-chain storage costs and indexer processing overhead.
- **Security**: Prevents accidental disclosure of private business logic or documents.

---

## Enforcement

- All event emissions in `lib.rs` carry a `// REDACTION POLICY:` comment explaining what is included and what is excluded.
- Unit tests in `contracts/escrow/src/test/event_redaction.rs` assert the exact topic and data payload of every event type.
- Any PR adding or modifying an event must update this document and add corresponding tests.
- Use `Bytes` (hashes) instead of `String` for any off-chain data references.
