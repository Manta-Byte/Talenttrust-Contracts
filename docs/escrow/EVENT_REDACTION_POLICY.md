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

The table below documents every event emitted by the contract, its payload, and the justification for each field.

### `contract_created`

| Field | Type | Justification |
|---|---|---|
| `contract_id` (topic) | `u32` | Non-sensitive internal identifier |
| `client` | `Address` | Public key; already visible in transaction history |
| `freelancer` | `Address` | Public key; already visible in transaction history |
| `arbiter` | `Option<Address>` | Public key; already visible in transaction history |
| `terms_hash` | `Option<Bytes>` | Hash of off-chain terms — provides integrity without leaking content |

**Excluded**: milestone amounts, milestone count, grace period, raw terms text.

---

### `funds_deposited`

| Field | Type | Justification |
|---|---|---|
| `contract_id` (topic) | `u32` | Non-sensitive internal identifier |
| `caller` | `Address` | Depositor identity; required for financial auditing |
| `amount` | `i128` | Deposit amount; required for financial auditing |

**Excluded**: contract terms, milestone breakdown, current contract status.

---

### `work_submitted`

| Field | Type | Justification |
|---|---|---|
| `contract_id` (topic) | `u32` | Non-sensitive internal identifier |
| `milestone_index` | `u32` | Identifies which milestone was submitted |
| `work_evidence_hash` | `Bytes` | Hash of off-chain evidence — integrity anchor without leaking content |

**Excluded**: `Milestone.work_evidence` raw bytes (URLs, file content, descriptions). This field is stored on-chain for contract logic but is **never** emitted in events.

---

### `milestone_approved`

| Field | Type | Justification |
|---|---|---|
| `contract_id` (topic) | `u32` | Non-sensitive internal identifier |
| `milestone_index` | `u32` | Identifies which milestone was approved |
| `approval_time` | `u64` | Ledger timestamp; standard blockchain metadata |

**Excluded**: approver address, work evidence, milestone amount.

---

### `milestone_released`

| Field | Type | Justification |
|---|---|---|
| `contract_id` (topic) | `u32` | Non-sensitive internal identifier |
| `milestone_index` | `u32` | Identifies which milestone was released |
| `released_amount` | `i128` | Amount released; required for financial auditing |

**Excluded**: recipient address, remaining balance, contract terms.

---

### `contract_cancelled`

| Field | Type | Justification |
|---|---|---|
| `contract_id` (topic) | `u32` | Non-sensitive internal identifier |
| `caller` | `Address` | Cancelling party; required for audit trail |
| `status` | `ContractStatus` | Final status enum; required for state tracking |
| `timestamp` | `u64` | Ledger timestamp; standard blockchain metadata |

**Excluded**: deposited amounts, milestone details, cancellation reason text.

---

## Sensitive Fields — Never Emitted

| Field | Location | Risk |
|---|---|---|
| `Milestone.work_evidence` | `ContractData.milestones` | May contain URLs, file paths, or descriptions of private work |
| Raw contract terms | Off-chain | Full text of legal agreements; PII risk |
| Milestone amounts breakdown | `ContractData.milestones` | Detailed financial structure; not needed in events |
| Deposited balance | `ContractData.total_deposited` | Aggregate balance; not needed per-event |

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
- The `test_full_lifecycle_event_sequence_is_redaction_compliant` test validates the complete event sequence for a contract lifecycle.
- Any PR adding or modifying an event must update this document and add corresponding tests.
- Use `Bytes` (hashes) instead of `String` for any off-chain data references.
