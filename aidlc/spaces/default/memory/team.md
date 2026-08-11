# Team-Level Rules

> This team's affirmed practices and corrections. Loaded after `org.md` as
> strict-additive guidance; contradictions with broader policy are rejected.
> Populated by the practices-discovery affirmation gate. Edit at the gate,
> not directly.

## Way of Working

### Question Format (Migrated from v1 `.aidlc-rule-details/common/`)

All questions must use structured question files — never ask in chat.

- File naming: `{phase-name}-questions.md`
- Every question: clear text + meaningful options + mandatory "Other (please describe…)" as last option
- Options separated by blank lines (CommonMark rendering)
- Answer via `[Answer]: <letter>` tag
- Workflow: create file → inform user → wait for confirmation → read & extract → proceed
- If answer missing or invalid: ask user to fix before proceeding
- Minimum 2 meaningful options + Other; maximum 5 + Other
- Options must be mutually exclusive, cover common scenarios, realistic, specific

## Walking Skeleton

<!-- Affirmed during practices-discovery. Example: -->
<!-- We don't run a walking skeleton — our deployment pipeline is mature -->
<!-- and the slice cost outweighs the value at our maturity stage. -->

## Testing Posture

<!-- Affirmed during practices-discovery. Example: -->
<!-- We use BDD. Specifications drive scenarios; scenarios drive code. -->
<!-- Each Unit ships with feature files in /features/. -->

## Deployment

<!-- Affirmed during practices-discovery. -->

## Code Style

<!-- Team-specific conventions beyond the linter. Example: -->
<!-- - Prefer named exports over default exports -->
<!-- - All async functions return Result<T, E>, never throw -->

### ASCII Diagram Standards (Migrated from v1 `.aidlc-rule-details/common/`)

- **Basic ASCII only** for diagrams: `+` `-` `|` `^` `v` `<` `>` and alphanumeric text
- **NO Unicode box-drawing** (`┌ ─ │ └ ┐ ┘ ├ ┤ ┬ ┴ ┼ ▼ ▲ ► ◄`)
- **Every line in a box MUST have EXACTLY the same character count** (including spaces)
- Corners use `+`; use spaces (not tabs) for alignment
- For complex diagrams, prefer Mermaid

### Content Validation (Migrated from v1 `.aidlc-rule-details/common/`)

Before writing any file with diagrams or code blocks:

- Validate embedded code blocks (Mermaid, JSON, YAML)
- Check special character escaping (`"` → `\"`, `'` → `\'`)
- Verify markdown syntax correctness
- Include text fallback for any Mermaid diagram
- On validation failure: log error → use fallback → continue workflow → inform user

## Forbidden

<!-- Team-specific forbidden patterns -->

## Mandated

### Overconfidence Prevention (Migrated from v1 `.aidlc-rule-details/common/`)

**Default to asking** — when there's any ambiguity, ask clarifying questions.

- Evaluate ALL question categories; don't skip entire areas
- Analyze ALL user responses for ambiguities (vague words: "depends", "maybe", "not sure", "mix of")
- Create follow-up questions for ANY unclear responses; don't proceed until resolved
- Red flags: stages completing with zero questions on complex projects; proceeding with vague answers
- Better to over-clarify than under-clarify

## Corrections

<!-- Self-learning loop appends here. -->
