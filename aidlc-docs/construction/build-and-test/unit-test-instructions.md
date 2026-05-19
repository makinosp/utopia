# Unit Test Instructions

## Purpose
Validate unit-level behavior for auth flow, compatibility mappers, and error translation.

## Run Unit Tests

### 1. Execute All Unit Tests
```bash
cargo test --test auth_validator_test
cargo test --test error_mapper_test
cargo test --test pagination_test
cargo test --test decimal_serialization_test
cargo test --test token_lifecycle_test
cargo test
```

### 2. Review Test Results
- **Expected**: All unit tests pass, 0 failures
- **Expected coverage**: Target >= 80% for core auth and error mapping paths
- **Test report location**: Console output from `cargo test`
- **Optional machine output**:
```bash
cargo test --message-format=json > target/unit-test-report.json
```

### 3. Fix Failing Tests
If tests fail:
1. Review test output in console
2. Identify failing assertion and failing test cases
3. Fix code issues in `src/core` or `src/api` modules
4. Re-run the specific failing test target
5. Re-run the full unit test suite before merge

### 4. Coverage Guidance
If using cargo-llvm-cov:
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --lcov --output-path target/lcov.info
```

Target baseline:
- Core auth and error mapping paths should remain above 80 percent line coverage
