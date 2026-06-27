# UOW-05: Compatibility Verification Suite — Requirements Clarification

Please answer the following questions to clarify the requirements for the new Unit of Work.

---

## Question 1
Which Firefly-III version should be the target for compatibility?

A) Firefly-III v6.x (latest stable)

B) Firefly-III v5.x

C) Version-flexible approach (configurable via k6 test parameters)

D) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
What is the scope of API endpoints to verify for compatibility?

A) All API endpoints (implemented + future endpoints)

B) Currently implemented endpoints only (Accounts, Transactions, Auth)

C) Start with read-only endpoints first, then expand gradually

D) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 3
What is the source of Golden Response data for k6 tests?

A) Record actual API responses from a live Firefly-III instance

B) Manually define expected response schemas from Firefly-III OpenAPI spec

C) Use Firefly-III community test fixtures

D) Other (please describe after [Answer]: tag below)

[Answer]: B

**Note**: This approach combines OpenAPI schema validation with separately defined sample response JSON files for strict mode testing.

---

## Question 4
What is the execution environment for k6 tests?

A) Add k6 container to Docker Compose; run in CI (GitHub Actions) with same environment

B) Install k6 locally and run (CI integration later)

C) Create a dedicated test Docker Compose file for isolated environment

D) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 5
How should the pass/fail criteria be set for compatibility tests?

A) Strict mode: Full JSON response match (excluding dynamic fields)

B) Partial match mode: Status code + required field presence + data type consistency

C) Hybrid: Strict match for critical endpoints, partial for others

D) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 6
When should the k6 test suite be integrated into CI (GitHub Actions)?

A) From the beginning (automatically run on PR creation)

B) Integrate manually after test suite completion

C) Out of scope for this UOW; handle separately in a later UOW

D) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 7
What is the strategy for test data management in this UOW?

A) Generate test data dynamically within k6 scripts (unique per execution)

B) Prepare seed data in advance and load into DB before tests

C) Combine both approaches (seed data as baseline, dynamic generation where needed)

D) Other (please describe after [Answer]: tag below)

[Answer]: B
