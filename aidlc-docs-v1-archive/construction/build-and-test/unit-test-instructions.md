# Unit Test Instructions

## Purpose
Validate unit-level behavior for auth flow, compatibility mappers, error translation, and Firefly-III contract compliance.

## Run Unit Tests

### 1. Execute All Unit Tests
```bash
cargo test --test auth_validator_test
cargo test --test error_mapper_test
cargo test --test pagination_test
cargo test --test decimal_serialization_test
cargo test --test token_lifecycle_test
cargo test --test firefly_error_contract_test
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

### 3. UOW-05 Compatibility Contract Tests
The following unit tests validate Firefly-III response contract compliance:

| Test File | Validates |
|-----------|-----------|
| `core/firefly_error_contract_test.rs` | Error envelope format matches Firefly-III spec |
| `core/auth_validator_test.rs` | Token format and validation logic |
| `core/decimal_serialization_test.rs` | Amount serialization for financial precision |
| `core/pagination_test.rs` | Pagination meta structure |

### 4. Fix Failing Tests
If tests fail:
1. Review test output in console
2. Identify failing assertion and failing test cases
3. Fix code issues in `src/core` or `src/api` modules
4. Re-run the specific failing test target
5. Re-run the full unit test suite before merge

### 5. Coverage Guidance
If using cargo-llvm-cov:
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --lcov --output-path target/lcov.info
```

Target baseline:
- Core auth and error mapping paths should remain above 80 percent line coverage
