#![cfg(test)]

//! Event Redaction Policy Tests
//!
//! These tests enforce the contract's event redaction policy:
//! - Only hashes/IDs are emitted for off-chain data references.
//! - Raw work evidence bytes, contract terms text, and PII are never emitted.
//! - Each event's topic and data payload is asserted exactly.
//!
//! See docs/escrow/EVENT_REDACTION_POLICY.md for the full policy.

use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Bytes, Env, IntoVal, Symbol,
};

use crate::{ContractStatus, Escrow, EscrowClient};

fn register_client(env: &Env) -> EscrowClient {
    let id = env.register(Escrow, ());
    EscrowClient::new(env, &id)
}

/// Helper: create a funded contract ready for milestone work.
fn setup_funded_contract(env: &Env, client: &EscrowClient) -> (Address, Address, u32) {
    let client_addr = Address::generate(env);
    let freelancer_addr = Address::generate(env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &vec![env, 100_i128],
        &None,
        &None,
    );
    client.deposit_funds(&id, &100_i128, &client_addr);
    (client_addr, freelancer_addr, id)
}

// ─── contract_created ────────────────────────────────────────────────────────

#[test]
fn test_contract_created_event_topics() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &vec![&env, 100_i128],
        &None,
        &None,
    );

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.0,
        (
            client.address.clone(),
            (Symbol::new(&env, "contract_created"), id).into_val(&env)
        )
    );
}

#[test]
fn test_contract_created_event_data_contains_only_safe_fields() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);
    let terms_hash = Some(Bytes::from_array(&env, &[0xabu8; 32]));

    client.create_contract(
        &client_addr,
        &freelancer_addr,
        &Some(arbiter_addr.clone()),
        &vec![&env, 100_i128],
        &terms_hash,
        &None,
    );

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.1,
        (
            client_addr.clone(),
            freelancer_addr.clone(),
            Some(arbiter_addr.clone()),
            terms_hash.clone()
        )
            .into_val(&env)
    );
}

#[test]
fn test_contract_created_event_no_milestone_amounts_in_payload() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestone_amounts = vec![&env, 500_i128, 1000_i128, 1500_i128];

    client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &milestone_amounts,
        &None,
        &None,
    );

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.1,
        (client_addr, freelancer_addr, Option::<Address>::None, Option::<Bytes>::None)
            .into_val(&env)
    );
}

// ─── funds_deposited ─────────────────────────────────────────────────────────

#[test]
fn test_funds_deposited_event_payload() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &vec![&env, 100_i128],
        &None,
        &None,
    );

    client.deposit_funds(&id, &100_i128, &client_addr);

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.0,
        (
            client.address.clone(),
            (Symbol::new(&env, "funds_deposited"), id).into_val(&env)
        )
    );
    assert_eq!(last.1, (client_addr.clone(), 100_i128).into_val(&env));
}

// ─── work_submitted ──────────────────────────────────────────────────────────

#[test]
fn test_work_submitted_emits_hash_not_raw_evidence() {
    // CRITICAL: raw work_evidence bytes must NEVER appear in events.
    // Only the hash submitted by the freelancer is emitted.
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let (_, freelancer_addr, id) = setup_funded_contract(&env, &client);
    let work_hash = Bytes::from_array(&env, &[0x42u8; 32]);

    client.submit_milestone_work(&id, &0, &work_hash, &freelancer_addr);

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.0,
        (
            client.address.clone(),
            (Symbol::new(&env, "work_submitted"), id).into_val(&env)
        )
    );
    assert_eq!(last.1, (0u32, work_hash.clone()).into_val(&env));
}

#[test]
fn test_work_submitted_different_hashes_produce_distinct_events() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);

    let id1 = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &vec![&env, 100_i128],
        &None,
        &None,
    );
    let id2 = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &vec![&env, 100_i128],
        &None,
        &None,
    );
    client.deposit_funds(&id1, &100_i128, &client_addr);
    client.deposit_funds(&id2, &100_i128, &client_addr);

    let hash_a = Bytes::from_array(&env, &[0x01u8; 32]);
    let hash_b = Bytes::from_array(&env, &[0x02u8; 32]);

    client.submit_milestone_work(&id1, &0, &hash_a, &freelancer_addr);
    client.submit_milestone_work(&id2, &0, &hash_b, &freelancer_addr);

    let events = env.events().all();
    let n = events.len();

    let ev1 = events.get(n - 2).unwrap();
    let ev2 = events.get(n - 1).unwrap();

    assert_eq!(ev1.1, (0u32, hash_a).into_val(&env));
    assert_eq!(ev2.1, (0u32, hash_b).into_val(&env));
    assert_ne!(ev1.1, ev2.1);
}

// ─── milestone_approved ──────────────────────────────────────────────────────

#[test]
fn test_milestone_approved_event_payload() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let (client_addr, freelancer_addr, id) = setup_funded_contract(&env, &client);
    let work_hash = Bytes::from_array(&env, &[0x01u8; 32]);
    client.submit_milestone_work(&id, &0, &work_hash, &freelancer_addr);
    client.approve_milestone(&id, &0, &client_addr);

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.0,
        (
            client.address.clone(),
            (Symbol::new(&env, "milestone_approved"), id).into_val(&env)
        )
    );

    // Data: (milestone_index, approval_timestamp) — no approver address, no amounts.
    let approval_time = env.ledger().timestamp();
    assert_eq!(last.1, (0u32, approval_time).into_val(&env));
}

// ─── milestone_released ──────────────────────────────────────────────────────

