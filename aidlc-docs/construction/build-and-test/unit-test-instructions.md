# Unit Test Instructions

## Purpose
Validate unit-level behavior for auth flow, compatibility mappers, and error translation.

## Run Unit Tests

### 1. Execute Full Unit Test Suite
```bash
cargo test --test auth_validator_test
cargo test --test error_mapper_test
cargo test --test pagination_test
cargo test --test decimal_serialization_test
cargo test --test token_lifecycle_test
cargo test
```

### 2. Review Results
Expected:
- All unit tests pass
- No panics
- Property-based decimal round-trip tests pass consistently

Test report locations:
- Console output from cargo test
- Optional machine output:
```bash
cargo test --message-format=json > target/unit-test-report.json
```

### 3. Coverage Guidance
If using cargo-llvm-cov:
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --lcov --output-path target/lcov.info
```

Target baseline:
- Core auth and error mapping paths should remain above 80 percent line coverage

## If Tests Fail
1. Identify failing assertion in test output
2. Fix source in src/core or src/api modules
3. Re-run specific test target
4. Re-run full cargo test before merge
