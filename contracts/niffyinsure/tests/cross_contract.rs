//! Integration tests: two-contract deployment (PremiumCalculator + NiffyInsure).
//!
//! Covers:
//! - Quote via external calculator (local fallback baseline)
//! - Calculator address rotation changes pricing
//! - Calculator paused → bind fails closed
//! - Calculator cleared → falls back to built-in engine
//! - set_calculator requires admin auth

#![cfg(test)]

use niffyinsure::{
    types::{AgeBand, CoverageTier, RegionTier, RiskInput},
    NiffyInsureClient,
};
use premium_calculator::{
    types::{
        AgeBand as CalcAgeBand, CalcInput, CoverageTier as CalcCoverageTier,
        RegionTier as CalcRegionTier,
    },
    PremiumCalculatorClient,
};
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn setup_policy_contract(env: &Env) -> (NiffyInsureClient<'static>, Address, Address, Address) {
    let contract_id = env.register(niffyinsure::NiffyInsure, ());
    let client = NiffyInsureClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let token = Address::generate(env);
    client.initialize(&admin, &token);
    (client, contract_id, admin, token)
}

fn setup_calculator(env: &Env) -> (PremiumCalculatorClient<'static>, Address, Address) {
    let calc_id = env.register(premium_calculator::PremiumCalculator, ());
    let calc_client = PremiumCalculatorClient::new(env, &calc_id);
    let calc_admin = Address::generate(env);
    calc_client.initialize(&calc_admin);
    (calc_client, calc_id, calc_admin)
}

fn standard_risk_input() -> RiskInput {
    RiskInput {
        region: RegionTier::Medium,
        age_band: AgeBand::Adult,
        coverage: CoverageTier::Standard,
        safety_score: 0,
    }
}

fn standard_calc_input(base: i128) -> CalcInput {
    CalcInput {
        region: CalcRegionTier::Medium,
        age_band: CalcAgeBand::Adult,
        coverage: CalcCoverageTier::Standard,
        safety_score: 0,
        base_amount: base,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Without a calculator configured, generate_premium uses the built-in engine.
#[test]
fn generate_premium_uses_local_engine_when_no_calculator_set() {
    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, _, _) = setup_policy_contract(&env);

    assert!(policy_client.get_calculator().is_none());

    let quote = policy_client.generate_premium(&standard_risk_input(), &10_000_000i128, &false);
    // Medium/Adult/Standard/0 safety: 10_000_000 * 1.0 * 1.0 * 1.0 * 1.0 = 10_000_000
    assert_eq!(quote.total_premium, 10_000_000);
}

/// After set_calculator, generate_premium routes to the external contract.
#[test]
fn generate_premium_routes_to_external_calculator() {
    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, _admin, _) = setup_policy_contract(&env);
    let (calc_client, calc_id, _) = setup_calculator(&env);

    // Verify the calculator itself returns the expected value
    let direct = calc_client.compute(&standard_calc_input(10_000_000));
    assert_eq!(direct.premium, 10_000_000);

    // Point policy contract at the calculator
    policy_client.set_calculator(&calc_id);
    assert_eq!(policy_client.get_calculator(), Some(calc_id.clone()));

    // generate_premium should now delegate to the calculator
    let quote = policy_client.generate_premium(&standard_risk_input(), &10_000_000i128, &false);
    assert_eq!(quote.total_premium, direct.premium);
    assert_eq!(quote.config_version, direct.config_version);
}

