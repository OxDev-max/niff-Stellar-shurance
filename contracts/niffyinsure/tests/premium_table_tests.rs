#![cfg(test)]

use niffyinsure::{
    premium::{
        admin_set_premium_multiplier, compute_premium, default_multiplier_table, MAX_MULTIPLIER,
        MAX_SAFETY_DISCOUNT, MIN_MULTIPLIER,
    },
    types::{AgeBand, CoverageTier, MultiplierKey, RegionTier, RiskInput},
};
use soroban_sdk::Env;

#[test]
fn default_table_matches_known_reference_vectors() {
    let env = Env::default();
    let table = default_multiplier_table(&env);

    let cases = [
        (
            RiskInput {
                region: RegionTier::Medium,
                age_band: AgeBand::Adult,
                coverage: CoverageTier::Standard,
                safety_score: 0,
            },
            10_000_000i128,
            10_000_000i128,
        ),
        (
            RiskInput {
                region: RegionTier::High,
                age_band: AgeBand::Young,
                coverage: CoverageTier::Premium,
                safety_score: 80,
            },
            12_345_678i128,
            22_749_999i128,
        ),
        (
            RiskInput {
                region: RegionTier::Low,
                age_band: AgeBand::Senior,
                coverage: CoverageTier::Basic,
                safety_score: 100,
            },
            5_000_000i128,
            3_519_000i128,
        ),
    ];

    for (input, base_amount, expected_total) in cases {
        let computation = compute_premium(&input, base_amount, &table).unwrap();
        assert_eq!(computation.total_premium, expected_total);
    }
}

// ── admin_set_premium_multiplier tests ────────────────────────────────────────

/// Helper: set up a contract env with admin + initialized multiplier table.
fn setup_env_with_table() -> (Env, soroban_sdk::Address) {
    use niffyinsure::storage;
    let env = Env::default();
    env.mock_all_auths();
    let admin = soroban_sdk::Address::generate(&env);
    storage::set_admin(&env, &admin);
    storage::set_multiplier_table(&env, &default_multiplier_table(&env));
    (env, admin)
}

#[test]
fn valid_region_multiplier_update_takes_effect_immediately() {
    let (env, _admin) = setup_env_with_table();

    // Update High region from 13_500 → 15_000
    admin_set_premium_multiplier(&env, MultiplierKey::Region(RegionTier::High), 15_000)
        .expect("valid update should succeed");

    let table = niffyinsure::storage::get_multiplier_table(&env);
    assert_eq!(
        table.region.get(RegionTier::High).unwrap(),
        15_000,
        "region multiplier should be updated in storage"
    );

    // Premium calculation must use the new value immediately
    let input = RiskInput {
        region: RegionTier::High,
        age_band: AgeBand::Adult,
        coverage: CoverageTier::Standard,
        safety_score: 0,
    };
    let computation = compute_premium(&input, 10_000_000, &table).unwrap();
    // With High=15_000, Adult=10_000, Standard=10_000, safety_discount=2_000, score=0
    // result = base * (15_000/10_000) * (10_000/10_000) * (10_000/10_000) = 15_000_000
    assert_eq!(computation.total_premium, 15_000_000);
}

#[test]
fn valid_age_multiplier_update_takes_effect_immediately() {
    let (env, _admin) = setup_env_with_table();

    admin_set_premium_multiplier(&env, MultiplierKey::Age(AgeBand::Young), 11_000)
        .expect("valid update should succeed");

    let table = niffyinsure::storage::get_multiplier_table(&env);
    assert_eq!(table.age.get(AgeBand::Young).unwrap(), 11_000);
}

#[test]
fn valid_coverage_multiplier_update_takes_effect_immediately() {
    let (env, _admin) = setup_env_with_table();

    admin_set_premium_multiplier(&env, MultiplierKey::Coverage(CoverageTier::Basic), 9_500)
        .expect("valid update should succeed");

    let table = niffyinsure::storage::get_multiplier_table(&env);
    assert_eq!(table.coverage.get(CoverageTier::Basic).unwrap(), 9_500);
}

#[test]
fn valid_safety_discount_update_takes_effect_immediately() {
    let (env, _admin) = setup_env_with_table();

    admin_set_premium_multiplier(&env, MultiplierKey::SafetyDiscount, 3_000)
        .expect("valid update should succeed");

    let table = niffyinsure::storage::get_multiplier_table(&env);
    assert_eq!(table.safety_discount, 3_000);
}

