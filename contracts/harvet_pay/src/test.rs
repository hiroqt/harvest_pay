#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Env,
};

/// Deploys a standard Stellar asset contract to stand in for USDC in tests.
fn create_token_contract<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let address = sac.address();
    (
        TokenClient::new(env, &address),
        StellarAssetClient::new(env, &address),
    )
}

/// Shared fixture: env, deployed HarvestPay contract, funded buyer, and a farmer wallet.
fn setup<'a>() -> (
    Env,
    HarvestPayContractClient<'a>,
    Address,
    Address,
    TokenClient<'a>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let farmer = Address::generate(&env);

    let (token_client, token_admin_client) = create_token_contract(&env, &admin);
    token_admin_client.mint(&buyer, &1_000_0000000);

    let contract_id = env.register(HarvestPayContract, ());
    let client = HarvestPayContractClient::new(&env, &contract_id);

    (env, client, buyer, farmer, token_client)
}

/// Test 1 (Happy path): full create -> confirm flow pays the farmer in one demo-able round trip.
#[test]
fn test_happy_path_create_and_confirm() {
    let (_env, client, buyer, farmer, token) = setup();
    let amount: i128 = 500_0000000;

    let id = client.create_escrow(&buyer, &farmer, &token.address, &amount);
    client.confirm_delivery(&id, &buyer);

    assert_eq!(token.balance(&farmer), amount);
}

/// Test 2 (Edge case): a stranger cannot confirm delivery on someone else's escrow.
#[test]
fn test_unauthorized_confirm_fails() {
    let (env, client, buyer, farmer, token) = setup();
    let amount: i128 = 100_0000000;
    let id = client.create_escrow(&buyer, &farmer, &token.address, &amount);

    let stranger = Address::generate(&env);
    let result = client.try_confirm_delivery(&id, &stranger);

    assert!(result.is_err());
}

/// Test 3 (State verification): after delivery is confirmed, storage reflects Delivered status
/// with the correct amount preserved.
#[test]
fn test_state_after_delivery() {
    let (_env, client, buyer, farmer, token) = setup();
    let amount: i128 = 250_0000000;
    let id = client.create_escrow(&buyer, &farmer, &token.address, &amount);
    client.confirm_delivery(&id, &buyer);

    let escrow = client.get_escrow(&id);

    assert_eq!(escrow.amount, amount);
    assert_eq!(escrow.status, EscrowStatus::Delivered);
}

/// Test 4 (Edge case): an already-delivered escrow cannot be confirmed a second time,
/// preventing double payout.
#[test]
fn test_double_confirm_fails() {
    let (_env, client, buyer, farmer, token) = setup();
    let amount: i128 = 100_0000000;
    let id = client.create_escrow(&buyer, &farmer, &token.address, &amount);
    client.confirm_delivery(&id, &buyer);

    let result = client.try_confirm_delivery(&id, &buyer);

    assert!(result.is_err());
}

/// Test 5 (State verification): cancelling a pending escrow refunds the buyer in full
/// and marks the escrow Cancelled.
#[test]
fn test_cancel_refunds_buyer() {
    let (_env, client, buyer, farmer, token) = setup();
    let amount: i128 = 300_0000000;
    let id = client.create_escrow(&buyer, &farmer, &token.address, &amount);

    let balance_before = token.balance(&buyer);
    client.cancel_escrow(&id, &buyer);
    let balance_after = token.balance(&buyer);

    let escrow = client.get_escrow(&id);

    assert_eq!(balance_after, balance_before + amount);
    assert_eq!(escrow.status, EscrowStatus::Cancelled);
}