#[test]
fn test_milestone_released_event_payload() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let (client_addr, _, id) = setup_funded_contract(&env, &client);
    client.release_milestone(&id, &0, &client_addr);

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.0,
        (
            client.address.clone(),
            (Symbol::new(&env, "milestone_released"), id).into_val(&env)
        )
    );
    // Data: (milestone_index, released_amount) — no participant addresses.
    assert_eq!(last.1, (0u32, 100_i128).into_val(&env));
}

// ─── contract_cancelled ──────────────────────────────────────────────────────

#[test]
fn test_contract_cancelled_event_payload() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &vec![&env, 100_i128],
        &None,
        &None,
    );

    client.cancel_contract(&id, &client_addr);

    let events = env.events().all();
    let last = events.last().unwrap();

    assert_eq!(
        last.0,
        (
            client.address.clone(),
            (Symbol::new(&env, "contract_cancelled"), id).into_val(&env)
        )
    );

    // Data: (caller, status_enum, timestamp) — no deposited amounts or milestone details.
    let ts = env.ledger().timestamp();
    assert_eq!(
        last.1,
        (client_addr.clone(), ContractStatus::Cancelled, ts).into_val(&env)
    );
}

// ─── Full lifecycle policy walkthrough ───────────────────────────────────────

#[test]
fn test_full_lifecycle_event_sequence_is_redaction_compliant() {
    // Walk through a complete contract lifecycle and verify every emitted event
    // conforms to the redaction policy: no raw off-chain data, only hashes/IDs.
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let terms_hash = Some(Bytes::from_array(&env, &[0xffu8; 32]));
    let work_hash = Bytes::from_array(&env, &[0xaau8; 32]);

    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &vec![&env, 200_i128],
        &terms_hash,
        &None,
    );
    client.deposit_funds(&id, &200_i128, &client_addr);
    client.submit_milestone_work(&id, &0, &work_hash, &freelancer_addr);
    client.approve_milestone(&id, &0, &client_addr);
    client.release_milestone(&id, &0, &client_addr);

    let events = env.events().all();
    assert_eq!(events.len(), 5, "Expected exactly 5 events for this lifecycle");

    // Event 0: contract_created — addresses + terms hash only
    let ev = events.get(0).unwrap();
    assert_eq!(
        ev.0,
        (client.address.clone(), (Symbol::new(&env, "contract_created"), id).into_val(&env))
    );
    assert_eq!(
        ev.1,
        (client_addr.clone(), freelancer_addr.clone(), Option::<Address>::None, terms_hash.clone())
            .into_val(&env)
    );

    // Event 1: funds_deposited — caller + amount only
    let ev = events.get(1).unwrap();
    assert_eq!(
        ev.0,
        (client.address.clone(), (Symbol::new(&env, "funds_deposited"), id).into_val(&env))
    );
    assert_eq!(ev.1, (client_addr.clone(), 200_i128).into_val(&env));

    // Event 2: work_submitted — hash only, no raw evidence
    let ev = events.get(2).unwrap();
    assert_eq!(
        ev.0,
        (client.address.clone(), (Symbol::new(&env, "work_submitted"), id).into_val(&env))
    );
    assert_eq!(ev.1, (0u32, work_hash.clone()).into_val(&env));

    // Event 3: milestone_approved — index + timestamp only
    let ev = events.get(3).unwrap();
    assert_eq!(
        ev.0,
        (client.address.clone(), (Symbol::new(&env, "milestone_approved"), id).into_val(&env))
    );

    // Event 4: milestone_released — index + amount only
    let ev = events.get(4).unwrap();
    assert_eq!(
        ev.0,
        (client.address.clone(), (Symbol::new(&env, "milestone_released"), id).into_val(&env))
    );
    assert_eq!(ev.1, (0u32, 200_i128).into_val(&env));
}

// ─── Legacy combined test (kept for regression) ──────────────────────────────

#[test]
fn test_event_redaction_policy() {
    let env = Env::default();
    env.mock_all_auths();
    let client = register_client(&env);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);
    let milestone_amounts = vec![&env, 100_i128];
    let terms_hash = None;

    let id = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &Some(arbiter_addr.clone()),
        &milestone_amounts,
        &terms_hash,
        &None,
    );

    let events = env.events().all();
    let last_event = events.last().unwrap();

    assert_eq!(
        last_event.0,
        (client.address.clone(), (Symbol::new(&env, "contract_created"), id).into_val(&env))
    );

    client.deposit_funds(&id, &100_i128, &client_addr);
    let events = env.events().all();
    let last_event = events.last().unwrap();
    assert_eq!(last_event.1, (client_addr.clone(), 100_i128).into_val(&env));

    client.approve_milestone(&id, &0, &client_addr);

    client.release_milestone(&id, &0, &client_addr);
    let events = env.events().all();
    let last_event = events.last().unwrap();
    assert_eq!(last_event.1, (0u32, 100_i128).into_val(&env));

    client.cancel_contract(&id, &client_addr);
    let events = env.events().all();
    let last_event = events.last().unwrap();
    assert_eq!(
        last_event.1,
        (client_addr.clone(), ContractStatus::Cancelled, env.ledger().timestamp()).into_val(&env)
    );

    let id2 = client.create_contract(
        &client_addr,
        &freelancer_addr,
        &None,
        &milestone_amounts,
        &None,
        &None,
    );
    let work_hash = Bytes::from_array(&env, &[1u8; 32]);
    client.deposit_funds(&id2, &100_i128, &client_addr);
    client.submit_milestone_work(&id2, &0, &work_hash, &freelancer_addr);

    let events = env.events().all();
    let last_event = events.last().unwrap();

    assert_eq!(
        last_event.0,
        (client.address.clone(), (Symbol::new(&env, "work_submitted"), id2).into_val(&env))
    );
    assert_eq!(last_event.1, (0u32, work_hash).into_val(&env));
}