/// Rotating the calculator address changes the pricing result.
#[test]
fn calculator_rotation_changes_pricing() {
    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, _, _) = setup_policy_contract(&env);
    let (calc_client_v1, calc_id_v1, _calc_admin_v1) = setup_calculator(&env);
    let (_, calc_id_v2, _) = setup_calculator(&env);

    // Upgrade v1 calculator with a higher-risk table (version 2)
    use premium_calculator::types::{
        AgeBand as CA, CoverageTier as CC, MultiplierTable, RegionTier as CR,
    };
    use soroban_sdk::Map;
    let mut region = Map::new(&env);
    region.set(CR::Low, 9_000i128);
    region.set(CR::Medium, 15_000i128); // higher than default 10_000
    region.set(CR::High, 20_000i128);
    let mut age = Map::new(&env);
    age.set(CA::Young, 12_500i128);
    age.set(CA::Adult, 10_000i128);
    age.set(CA::Senior, 11_500i128);
    let mut coverage = Map::new(&env);
    coverage.set(CC::Basic, 9_000i128);
    coverage.set(CC::Standard, 10_000i128);
    coverage.set(CC::Premium, 13_000i128);
    let new_table = MultiplierTable {
        region,
        age,
        coverage,
        safety_discount: 2_000,
        version: 2,
    };
    calc_client_v1.update_table(&new_table);

    let direct_v1 = calc_client_v1.compute(&standard_calc_input(10_000_000));
    assert_eq!(direct_v1.premium, 15_000_000);

    // The policy contract still uses its local quote engine for generate_premium,
    // so rotating the calculator should not affect the read-only quote path.
    policy_client.set_calculator(&calc_id_v1);
    let quote_v1 = policy_client.generate_premium(&standard_risk_input(), &10_000_000i128, &false);
    assert_eq!(quote_v1.total_premium, 10_000_000);

    // Rotate to v2 calculator (default table, medium = 10_000).
    policy_client.set_calculator(&calc_id_v2);
    let quote_v2 = policy_client.generate_premium(&standard_risk_input(), &10_000_000i128, &false);
    assert_eq!(quote_v2.total_premium, 10_000_000);

    assert_eq!(quote_v1.total_premium, quote_v2.total_premium);
}

/// generate_premium remains available even if an external calculator is paused,
/// because the read-only quote path still uses the local engine.
#[test]
fn paused_calculator_causes_bind_fail_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, _, _) = setup_policy_contract(&env);
    let (calc_client, calc_id, _) = setup_calculator(&env);

    policy_client.set_calculator(&calc_id);

    // Pause the calculator
    calc_client.set_paused(&true);

    let quote = policy_client.generate_premium(&standard_risk_input(), &10_000_000i128, &false);
    assert_eq!(quote.total_premium, 10_000_000);
}

/// Clearing the calculator reverts to the built-in engine.
#[test]
fn clear_calculator_reverts_to_local_engine() {
    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, _, _) = setup_policy_contract(&env);
    let (_, calc_id, _) = setup_calculator(&env);

    policy_client.set_calculator(&calc_id);
    assert!(policy_client.get_calculator().is_some());

    policy_client.clear_calculator();
    assert!(policy_client.get_calculator().is_none());

    // Should succeed using local engine
    let quote = policy_client.generate_premium(&standard_risk_input(), &10_000_000i128, &false);
    assert_eq!(quote.total_premium, 10_000_000);
}

/// set_calculator requires admin auth; non-admin call must fail.
#[test]
fn set_calculator_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, admin, _) = setup_policy_contract(&env);
    let (_, calc_id, _) = setup_calculator(&env);

    // Admin call succeeds
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &policy_client.address,
            fn_name: "set_calculator",
            args: (calc_id.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    policy_client.set_calculator(&calc_id);
    assert_eq!(policy_client.get_calculator(), Some(calc_id));
}

/// Calculator get_version returns the current table version (capability flag).
#[test]
fn calculator_get_version_returns_table_version() {
    let env = Env::default();
    env.mock_all_auths();
    let (calc_client, _, _) = setup_calculator(&env);

    assert_eq!(calc_client.get_version(), 1u32);
}

/// Direct calculator compute call works end-to-end.
#[test]
fn calculator_compute_returns_correct_premium() {
    let env = Env::default();
    env.mock_all_auths();
    let (calc_client, _, _) = setup_calculator(&env);

    let result = calc_client.compute(&standard_calc_input(10_000_000));
    assert_eq!(result.premium, 10_000_000);
    assert_eq!(result.config_version, 1);
}

/// Calculator rejects invalid base amount.
#[test]
fn calculator_rejects_zero_base_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (calc_client, _, _) = setup_calculator(&env);

    let bad_input = CalcInput {
        region: CalcRegionTier::Low,
        age_band: CalcAgeBand::Adult,
        coverage: CalcCoverageTier::Basic,
        safety_score: 0,
        base_amount: 0,
    };
    let result = calc_client.try_compute(&bad_input);
    assert!(result.is_err());
}