#[test]
fn region_multiplier_above_max_is_rejected() {
    let (env, _admin) = setup_env_with_table();

    let result = admin_set_premium_multiplier(
        &env,
        MultiplierKey::Region(RegionTier::Low),
        MAX_MULTIPLIER + 1,
    );
    assert_eq!(
        result,
        Err(niffyinsure::validate::Error::RegionMultiplierOutOfBounds),
        "value above MAX_MULTIPLIER must be rejected"
    );
}

#[test]
fn region_multiplier_below_min_is_rejected() {
    let (env, _admin) = setup_env_with_table();

    let result = admin_set_premium_multiplier(
        &env,
        MultiplierKey::Region(RegionTier::Medium),
        MIN_MULTIPLIER - 1,
    );
    assert_eq!(
        result,
        Err(niffyinsure::validate::Error::RegionMultiplierOutOfBounds),
        "value below MIN_MULTIPLIER must be rejected"
    );
}

#[test]
fn age_multiplier_out_of_bounds_is_rejected() {
    let (env, _admin) = setup_env_with_table();

    let result = admin_set_premium_multiplier(
        &env,
        MultiplierKey::Age(AgeBand::Senior),
        MAX_MULTIPLIER + 500,
    );
    assert_eq!(
        result,
        Err(niffyinsure::validate::Error::AgeMultiplierOutOfBounds)
    );
}

#[test]
fn coverage_multiplier_out_of_bounds_is_rejected() {
    let (env, _admin) = setup_env_with_table();

    let result = admin_set_premium_multiplier(
        &env,
        MultiplierKey::Coverage(CoverageTier::Premium),
        MIN_MULTIPLIER - 1,
    );
    assert_eq!(
        result,
        Err(niffyinsure::validate::Error::CoverageMultiplierOutOfBounds)
    );
}

#[test]
fn safety_discount_above_max_is_rejected() {
    let (env, _admin) = setup_env_with_table();

    let result =
        admin_set_premium_multiplier(&env, MultiplierKey::SafetyDiscount, MAX_SAFETY_DISCOUNT + 1);
    assert_eq!(
        result,
        Err(niffyinsure::validate::Error::SafetyDiscountOutOfBounds)
    );
}

#[test]
fn safety_discount_negative_is_rejected() {
    let (env, _admin) = setup_env_with_table();

    let result = admin_set_premium_multiplier(&env, MultiplierKey::SafetyDiscount, -1);
    assert_eq!(
        result,
        Err(niffyinsure::validate::Error::SafetyDiscountOutOfBounds)
    );
}

#[test]
fn boundary_values_at_min_and_max_are_accepted() {
    let (env, _admin) = setup_env_with_table();

    admin_set_premium_multiplier(&env, MultiplierKey::Region(RegionTier::Low), MIN_MULTIPLIER)
        .expect("MIN_MULTIPLIER should be accepted");

    admin_set_premium_multiplier(&env, MultiplierKey::Age(AgeBand::Adult), MAX_MULTIPLIER)
        .expect("MAX_MULTIPLIER should be accepted");

    admin_set_premium_multiplier(&env, MultiplierKey::SafetyDiscount, 0)
        .expect("zero safety discount should be accepted");

    admin_set_premium_multiplier(&env, MultiplierKey::SafetyDiscount, MAX_SAFETY_DISCOUNT)
        .expect("MAX_SAFETY_DISCOUNT should be accepted");
}

#[test]
fn storage_unchanged_after_out_of_bounds_rejection() {
    let (env, _admin) = setup_env_with_table();

    let table_before = niffyinsure::storage::get_multiplier_table(&env);
    let original_high = table_before.region.get(RegionTier::High).unwrap();

    // Attempt an invalid update
    let _ = admin_set_premium_multiplier(
        &env,
        MultiplierKey::Region(RegionTier::High),
        MAX_MULTIPLIER + 9999,
    );

    let table_after = niffyinsure::storage::get_multiplier_table(&env);
    assert_eq!(
        table_after.region.get(RegionTier::High).unwrap(),
        original_high,
        "storage must not be modified on rejection"
    );
}
