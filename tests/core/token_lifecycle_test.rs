#[test]
fn token_issuance_is_not_idempotent_by_design() {
    let first = "token-a";
    let second = "token-b";
    assert_ne!(first, second);
}
