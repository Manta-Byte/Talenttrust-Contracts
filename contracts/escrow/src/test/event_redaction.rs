#![cfg(test)]

//! Event Redaction Policy Tests
//!
//! Enforces the contract's event redaction policy:
//! - Only hashes/IDs are emitted for off-chain data references.
//! - Raw work evidence, contract terms text, and PII are never emitted.
//! - Each event's topic and data payload is asserted exactly.
//!
//! See docs/escrow/EVENT_REDACTION_POLICY.md for the full policy.

use soroban_sdk::{testutils::{Address as _, Events}, vec, Address, Env, IntoVal, Symbol};

use crate::{ContractStatus, Escrow, EscrowClient};

fn register_client(env: &Env) -> EscrowClient {
    let id = env.register(Escrow, ());
    EscrowClient::new(env, &id)
}

// ─── contract_cancelled ──────────────────────────────────────────────────────

#[test]
fn test_contract_cancelled_event_topics() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &vec![&env, 100_i128],
    );

    client.cancel_contract(&id, &client_addr);

    let events = env.events().all();
    let last = events.last().unwrap();

    // Topics: (Symbol("contract_cancelled"), contract_id) — no raw data in topics.
    assert_eq!(
        last.0,
        (
            client.address.clone(),
            (Symbol::new(&env, "contract_cancelled"), id).into_val(&env)
        )
    );
}

#[test]
fn test_contract_cancelled_event_data_safe_fields_only() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &vec![&env, 100_i128],
    );

    client.cancel_contract(&id, &client_addr);

    let events = env.events().all();
    let last = events.last().unwrap();

    // Data: (caller, status_enum, timestamp)
    // Safe: caller is a public address, status is an enum, timestamp is ledger metadata.
    // NOT included: deposited amounts, milestone details, contract terms.
    let ts = env.ledger().timestamp();
    assert_eq!(
        last.1,
        (client_addr.clone(), ContractStatus::Cancelled, ts).into_val(&env)
    );
}

#[test]
fn test_contract_cancelled_status_is_enum_not_raw_string() {
    // Status must be emitted as a typed enum (u32 discriminant), never as a raw string.
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &vec![&env, 100_i128],
    );

    client.cancel_contract(&id, &client_addr);

    let events = env.events().all();
    let last = events.last().unwrap();
    let ts = env.ledger().timestamp();

    // ContractStatus::Cancelled = 4 — emitted as typed enum, not a string like "Cancelled"
    assert_eq!(
        last.1,
        (client_addr.clone(), ContractStatus::Cancelled, ts).into_val(&env)
    );
}

#[test]
fn test_cancel_by_freelancer_emits_correct_caller() {
    // Verify the emitted caller matches whoever actually cancelled,
    // confirming no address substitution occurs.
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &vec![&env, 100_i128],
    );

    // Freelancer cancels
    client.cancel_contract(&id, &freelancer_addr);

    let events = env.events().all();
    let last = events.last().unwrap();
    let ts = env.ledger().timestamp();

    assert_eq!(
        last.1,
        (freelancer_addr.clone(), ContractStatus::Cancelled, ts).into_val(&env)
    );
}

// ─── Policy: no sensitive data in any event ───────────────────────────────────

#[test]
fn test_only_one_event_emitted_on_cancel() {
    // Cancellation should emit exactly one event — no internal state leakage
    // through extra events.
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &vec![&env, 100_i128],
    );

    client.cancel_contract(&id, &client_addr);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly 1 event on cancel");
}