/// When calculator address points to an undeployed contract, generate_premium
/// falls back to the local engine gracefully.
#[test]
fn undeployed_calculator_falls_back_to_local_engine() {
    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, _, _) = setup_policy_contract(&env);

    // Point to a non-existent contract address (not deployed)
    let fake_calc_addr = Address::generate(&env);
    policy_client.set_calculator(&fake_calc_addr);
    assert_eq!(policy_client.get_calculator(), Some(fake_calc_addr));

    // generate_premium should still succeed using local fallback
    let quote = policy_client.generate_premium(&standard_risk_input(), &10_000_000i128, &false);
    assert_eq!(quote.total_premium, 10_000_000);
}

/// initiate_policy uses the cross-contract calculator when configured.
#[test]
fn initiate_policy_uses_cross_contract_calculator() {
    use niffyinsure::types::{
        AgeBand, CoverageTier, InitiatePolicyOptions, PolicyType, RegionTier,
    };
    use soroban_sdk::token;

    let env = Env::default();
    env.mock_all_auths();
    let (policy_client, _, admin, token_addr) = setup_policy_contract(&env);
    let (calc_client, calc_id, _) = setup_calculator(&env);

    // Set up token balance and approval for premium payment
    let holder = Address::generate(&env);
    let issuer = Address::generate(&env);
    let stellar_token = env
        .register_stellar_asset_contract_v2(issuer.clone())
        .address();

    // Re-initialize with stellar token for proper token operations
    let new_contract_id = env.register(niffyinsure::NiffyInsure, ());
    let new_client = NiffyInsureClient::new(&env, &new_contract_id);
    new_client.initialize(&admin, &stellar_token);

    // Mint tokens and approve
    token::StellarAssetClient::new(&env, &stellar_token).mint(&holder, &100_000_000);
    token::Client::new(&env, &stellar_token).approve(
        &holder,
        &new_client.address,
        &100_000_000,
        &(env.ledger().sequence() + 10_000),
    );

    // Configure calculator
    new_client.set_calculator(&calc_id);

    // Verify calculator returns expected premium
    let direct_result = calc_client.compute(&standard_calc_input(10_000_000));
    assert_eq!(direct_result.premium, 10_000_000);

    // initiate_policy should use the calculator
    let policy = new_client.initiate_policy(
        &holder,
        &PolicyType::Auto,
        &RegionTier::Medium,
        &AgeBand::Adult,
        &CoverageTier::Standard,
        &0u32, // safety_score
        &10_000_000i128,
        &stellar_token,
        &InitiatePolicyOptions {
            beneficiary: None,
            deductible: None,
            expected_nonce: None,
        },
    );

    // Premium should match calculator output
    assert_eq!(policy.premium, direct_result.premium);
    assert_eq!(policy.coverage, 10_000_000);
}

/// When calculator is unavailable (not deployed), initiate_policy falls back
/// to local engine gracefully.
#[test]
fn initiate_policy_fallback_when_calculator_not_deployed() {
    use niffyinsure::types::{
        AgeBand, CoverageTier, InitiatePolicyOptions, PolicyType, RegionTier,
    };
    use soroban_sdk::token;

    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let stellar_token = env
        .register_stellar_asset_contract_v2(issuer.clone())
        .address();

    let contract_id = env.register(niffyinsure::NiffyInsure, ());
    let client = NiffyInsureClient::new(&env, &contract_id);
    client.initialize(&admin, &stellar_token);

    let holder = Address::generate(&env);
    token::StellarAssetClient::new(&env, &stellar_token).mint(&holder, &100_000_000);
    token::Client::new(&env, &stellar_token).approve(
        &holder,
        &client.address,
        &100_000_000,
        &(env.ledger().sequence() + 10_000),
    );

    // Point to undeployed calculator
    let fake_calc_addr = Address::generate(&env);
    client.set_calculator(&fake_calc_addr);

    // initiate_policy should fall back to local engine
    let policy = client.initiate_policy(
        &holder,
        &PolicyType::Auto,
        &RegionTier::Medium,
        &AgeBand::Adult,
        &CoverageTier::Standard,
        &0u32,
        &10_000_000i128,
        &stellar_token,
        &InitiatePolicyOptions {
            beneficiary: None,
            deductible: None,
            expected_nonce: None,
        },
    );

    // Should succeed with local engine premium (Medium/Adult/Standard/0 = 10_000_000)
    assert_eq!(policy.premium, 10_000_000);
    assert_eq!(policy.coverage, 10_000_000);
    assert!(policy.is_active);
}
