# Path Conventions for Documentation

## Purpose
This document defines how to represent repository paths in committed documentation and metadata files.

## Rule
- Do not commit absolute filesystem paths that are specific to a local developer environment.
- Use `<REPO_ROOT>` as the standard placeholder for the repository root in documentation.

## Examples
- Correct: `| Application Code Root | <REPO_ROOT> |`
- Correct: `\`<REPO_ROOT>/src/\``
- Incorrect: `| Application Code Root | <ABSOLUTE_PATH> |`
- Incorrect: `\`<ABSOLUTE_PATH>/src\``

## Local usage
- If a developer needs to store a local path for convenience, keep it in an untracked file or local config that is not committed.
- Recommended pattern: use environment-specific configuration from `.env` or other ignored files, not source-controlled docs.

## Why
- Absolute paths vary per machine and create noise in diffs.
- They can leak environment details and cause confusion for collaborators.
- `<REPO_ROOT>` makes documentation reusable across all contributors.
