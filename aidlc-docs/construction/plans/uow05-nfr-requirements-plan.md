# NFR Requirements Plan — UOW-05: Compatibility Verification Suite

## Prerequisites
- Functional Design: SKIPPED (test infrastructure only, no application business logic or data models)
- NFR sources: `aidlc-docs/inception/requirements/uow05-requirements.md` (NFR-001 to NFR-004)
- Execution plan: NFR Requirements is the first stage in the construction loop

## Part 1 - Planning Checklist
- [x] Analyze NFR sources from requirements document
- [x] Generate context-appropriate questions for NFR refinement
- [x] Collect and analyze answers
- [x] Generate nfr-requirements.md and tech-stack-decisions.md
- [x] Move stage to awaiting approval

## Part 2 - Generation Execution Checklist
- [x] Read approved plan and identify first uncompleted generation step
- [x] Generate nfr-requirements.md
- [x] Mark step complete
- [x] Generate tech-stack-decisions.md
- [x] Mark step complete
- [x] Validate NFR requirements artifacts
- [ ] Mark NFR Requirements stage complete (awaiting user approval)

---

## NFR Assessment Questions

Please fill all `[Answer]:` fields.

### Question 1: Performance Threshold Refinement

NFR-001 specifies a target of < 2 minutes for the full suite. Given that the suite covers 12+ endpoints across Auth, Accounts, and Transactions, should the threshold be:

A) < 2 minutes (keep as-is — a conservative target acceptable for CI)
B) < 1 minute (target faster execution with parallel k6 scenarios)
C) < 3 minutes (allow headroom for CI overhead and slower environments)
D) Other (please specify after [Answer]:)

[Answer]: A

### Question 2: k6 Execution Mode

k6 supports multiple execution modes. Which mode best suits the compatibility suite?

A) **Shared iteration** — single VU processes all test scenarios sequentially (simplest, deterministic)
B) **Per-VU iteration** — each VU runs the full scenario loop independently (good for load simulation)
C) **Constant arrival rate** — fixed number of iterations per second (useful for soak/load testing)
D) Other (please specify after [Answer]:)

[Answer]: A

### Question 3: Concurrency and Virtual Users

Should the test suite use a single Virtual User (VU) or multiple VUs?

A) **1 VU** — simple sequential execution, easier to debug
B) **2-3 VUs** — limited concurrency to reduce wall-clock time
C) **5+ VUs** — maximize speed, but may increase flakiness due to shared DB state
D) Other (please specify after [Answer]:)

[Answer]: A

### Question 4: Database Reset Strategy (NFR-003)

What mechanism should be used to ensure each test run starts from a known database state?

A) **Drop + recreate database** before test run, then apply migrations and seed data
B) **Truncate all tables** and re-insert seed data (faster, avoids DDL operations)
C) **Use Docker Compose `docker compose down -v && docker compose up -d`** to reset the entire stack
D) Other (please specify after [Answer]:)

[Answer]: B

### Question 5: Observability Detail Level (NFR-004)

What level of detail should k6 report on test completion?

A) **Summary only** — pass/fail count per endpoint group, total duration
B) **Per-test detail** — pass/fail per individual test case, with timing breakdown
C) **Full output** — includes request/response diffs on failure for strict mode comparison
D) Other (please specify after [Answer]:)

[Answer]: C

### Question 6: CI Run Frequency (FR-007)

When should the k6 suite execute in CI?

A) **On every pull request** — full suite (may add ~2 min to CI time)
B) **On every pull request** — smoke tests only; full suite on push to main
C) **On pull request labeled `compatibility-check`** — opt-in trigger
D) Other (please specify after [Answer]:)

[Answer]: A

### Question 7: k6 Output Artifacts

What output artifacts should be retained from k6 test runs?

A) **JSON summary report** only (machine-parseable)
B) **HTML report** only (human-readable)
C) **Both JSON and HTML** reports
D) **No artifacts** — just console output
E) Other (please specify after [Answer]:)

[Answer]: C

### Question 8: Seed Data Versioning

How should seed SQL data be managed as the schema evolves?

A) **Single seed file** (`seed.sql`) updated in-place as schema changes
B) **Versioned seed files** (`seed-v1.sql`, `seed-v2.sql`) — keep a history
C) **Generate seed data programmatically** from a script (avoids drift)
D) Other (please specify after [Answer]:)

[Answer]: C — Generate seed data programmatically using TypeScript with Bun runtime. Unify the language with k6 test scripts to share type definitions. Place under `scripts/seed/` and execute via `bun run scripts/seed/index.ts`.

---

[Answer]: Ready to proceed to NFR Requirements stage with the above plan.
