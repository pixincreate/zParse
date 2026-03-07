# zParse Refactor: Test Migration + JSONC Support

## TL;DR

> **Quick Summary**: Consolidate all scattered inline tests into the integration test directory (`crates/zparse/tests/`), then add `.jsonc` file extension support across library, CLI, and API so that JSONC files automatically parse with comments and trailing commas enabled.
> 
> **Deliverables**:
> - All 13 inline `#[cfg(test)]` blocks removed from source files
> - All unit tests migrated to `crates/zparse/tests/` as integration tests
> - Dead code `src/tests/` directory and `src/tests.rs` deleted
> - `.jsonc` auto-detection in library, CLI, and API
> - JSONC tests validating the new functionality
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: YES — 4 waves
> **Critical Path**: Task 1 (baseline) → Task 2 (getters) → Tasks 3-7 (parallel migration) → Task 8 (cleanup) → Tasks 9-11 (parallel JSONC) → Task 12 (JSONC tests) → Final Verification

---

## Context

### Original Request
Two-part refactor:
1. Move internal tests out of `crates/zparse/src/` — inline `#[cfg(test)]` blocks in 13 source files need to be removed since they've been migrated to `crates/zparse/src/tests/`. Consolidate all tests in one place (`crates/zparse/tests/`).
2. Add JSONC support — zParse already parses trailing commas and comments. Add `.jsonc` file extension detection so it auto-enables these flags across library, CLI, and API.

### Interview Summary
**Key Discussions**:
- **Test location**: User confirmed tests should move to `crates/zparse/tests/` (integration tests). All items used by tests are publicly exported.
- **JSONC auto-flags**: `.jsonc` should auto-enable BOTH comments AND trailing commas.
- **API scope**: Minimal — just add `Jsonc` variant to `ApiFormat` enum. No granular flags on request bodies.
- **Test strategy**: Implement first, tests after. Ensure ALL existing inline tests are migrated.

**Research Findings**:
- 13 source files contain inline `#[cfg(test)]` modules
- `src/tests/` is DEAD CODE — `lib.rs` never declares `mod tests;`, so those files were never compiled. They are pre-written drafts for migration, not "already migrated" tests. Must be validated against inline tests before use.
- `json/parser_tests.rs` accesses private `Parser` fields (`config`, `depth`, `bytes_parsed`) — needs 3 pub getter methods
- Library already supports comments/trailing_commas via `JsonConfig` — gap is only auto-detection wiring
- API calls `convert()` not `convert_with_options()` — needs switching for JSONC to work
- API `/api/formats` endpoint is hard-coded — must manually add "jsonc"
- Pre-existing failure: `yaml_property_tests::yaml_roundtrip` — exclude from acceptance criteria

### Metis Review
**Identified Gaps (addressed)**:
- **src/tests/ is dead code**: Plan now treats it as draft material requiring validation, not as migrated tests
- **`--to jsonc` is nonsensical**: JSONC is input-only. Plan treats `--to jsonc` as `--to json` (no error, just outputs valid JSON)
- **detect_format_from_path loses jsonc info**: Plan adds an `is_jsonc_path()` helper alongside the detection
- **API has TWO call sites**: Both `convert()` handler AND `parse_to_json()` helper need updating
- **Pub getters expand API surface**: Acceptable tradeoff — 3 read-only getters on Parser for test accessibility

---

## Work Objectives

### Core Objective
Clean up test organization by consolidating all tests into `crates/zparse/tests/`, then add `.jsonc` auto-detection so that JSONC files get parsed with permissive JSON settings across all entry points (library, CLI, API).

### Concrete Deliverables
- `crates/zparse/src/` — 13 files with inline `#[cfg(test)]` blocks cleaned (blocks removed)
- `crates/zparse/src/json/parser.rs` — 3 pub getter methods added (`config()`, `depth()`, `bytes_parsed()`)
- `crates/zparse/tests/` — New integration test files for all migrated tests
- `crates/zparse/src/tests.rs` and `crates/zparse/src/tests/` — Deleted
- `crates/zparse/src/lib.rs` — `detect_format_from_path()` handles `.jsonc`, new `is_jsonc_path()` helper
- `crates/zparse-cli/src/main.rs` — `Jsonc` variant in `FormatArg`, auto-enables flags for `.jsonc` files
- `crates/zparse-api/src/main.rs` — `Jsonc` variant in `ApiFormat`, `convert_with_options()` usage, updated `/api/formats`

### Definition of Done
- [ ] `cargo test -p zparse` passes — all previously-inline tests run as integration tests
- [ ] `cargo test -p zparse-cli` passes
- [ ] `cargo test -p zparse-api` passes (if applicable)
- [ ] `cargo clippy --workspace` has no errors
- [ ] `.jsonc` files parsed correctly via CLI and API
- [ ] No `#[cfg(test)]` blocks remain in source files (excluding test files themselves)
- [ ] `src/tests.rs` and `src/tests/` directory no longer exist

### Must Have
- All 92 inline tests preserved and passing as integration tests
- `.jsonc` auto-enables both comments AND trailing commas
- JSONC support in library detection, CLI, and API
- Clean deletion of all dead test code

### Must NOT Have (Guardrails)
- **NO new `Format::Jsonc` variant** in the core library `Format` enum — JSONC is JSON with config flags, not a separate format
- **NO modification of test logic/assertions** during migration — only import paths and field accessor syntax
- **NO changes to `convert()` or `convert_with_options()` function signatures**
- **NO JSONC-specific serialization** — JSONC is input-only; output is always valid JSON
- **NO granular JSON flag fields on API request bodies** — minimal API changes only
- **NO fixing the pre-existing `yaml_property_tests` failure** — out of scope
- **NO excessive comments, over-abstraction, or AI slop** in added code

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (`cargo test` with standard Rust test framework)
- **Automated tests**: Tests after (implement first, add JSONC-specific tests after)
- **Framework**: `cargo test` (built-in)

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Library/CLI**: Use Bash — `cargo test`, `cargo run`, `cargo clippy`
- **API**: Use Bash — `cargo build`, grep verification

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — baseline + foundation):
├── Task 1: Capture test baseline [quick]
├── Task 2: Add pub getters to Parser [quick]

Wave 2 (After Wave 1 — parallel test migration, MAX PARALLEL):
├── Task 3: Migrate lexer tests (5 test files) [unspecified-high]
├── Task 4: Migrate JSON tests (2 test files) [unspecified-high]
├── Task 5: Migrate TOML/YAML/XML parser tests (3 test files) [unspecified-high]
├── Task 6: Migrate value + error + input tests (3 test files) [unspecified-high]

Wave 3 (After Wave 2 — cleanup + JSONC, parallel):
├── Task 7: Delete inline tests from 13 source files [unspecified-high]
├── Task 8: Delete dead src/tests/ directory [quick]
├── Task 9: Add JSONC detection to library [quick]
├── Task 10: Add JSONC support to CLI [unspecified-high]
├── Task 11: Add JSONC support to API [unspecified-high]

Wave 4 (After Wave 3 — tests + verification):
├── Task 12: Add JSONC integration tests [unspecified-high]

Wave FINAL (After ALL tasks — independent review, 4 parallel):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
├── Task F4: Scope fidelity check (deep)

Critical Path: T1 → T2 → T3-T6 → T7+T8+T9+T10+T11 → T12 → F1-F4
Parallel Speedup: ~50% faster than sequential
Max Concurrent: 5 (Wave 3)
```

### Dependency Matrix

| Task | Blocked By | Blocks |
|------|-----------|--------|
| T1   | —         | T2-T12 |
| T2   | T1        | T3-T6  |
| T3   | T2        | T7     |
| T4   | T2        | T7     |
| T5   | T2        | T7     |
| T6   | T2        | T7     |
| T7   | T3-T6     | T12    |
| T8   | T3-T6     | T12    |
| T9   | —*        | T10, T11, T12 |
| T10  | T9        | T12    |
| T11  | T9        | T12    |
| T12  | T7-T11    | F1-F4  |
| F1-F4| T12       | —      |

*T9 can technically start in Wave 1 since it only touches lib.rs detection, but logically groups with Wave 3.

### Agent Dispatch Summary

- **Wave 1**: **2** — T1 → `quick`, T2 → `quick`
- **Wave 2**: **4** — T3-T6 → `unspecified-high`
- **Wave 3**: **5** — T7 → `unspecified-high`, T8 → `quick`, T9 → `quick`, T10 → `unspecified-high`, T11 → `unspecified-high`
- **Wave 4**: **1** — T12 → `unspecified-high`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Capture Test Baseline

  **What to do**:
  - Run `cargo test --lib -p zparse` and record exact test count and results (expected: 92 passed, 0 failed)
  - Run `cargo test -p zparse` (all tests including integration) and record results
  - Save both outputs as evidence — this is the invariant that must be preserved after migration
  - Verify `src/tests.rs` is dead code: confirm `lib.rs` does NOT contain `mod tests;` declaration

  **Must NOT do**:
  - Modify any files
  - Fix any pre-existing test failures (yaml_property_tests is known-failing)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3-6 (migration tasks need baseline as reference)
  - **Blocked By**: None

  **References**:
  - `crates/zparse/src/lib.rs` — Check that NO `mod tests;` declaration exists (confirms src/tests/ is dead code)
  - Pre-existing failure: `yaml_property_tests::yaml_roundtrip` — known issue, ignore in baseline

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Capture inline test baseline
    Tool: Bash
    Preconditions: Clean checkout, no uncommitted changes
    Steps:
      1. Run: cargo test --lib -p zparse 2>&1 | tee /tmp/zparse-baseline-lib.txt
      2. Assert output contains "test result: ok. 92 passed; 0 failed" (or close — exact count may vary)
      3. Run: cargo test -p zparse 2>&1 | tee /tmp/zparse-baseline-all.txt
      4. Record the total test count from both runs
      5. Run: grep "mod tests;" crates/zparse/src/lib.rs
      6. Assert: no output (confirms src/tests/ is orphaned dead code)
    Expected Result: Baseline captured with exact test counts
    Failure Indicators: Test failures beyond yaml_property_tests, lib.rs contains mod tests declaration
    Evidence: .sisyphus/evidence/task-1-baseline-lib.txt, .sisyphus/evidence/task-1-baseline-all.txt
  ```

  **Commit**: NO (no file changes)

---

- [x] 2. Add Pub Getters to Parser

  **What to do**:
  - Add 3 public getter methods to `Parser<'a>` in `crates/zparse/src/json/parser.rs`:
    ```rust
    /// Returns the parser configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns the current parsing depth.
    pub fn depth(&self) -> u16 {
        self.depth
    }

    /// Returns the number of bytes parsed so far.
    pub fn bytes_parsed(&self) -> usize {
        self.bytes_parsed
    }
    ```
  - These getters are needed because `json/parser_tests.rs` accesses `parser.config`, `parser.depth`, and `parser.bytes_parsed` which are private fields
  - When tests move to integration tests (tests/ directory), they can't access private fields — getters provide the public interface

  **Must NOT do**:
  - Add setters or mutable access
  - Modify any existing Parser methods
  - Change field visibility directly (keep fields private, expose via getters)
  - Add getters beyond these 3

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3-6 (migration tasks depend on getters being available)
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `crates/zparse/src/json/parser.rs` — The `Parser<'a>` struct definition. Fields `config: Config`, `depth: u16`, `bytes_parsed: usize` are all private. Add getters in the `impl<'a> Parser<'a>` block.

  **API/Type References**:
  - `crates/zparse/src/json/parser.rs` — `Config` struct (has `pub` fields: `max_depth`, `max_size`, `allow_comments`, `allow_trailing_commas`). Returning `&Config` is sufficient since its fields are already pub.

  **WHY Each Reference Matters**:
  - The Parser struct and impl block are where the getters must be added. Config's pub fields mean returning `&Config` gives tests full read access to config values.

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Getters compile and existing tests still pass
    Tool: Bash
    Preconditions: Parser struct in crates/zparse/src/json/parser.rs
    Steps:
      1. Run: cargo check -p zparse
      2. Assert: exit code 0, no compilation errors
      3. Run: cargo test --lib -p zparse
      4. Assert: same test count as baseline (getters don't affect existing tests)
    Expected Result: Clean compilation, all existing tests pass unchanged
    Failure Indicators: Compilation error, test count differs from baseline
    Evidence: .sisyphus/evidence/task-2-check.txt

  Scenario: Getters are accessible from outside the crate
    Tool: Bash
    Preconditions: Getters added to Parser
    Steps:
      1. Run: cargo doc -p zparse --no-deps 2>&1
      2. Assert: no warnings about missing docs (or at most pre-existing ones)
      3. Verify getters appear in public API: grep for "pub fn config\|pub fn depth\|pub fn bytes_parsed" in crates/zparse/src/json/parser.rs
      4. Assert: all 3 found
    Expected Result: 3 public getter methods present and documented
    Evidence: .sisyphus/evidence/task-2-getters-verify.txt
  ```

  **Commit**: YES (groups with T1)
  - Message: `refactor(zparse): add pub getters to Parser for test migration`
  - Files: `crates/zparse/src/json/parser.rs`
  - Pre-commit: `cargo test --lib -p zparse`

---

- [x] 3. Migrate Lexer Tests (5 files)

  **What to do**:
  - Migrate these 5 test files from `crates/zparse/src/tests/lexer/` to `crates/zparse/tests/`:
    - `cursor_tests.rs` → `crates/zparse/tests/lexer_cursor_tests.rs`
    - `token_tests.rs` → `crates/zparse/tests/lexer_token_tests.rs`
    - `json_lexer_tests.rs` → `crates/zparse/tests/lexer_json_tests.rs`
    - `toml_lexer_tests.rs` → `crates/zparse/tests/lexer_toml_tests.rs`
    - `yaml_lexer_tests.rs` → `crates/zparse/tests/lexer_yaml_tests.rs`
  - **CRITICAL**: Before copying, diff each `src/tests/lexer/*.rs` file against its corresponding inline `#[cfg(test)]` block in the source file. If they differ, use the inline version as source of truth (since src/tests/ was never compiled and may have drifted).
  - Replace ALL `use crate::` imports with `use zparse::` (approximately 5-8 import lines per file)
  - Tests use `Error::with_message` and `Span::empty()` in helper functions — both are public, just update import path
  - Follow existing integration test pattern: flat files with `use zparse::...;` + standalone `#[test]` functions (see `crates/zparse/tests/convert_tests.rs` for pattern)

  **Must NOT do**:
  - Modify test logic or assertions — only change import paths
  - Create a module tree in tests/ — keep flat file structure
  - Add new test cases (that's Task 12)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
    - Reason: Bulk file migration with careful import rewriting across 5 files

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5, 6)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 7 (inline test deletion depends on migration being complete)
  - **Blocked By**: Task 2 (getters must exist before tests can reference them via public API)

  **References**:

  **Pattern References**:
  - `crates/zparse/tests/convert_tests.rs` — Follow this file's pattern for import style and test organization (flat `use zparse::...;` + `#[test]` functions)
  - `crates/zparse/src/tests/lexer/json_lexer_tests.rs` — Source test file with `crate::` imports to rewrite
  - `crates/zparse/src/tests/lexer/cursor_tests.rs` — Source test file
  - `crates/zparse/src/tests/lexer/token_tests.rs` — Source test file
  - `crates/zparse/src/tests/lexer/toml_lexer_tests.rs` — Source test file
  - `crates/zparse/src/tests/lexer/yaml_lexer_tests.rs` — Source test file

  **Inline Test References** (source of truth if drift detected):
  - `crates/zparse/src/lexer/cursor.rs` — inline `#[cfg(test)] mod tests` block
  - `crates/zparse/src/lexer/token.rs` — inline `#[cfg(test)] mod tests` block
  - `crates/zparse/src/lexer/json.rs` — inline `#[cfg(test)] mod tests` block
  - `crates/zparse/src/lexer/toml.rs` — inline `#[cfg(test)] mod tests` block
  - `crates/zparse/src/lexer/yaml.rs` — inline `#[cfg(test)] mod tests` block

  **WHY Each Reference Matters**:
  - The convert_tests.rs shows the EXACT pattern for integration test files (import style, no module hierarchy)
  - The src/tests/ files are pre-written but never compiled — must validate against inline blocks
  - The inline #[cfg(test)] blocks are the source of truth for test logic

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Migrated lexer tests compile and pass
    Tool: Bash
    Preconditions: 5 new test files in crates/zparse/tests/
    Steps:
      1. Run: cargo test -p zparse --test lexer_cursor_tests 2>&1
      2. Assert: exit code 0, all tests pass
      3. Run: cargo test -p zparse --test lexer_token_tests 2>&1
      4. Assert: exit code 0, all tests pass
      5. Run: cargo test -p zparse --test lexer_json_tests 2>&1
      6. Assert: exit code 0, all tests pass
      7. Run: cargo test -p zparse --test lexer_toml_tests 2>&1
      8. Assert: exit code 0, all tests pass
      9. Run: cargo test -p zparse --test lexer_yaml_tests 2>&1
      10. Assert: exit code 0, all tests pass
    Expected Result: All 5 migrated test files compile and all tests pass
    Failure Indicators: Compilation errors (wrong import path), test failures (logic drift)
    Evidence: .sisyphus/evidence/task-3-lexer-tests.txt

  Scenario: No crate:: imports remain in migrated files
    Tool: Bash
    Preconditions: Migration complete
    Steps:
      1. Run: grep -r "use crate::" crates/zparse/tests/lexer_*.rs
      2. Assert: no output (all crate:: replaced with zparse::)
    Expected Result: Zero crate:: imports in integration test files
    Evidence: .sisyphus/evidence/task-3-import-check.txt
  ```

  **Commit**: YES (groups with T4-T6)
  - Message: `test(zparse): migrate inline tests to integration test directory`
  - Files: `crates/zparse/tests/lexer_*.rs`
  - Pre-commit: `cargo test -p zparse --test lexer_cursor_tests --test lexer_token_tests --test lexer_json_tests --test lexer_toml_tests --test lexer_yaml_tests`

---

- [x] 4. Migrate JSON Tests (2 files)

  **What to do**:
  - Migrate these 2 test files from `crates/zparse/src/tests/json/` to `crates/zparse/tests/`:
    - `event_tests.rs` → `crates/zparse/tests/json_event_tests.rs`
    - `parser_tests.rs` → `crates/zparse/tests/json_parser_tests.rs`
  - **CRITICAL**: Diff each src/tests/ file against inline `#[cfg(test)]` block first. Use inline version as source of truth if they differ.
  - Replace ALL `use crate::` imports with `use zparse::`
  - **SPECIAL for parser_tests.rs**: Replace private field access with getter calls:
    - `parser.config.max_depth` → `parser.config().max_depth`
    - `parser.config.max_size` → `parser.config().max_size`
    - `parser.depth` → `parser.depth()` (in assertion context)
    - `parser.bytes_parsed` → `parser.bytes_parsed()` (in assertion context)
    - There are approximately 5 occurrences across 2 tests (`test_parser_new`, `test_parser_with_config`)

  **Must NOT do**:
  - Modify test logic or assertions beyond import/accessor changes
  - Add new test cases

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 5, 6)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 7
  - **Blocked By**: Task 2 (parser getters must exist)

  **References**:

  **Pattern References**:
  - `crates/zparse/tests/convert_tests.rs` — Integration test pattern to follow
  - `crates/zparse/src/tests/json/parser_tests.rs` — Source file with private field access that needs getter conversion
  - `crates/zparse/src/tests/json/event_tests.rs` — Source file

  **Inline Test References**:
  - `crates/zparse/src/json/parser.rs:474` — inline `#[cfg(test)]` block (source of truth)
  - `crates/zparse/src/json/event.rs:22` — inline `#[cfg(test)]` block (source of truth)

  **WHY Each Reference Matters**:
  - parser_tests.rs is the ONLY file with the private field access blocker — the getter conversion must happen here
  - The inline blocks are source of truth since src/tests/ was never compiled

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Migrated JSON tests compile with getter calls
    Tool: Bash
    Preconditions: Parser getters added (Task 2), test files in crates/zparse/tests/
    Steps:
      1. Run: cargo test -p zparse --test json_event_tests 2>&1
      2. Assert: exit code 0, all tests pass
      3. Run: cargo test -p zparse --test json_parser_tests 2>&1
      4. Assert: exit code 0, all tests pass
      5. Verify getter usage: grep "\.config()\.\|\.depth()\|\.bytes_parsed()" crates/zparse/tests/json_parser_tests.rs
      6. Assert: getter calls found (no direct field access)
    Expected Result: JSON tests pass using public getter API
    Failure Indicators: Compilation error on private field access, test count mismatch
    Evidence: .sisyphus/evidence/task-4-json-tests.txt

  Scenario: No private field access in migrated tests
    Tool: Bash
    Steps:
      1. Verify no direct parser.config (without parentheses) usage: grep "parser\.config[^(]" crates/zparse/tests/json_parser_tests.rs || true
      2. Assert: no matches (all converted to parser.config())
    Expected Result: All private field access replaced with getter calls
    Evidence: .sisyphus/evidence/task-4-no-private-access.txt
  ```

  **Commit**: YES (groups with T3, T5, T6)
  - Files: `crates/zparse/tests/json_event_tests.rs`, `crates/zparse/tests/json_parser_tests.rs`

---

- [x] 5. Migrate TOML, YAML, and XML Parser Tests (3 files)

  **What to do**:
  - Migrate these 3 test files from `crates/zparse/src/tests/` to `crates/zparse/tests/`:
    - `toml/parser_tests.rs` → `crates/zparse/tests/toml_parser_tests.rs`
    - `yaml/parser_tests.rs` → `crates/zparse/tests/yaml_parser_tests.rs`
    - `xml/parser_tests.rs` → `crates/zparse/tests/xml_parser_tests.rs`
  - **CRITICAL**: Diff each src/tests/ file against inline `#[cfg(test)]` block first. Use inline version as source of truth if they differ.
  - Replace ALL `use crate::` imports with `use zparse::`
  - Follow flat file pattern (no module hierarchy)

  **Must NOT do**:
  - Modify test logic or assertions — only change import paths
  - Create module tree in tests/

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 4, 6)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 7
  - **Blocked By**: Task 2

  **References**:

  **Pattern References**:
  - `crates/zparse/tests/convert_tests.rs` — Integration test pattern
  - `crates/zparse/src/tests/toml/parser_tests.rs` — Source file
  - `crates/zparse/src/tests/yaml/parser_tests.rs` — Source file
  - `crates/zparse/src/tests/xml/parser_tests.rs` — Source file

  **Inline Test References**:
  - `crates/zparse/src/toml/parser.rs:927` — inline `#[cfg(test)]` block (source of truth)
  - `crates/zparse/src/yaml/parser.rs:521` — inline `#[cfg(test)]` block (source of truth)
  - `crates/zparse/src/xml/parser.rs:342` — inline `#[cfg(test)]` block (source of truth)

  **WHY Each Reference Matters**:
  - Inline blocks are source of truth — src/tests/ was dead code and may have drifted

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Migrated TOML/YAML/XML parser tests compile and pass
    Tool: Bash
    Preconditions: Test files in crates/zparse/tests/
    Steps:
      1. Run: cargo test -p zparse --test toml_parser_tests 2>&1
      2. Assert: exit code 0, all tests pass
      3. Run: cargo test -p zparse --test yaml_parser_tests 2>&1
      4. Assert: exit code 0, all tests pass
      5. Run: cargo test -p zparse --test xml_parser_tests 2>&1
      6. Assert: exit code 0, all tests pass
    Expected Result: All 3 parser test files compile and pass
    Failure Indicators: Compilation errors, test logic failures
    Evidence: .sisyphus/evidence/task-5-parser-tests.txt

  Scenario: No crate:: imports remain
    Tool: Bash
    Steps:
      1. grep -r "use crate::" crates/zparse/tests/toml_parser_tests.rs crates/zparse/tests/yaml_parser_tests.rs crates/zparse/tests/xml_parser_tests.rs
      2. Assert: no output
    Expected Result: All crate:: imports replaced with zparse::
    Evidence: .sisyphus/evidence/task-5-import-check.txt
  ```

  **Commit**: YES (groups with T3, T4, T6)
  - Files: `crates/zparse/tests/toml_parser_tests.rs`, `crates/zparse/tests/yaml_parser_tests.rs`, `crates/zparse/tests/xml_parser_tests.rs`

---

- [x] 6. Migrate Value, Error, and Input Tests (3 files)

  **What to do**:
  - Migrate these 3 test files from `crates/zparse/src/tests/` to `crates/zparse/tests/`:
    - `value_tests.rs` → `crates/zparse/tests/value_tests.rs`
    - `error_tests.rs` → `crates/zparse/tests/error_tests.rs`
    - `input_tests.rs` → `crates/zparse/tests/input_tests.rs`
  - **CRITICAL**: Diff each src/tests/ file against inline `#[cfg(test)]` block first. Use inline version as source of truth if they differ.
  - Replace ALL `use crate::` imports with `use zparse::`
  - **Note**: `value_tests.rs` tests Value enum methods (new, from, get, push, insert, display). `error_tests.rs` tests Pos, Error, ErrorKind. `input_tests.rs` tests Input struct.

  **Must NOT do**:
  - Modify test logic or assertions — only change import paths
  - Rename existing test files that already exist in tests/ (check for conflicts with `value_tests.rs` naming)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 4, 5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 7
  - **Blocked By**: Task 2

  **References**:

  **Pattern References**:
  - `crates/zparse/tests/convert_tests.rs` — Integration test pattern
  - `crates/zparse/src/tests/value_tests.rs` — Source file
  - `crates/zparse/src/tests/error_tests.rs` — Source file
  - `crates/zparse/src/tests/input_tests.rs` — Source file

  **Inline Test References**:
  - `crates/zparse/src/value.rs:471` — inline `#[cfg(test)]` block (source of truth)
  - `crates/zparse/src/error.rs:146` — inline `#[cfg(test)]` block (source of truth)
  - `crates/zparse/src/input.rs:66` — inline `#[cfg(test)]` block (source of truth)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Migrated value/error/input tests compile and pass
    Tool: Bash
    Preconditions: Test files in crates/zparse/tests/
    Steps:
      1. Run: cargo test -p zparse --test value_tests 2>&1
      2. Assert: exit code 0, all tests pass
      3. Run: cargo test -p zparse --test error_tests 2>&1
      4. Assert: exit code 0, all tests pass
      5. Run: cargo test -p zparse --test input_tests 2>&1
      6. Assert: exit code 0, all tests pass
    Expected Result: All 3 test files compile and pass
    Failure Indicators: Compilation errors, assertion failures
    Evidence: .sisyphus/evidence/task-6-core-tests.txt
  ```

  **Commit**: YES (groups with T3, T4, T5)
  - Files: `crates/zparse/tests/value_tests.rs`, `crates/zparse/tests/error_tests.rs`, `crates/zparse/tests/input_tests.rs`

---

- [x] 7. Delete Inline Tests from 13 Source Files

  **What to do**:
  - Remove `#[cfg(test)] mod tests { ... }` blocks from ALL 13 source files:
    1. `crates/zparse/src/input.rs` (line ~66)
    2. `crates/zparse/src/value.rs` (line ~471)
    3. `crates/zparse/src/error.rs` (line ~146)
    4. `crates/zparse/src/json/parser.rs` (line ~474)
    5. `crates/zparse/src/json/event.rs` (line ~22)
    6. `crates/zparse/src/toml/parser.rs` (line ~927)
    7. `crates/zparse/src/yaml/parser.rs` (line ~521)
    8. `crates/zparse/src/xml/parser.rs` (line ~342)
    9. `crates/zparse/src/lexer/cursor.rs` (line ~134)
    10. `crates/zparse/src/lexer/token.rs` (line ~83)
    11. `crates/zparse/src/lexer/json.rs` (line ~417)
    12. `crates/zparse/src/lexer/toml.rs` (line ~527)
    13. `crates/zparse/src/lexer/yaml.rs` (line ~404)
  - Also remove any `#[cfg(test)] use ...;` imports at the top of these files if they exist (test-only imports)
  - Verify each file still compiles after removal

  **Must NOT do**:
  - Remove any production code — only `#[cfg(test)]` annotated items
  - Remove the 3 pub getters added in Task 2 (those are production code now)
  - Touch files that DON'T have inline tests

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
    - Reason: Surgical deletion across 13 files requires careful attention

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8, 9, 10, 11)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 12
  - **Blocked By**: Tasks 3, 4, 5, 6 (all migration must complete before deleting inline tests)

  **References**:

  **Pattern References**:
  - Each of the 13 source files listed above — find the `#[cfg(test)]` block and remove it entirely

  **WHY Each Reference Matters**:
  - The line numbers are approximate — actual positions may shift. Search for `#[cfg(test)]` in each file to find the exact location.

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: No inline tests remain in source files
    Tool: Bash
    Preconditions: All tests migrated to tests/ (Tasks 3-6 complete)
    Steps:
      1. Run: grep -rn "#\[cfg(test)\]" crates/zparse/src/ --include="*.rs"
      2. Assert: no output (zero inline test blocks remain)
      3. Run: cargo check -p zparse
      4. Assert: exit code 0, clean compilation
      5. Run: cargo test --lib -p zparse 2>&1 | grep "test result:"
      6. Assert: "test result: ok. 0 passed" (no inline tests to run)
    Expected Result: All 13 files cleaned, zero inline tests remain, crate compiles cleanly
    Failure Indicators: Any #[cfg(test)] found in src/, compilation error, inline tests still running
    Evidence: .sisyphus/evidence/task-7-no-inline-tests.txt

  Scenario: Integration tests still pass (regression check)
    Tool: Bash
    Steps:
      1. Run: cargo test -p zparse 2>&1 | grep "test result:"
      2. Assert: all integration test binaries pass (92+ tests from migration + existing integration tests)
    Expected Result: Test count preserved — tests just moved from inline to integration
    Evidence: .sisyphus/evidence/task-7-regression-check.txt
  ```

  **Commit**: YES (groups with T8)
  - Message: `refactor(zparse): remove inline test blocks and dead src/tests directory`
  - Files: 13 source files listed above
  - Pre-commit: `cargo test -p zparse`

---

- [x] 8. Delete Dead src/tests/ Directory

  **What to do**:
  - Delete `crates/zparse/src/tests.rs` (the module root file)
  - Delete `crates/zparse/src/tests/` directory and all its contents
  - These were never compiled (dead code — lib.rs never declared `mod tests;`)
  - Now that all tests have been properly migrated to `tests/` via Tasks 3-6, this dead code is unnecessary

  **Must NOT do**:
  - Delete any files outside of `src/tests.rs` and `src/tests/`
  - Touch integration test files in `crates/zparse/tests/`

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 9, 10, 11)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 12
  - **Blocked By**: Tasks 3, 4, 5, 6 (migration must be complete before deleting source material)

  **References**:
  - `crates/zparse/src/tests.rs` — Module root declaring submodules (8 lines, dead code)
  - `crates/zparse/src/tests/` — Directory with 13+ test files (all dead code, never compiled)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Dead test code deleted
    Tool: Bash
    Steps:
      1. Run: ls crates/zparse/src/tests.rs 2>&1
      2. Assert: "No such file or directory"
      3. Run: ls crates/zparse/src/tests/ 2>&1
      4. Assert: "No such file or directory"
      5. Run: cargo check -p zparse
      6. Assert: exit code 0 (nothing referenced this code)
    Expected Result: Both tests.rs and tests/ directory removed, compilation unaffected
    Failure Indicators: Files still exist, compilation error
    Evidence: .sisyphus/evidence/task-8-dead-code-removed.txt
  ```

  **Commit**: YES (groups with T7)
  - Files: `crates/zparse/src/tests.rs`, `crates/zparse/src/tests/` (deleted)

---

- [x] 9. Add JSONC Detection to Library

  **What to do**:
  - In `crates/zparse/src/lib.rs`, update `detect_format_from_path()`:
    - Add `"jsonc" => Some(Format::Json)` to the extension match
  - Add a new public helper function `is_jsonc_path(path: &str) -> bool` (or similar) that returns `true` when the file extension is `.jsonc`. This is needed because `detect_format_from_path()` returns `Option<Format>` which loses the json/jsonc distinction — callers need to know whether to auto-enable permissive JSON flags.
  - The helper should be simple: extract extension, compare case-insensitively to "jsonc"
  - Export the new function from `lib.rs` so CLI and API can use it

  **Must NOT do**:
  - Add a `Format::Jsonc` variant to the `Format` enum — JSONC is JSON with config flags, not a separate format
  - Modify `convert()` or `convert_with_options()` signatures
  - Change any existing format detection behavior

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8)
  - **Parallel Group**: Wave 3
  - **Blocks**: Tasks 10, 11 (CLI and API depend on library detection)
  - **Blocked By**: None (independent of test migration)

  **References**:

  **Pattern References**:
  - `crates/zparse/src/lib.rs` — `detect_format_from_path()` function: matches on file extension case-insensitively. Currently handles "json", "toml", "yaml", "yml", "xml". Add "jsonc" → `Some(Format::Json)`.
  - `crates/zparse/src/lib.rs` — `is_jsonc_path()` should follow the same pattern as `detect_format_from_path()`: extract extension from path, lowercase compare.

  **WHY Each Reference Matters**:
  - detect_format_from_path is the ONLY place format-from-extension logic lives. Adding jsonc here ensures library-level support.
  - The new is_jsonc_path helper is needed because Format::Json can't distinguish "came from .json" vs "came from .jsonc" — callers need this signal.

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: Library detects .jsonc as JSON format
    Tool: Bash
    Preconditions: detect_format_from_path updated
    Steps:
      1. Run: cargo check -p zparse
      2. Assert: exit code 0
      3. Verify the match arm exists: grep "jsonc" crates/zparse/src/lib.rs
      4. Assert: found in detect_format_from_path and is_jsonc_path
    Expected Result: .jsonc maps to Format::Json and is_jsonc_path returns true
    Evidence: .sisyphus/evidence/task-9-jsonc-detection.txt

  Scenario: Existing format detection unchanged
    Tool: Bash
    Steps:
      1. Run: cargo test -p zparse --test format_detection_tests
      2. Assert: all existing tests pass (no regression)
    Expected Result: .json, .toml, .yaml, .yml, .xml all still work correctly
    Evidence: .sisyphus/evidence/task-9-no-regression.txt
  ```

  **Commit**: YES (groups with T10, T11)
  - Message: `feat(zparse): add JSONC file extension support`
  - Files: `crates/zparse/src/lib.rs`

---

- [x] 10. Add JSONC Support to CLI

  **What to do**:
  - In `crates/zparse-cli/src/main.rs`:
    1. Add `Jsonc` variant to the `FormatArg` enum: `#[value(name = "jsonc")] Jsonc`
    2. Update `From<FormatArg> for zparse::Format` impl: add `FormatArg::Jsonc => Format::Json`
    3. Update `resolve_format()` function or its call sites:
       - When format is resolved from file extension AND the path is `.jsonc` (use `zparse::is_jsonc_path()`), auto-enable comments and trailing commas by overriding the json_config flags
       - When format is `FormatArg::Jsonc` explicitly (via `--from jsonc`), auto-enable both flags regardless of the `--json-comments` / `--json-trailing-commas` CLI flags
    4. Handle `--to jsonc`: Treat it as `--to json` (JSONC is input-only, output is always valid JSON). No error, just produce standard JSON output.
  - The flow should be: if source is jsonc (by extension OR by `--from jsonc`), build JsonConfig with `with_comments(true).with_trailing_commas(true)`, then apply any additional explicit flags on top

  **Must NOT do**:
  - Add JSONC-specific serialization (output is always valid JSON)
  - Error on `--to jsonc` — just silently treat as `--to json`
  - Modify the library crate from CLI code

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
    - Reason: Multiple integration points in CLI (enum, From impl, resolve_format, json_config_from_flags)

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 11)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 12
  - **Blocked By**: Task 9 (needs `is_jsonc_path()` from library)

  **References**:

  **Pattern References**:
  - `crates/zparse-cli/src/main.rs` — `FormatArg` enum: `#[derive(Clone, Copy, ValueEnum)]` with variants `Json`, `Toml`, `Yaml` (alias "yml"), `Xml`. Add `Jsonc` here.
  - `crates/zparse-cli/src/main.rs` — `From<FormatArg> for zparse::Format` impl: maps each variant. Add `Jsonc => Format::Json`.
  - `crates/zparse-cli/src/main.rs` — `resolve_format()`: tries `--from`, then `detect_format_from_path()`, else error. Update to also check `is_jsonc_path()`.
  - `crates/zparse-cli/src/main.rs` — `json_config_from_flags()`: builds `JsonConfig::default().with_comments(c).with_trailing_commas(t)`. When source is jsonc, force both to `true`.
  - `crates/zparse-cli/src/main.rs` — `run_parse()` and `run_convert()`: both call `json_config_from_flags` — ensure jsonc detection flows through.

  **WHY Each Reference Matters**:
  - FormatArg is the clap CLI type — adding Jsonc makes `--from jsonc` valid
  - resolve_format + json_config_from_flags is where the auto-enable magic happens
  - Both run_parse and run_convert need to receive the jsonc-aware config

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: CLI parses .jsonc file with comments and trailing commas
    Tool: Bash
    Preconditions: Task 9 complete (library detection), CLI updated
    Steps:
      1. Create test file: echo '{"a": 1, /* comment */ "b": 2,}' > /tmp/test.jsonc
      2. Run: cargo run -p zparse-cli -- --parse /tmp/test.jsonc --print-output 2>&1
      3. Assert: exit code 0, output contains parsed JSON values
      4. Cleanup: rm /tmp/test.jsonc
    Expected Result: .jsonc file parsed successfully with auto-enabled permissive flags
    Failure Indicators: Parse error about unexpected token (comments/trailing comma rejected)
    Evidence: .sisyphus/evidence/task-10-cli-jsonc-parse.txt

  Scenario: CLI accepts --from jsonc explicitly
    Tool: Bash
    Steps:
      1. Run: echo '{"a": 1, // line comment\n"b": 2,}' | cargo run -p zparse-cli -- --parse --from jsonc --print-output 2>&1
      2. Assert: exit code 0, parsed successfully
    Expected Result: --from jsonc enables permissive JSON parsing
    Evidence: .sisyphus/evidence/task-10-cli-from-jsonc.txt

  Scenario: CLI converts .jsonc to TOML
    Tool: Bash
    Steps:
      1. echo '{"name": "test", /* comment */ "version": 1,}' > /tmp/test.jsonc
      2. Run: cargo run -p zparse-cli -- --convert /tmp/test.jsonc --to toml --print-output 2>&1
      3. Assert: exit code 0, output is valid TOML (contains 'name = "test"' and 'version = 1')
      4. Cleanup: rm /tmp/test.jsonc
    Expected Result: JSONC→TOML conversion works end-to-end
    Evidence: .sisyphus/evidence/task-10-cli-jsonc-convert.txt

  Scenario: --to jsonc gracefully produces valid JSON
    Tool: Bash
    Steps:
      1. Run: echo 'name = "test"' | cargo run -p zparse-cli -- --convert --from toml --to jsonc --print-output 2>&1
      2. Assert: exit code 0, output is valid JSON (no comments or trailing commas in output)
    Expected Result: --to jsonc treated as --to json (no error)
    Evidence: .sisyphus/evidence/task-10-cli-to-jsonc.txt
  ```

  **Commit**: YES (groups with T9, T11)
  - Files: `crates/zparse-cli/src/main.rs`
  - Pre-commit: `cargo build -p zparse-cli`

---

- [x] 11. Add JSONC Support to API

  **What to do**:
  - In `crates/zparse-api/src/main.rs`:
    1. Add `Jsonc` variant to `ApiFormat` enum (serde will auto-deserialize "jsonc" with `rename_all = "lowercase"`)
    2. Update `From<ApiFormat> for zparse::Format` impl: add `Jsonc => Format::Json`
    3. Update `formats()` handler: add `"jsonc"` to the returned vector
    4. Update `convert()` handler: when `from == ApiFormat::Jsonc`, call `zparse::convert_with_options()` with `ConvertOptions { json: JsonConfig::default().with_comments(true).with_trailing_commas(true) }` instead of `zparse::convert()`
    5. Update `parse_to_json()` helper: same treatment — when format is `ApiFormat::Jsonc`, use `convert_with_options()` with permissive config
    6. Handle `to == ApiFormat::Jsonc` in convert: treat as `ApiFormat::Json` (output is always valid JSON)
  - **IMPORTANT**: Both the `convert` handler AND `parse_to_json` helper currently call `zparse::convert()` — BOTH must be updated

  **Must NOT do**:
  - Add granular `allow_comments`/`allow_trailing_commas` fields to request bodies
  - Add JSONC-specific output format
  - Change the API's external interface beyond adding "jsonc" as a valid format value

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 10)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 12
  - **Blocked By**: Task 9 (needs library JSONC detection)

  **References**:

  **Pattern References**:
  - `crates/zparse-api/src/main.rs` — `ApiFormat` enum: `#[derive(Deserialize)] #[serde(rename_all = "lowercase")]` with variants `Json`, `Toml`, `Yaml`, `Xml`. Add `Jsonc`.
  - `crates/zparse-api/src/main.rs` — `From<ApiFormat> for zparse::Format` impl: direct mapping. Add `Jsonc => Format::Json`.
  - `crates/zparse-api/src/main.rs` — `formats()` handler: returns `Json(vec!["json", "toml", "yaml", "xml"])`. Add "jsonc".
  - `crates/zparse-api/src/main.rs` — `convert()` handler: currently calls `zparse::convert()`. Must conditionally call `convert_with_options()` when from is Jsonc.
  - `crates/zparse-api/src/main.rs` — `parse_to_json()` helper: also calls `zparse::convert()`. Same update needed.

  **API/Type References**:
  - `crates/zparse/src/convert.rs` — `ConvertOptions { json: JsonConfig }` and `convert_with_options()` signature. Needed import for API changes.
  - `crates/zparse/src/json/parser.rs` — `Config` (aka `JsonConfig`): `with_comments(bool)`, `with_trailing_commas(bool)` builder methods.

  **WHY Each Reference Matters**:
  - ApiFormat enum is the request deserialization type — adding Jsonc makes `"from": "jsonc"` valid
  - BOTH convert() and parse_to_json() call zparse::convert — easy to miss parse_to_json (Metis flagged this)
  - ConvertOptions and JsonConfig are needed to construct the permissive config

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: API compiles with JSONC support
    Tool: Bash
    Steps:
      1. Run: cargo check -p zparse-api
      2. Assert: exit code 0, no compilation errors
      3. Verify jsonc in formats: grep '"jsonc"' crates/zparse-api/src/main.rs
      4. Assert: found (at minimum in formats() handler)
    Expected Result: API crate compiles cleanly with Jsonc variant
    Evidence: .sisyphus/evidence/task-11-api-compile.txt

  Scenario: API uses convert_with_options for JSONC
    Tool: Bash
    Steps:
      1. grep "convert_with_options" crates/zparse-api/src/main.rs
      2. Assert: found (at least 2 occurrences — convert handler and parse_to_json)
      3. grep "ApiFormat::Jsonc" crates/zparse-api/src/main.rs
      4. Assert: found (in conditional logic for enabling permissive config)
    Expected Result: Both call sites updated to use convert_with_options when format is Jsonc
    Evidence: .sisyphus/evidence/task-11-api-convert-options.txt
  ```

  **Commit**: YES (groups with T9, T10)
  - Message: `feat(zparse): add JSONC file extension support`
  - Files: `crates/zparse-api/src/main.rs`
  - Pre-commit: `cargo check -p zparse-api`

---

- [x] 12. Add JSONC Integration Tests

  **What to do**:
  - Create `crates/zparse/tests/jsonc_tests.rs` with integration tests for JSONC support:
    1. **Format detection tests**: `detect_format_from_path("file.jsonc")` returns `Some(Format::Json)`, `is_jsonc_path("file.jsonc")` returns `true`, `is_jsonc_path("file.json")` returns `false`
    2. **Case insensitivity**: `detect_format_from_path("FILE.JSONC")` works, `is_jsonc_path("FILE.JSONC")` works
    3. **Parsing tests**: Create JSONC strings with comments (both `//` and `/* */`), trailing commas, and both together. Parse using `from_str_with_config()` with `Config::default().with_comments(true).with_trailing_commas(true)`. Assert correct `Value` output.
    4. **Conversion tests**: Convert JSONC input to TOML/YAML/XML using `convert_with_options()` with permissive config. Assert output is valid.
    5. **Edge cases**: Empty JSONC object `{}`, nested comments, comment-only lines, trailing comma in arrays
  - Also add `.jsonc` test case to existing `format_detection_tests.rs` if appropriate
  - Follow existing integration test pattern (flat file, `use zparse::...;`)

  **Must NOT do**:
  - Test CLI or API behavior (those are tested by their own crates)
  - Test with actual `.jsonc` files on disk (use string inputs)
  - Add excessive tests — focus on the core JSONC behavior

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (sequential, after all implementation)
  - **Blocks**: Final Verification
  - **Blocked By**: Tasks 7-11 (all implementation must be complete)

  **References**:

  **Pattern References**:
  - `crates/zparse/tests/convert_tests.rs` — Integration test pattern: `use zparse::{ConvertOptions, Format, convert_with_options};`
  - `crates/zparse/tests/format_detection_tests.rs` — Existing format detection tests to extend/complement
  - `crates/zparse/src/lib.rs` — Public API: `detect_format_from_path()`, `is_jsonc_path()`, `from_str_with_config()`, `convert_with_options()`, `ConvertOptions`, `Format`, `JsonConfig` (re-exported as `json::Config`)

  **WHY Each Reference Matters**:
  - convert_tests.rs shows exact import pattern for using convert_with_options in tests
  - format_detection_tests.rs may already have a pattern for testing extension detection — add jsonc test cases there

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**
  ```
  Scenario: JSONC integration tests pass
    Tool: Bash
    Steps:
      1. Run: cargo test -p zparse --test jsonc_tests 2>&1
      2. Assert: exit code 0, all tests pass
      3. Assert: at least 5 test functions (detection + parsing + conversion + edge cases)
    Expected Result: Comprehensive JSONC test suite passes
    Failure Indicators: Compilation errors, assertion failures
    Evidence: .sisyphus/evidence/task-12-jsonc-tests.txt

  Scenario: Existing format detection tests still pass
    Tool: Bash
    Steps:
      1. Run: cargo test -p zparse --test format_detection_tests 2>&1
      2. Assert: exit code 0, no regressions
    Expected Result: Original tests unaffected by additions
    Evidence: .sisyphus/evidence/task-12-no-regression.txt
  ```

  **Commit**: YES
  - Message: `test(zparse): add JSONC integration tests`
  - Files: `crates/zparse/tests/jsonc_tests.rs`
  - Pre-commit: `cargo test -p zparse --test jsonc_tests`

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Rejection → fix → re-run.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (run `cargo test`, check file absence/presence). For each "Must NOT Have": search codebase for forbidden patterns (e.g., `Format::Jsonc` in convert.rs, `#[cfg(test)]` in source files) — reject with file:line if found. Check evidence files exist in `.sisyphus/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace` + `cargo test -p zparse` + `cargo test --workspace`. Review all changed files for: unused imports, dead code, `#[allow(...)]` hacks, commented-out code. Check that migrated tests preserved their original assertions exactly. Verify no AI slop (excessive comments, over-abstraction).
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Test JSONC end-to-end: create `.jsonc` files with comments and trailing commas, parse via CLI (`--parse`, `--convert --to toml`), verify output. Test `--from jsonc` explicit flag. Test `--to jsonc` graceful handling. Verify API compilation includes jsonc in formats. Run every migrated test file individually to confirm no regressions.
  Evidence: `.sisyphus/evidence/final-qa/`
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (`git diff`). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| After Task(s) | Commit Message | Key Files |
|---------------|---------------|-----------|
| T1-T2 | `refactor(zparse): add pub getters to Parser for test migration` | `crates/zparse/src/json/parser.rs` |
| T3-T6 | `test(zparse): migrate inline tests to integration test directory` | `crates/zparse/tests/*.rs` |
| T7-T8 | `refactor(zparse): remove inline test blocks and dead src/tests directory` | 13 source files + deleted `src/tests/` |
| T9-T11 | `feat(zparse): add JSONC file extension support` | `lib.rs`, `main.rs` (CLI), `main.rs` (API) |
| T12 | `test(zparse): add JSONC integration tests` | `crates/zparse/tests/jsonc_tests.rs` |

---

## Success Criteria

### Verification Commands
```bash
# All workspace tests pass (except pre-existing yaml_property_tests failure)
cargo test --workspace 2>&1 | tail -5
# Expected: test result: ok (with yaml_property_tests excluded)

# No inline tests remain in source
grep -r "#\[cfg(test)\]" crates/zparse/src/ --include="*.rs" | grep -v "src/tests"
# Expected: no output

# Dead test code removed
ls crates/zparse/src/tests.rs crates/zparse/src/tests/ 2>&1
# Expected: "No such file or directory" for both

# JSONC detection works
echo '{"a": 1, /* comment */ "b": 2,}' > /tmp/test.jsonc
cargo run -p zparse-cli -- --parse /tmp/test.jsonc --print-output
# Expected: exits 0, parses successfully

# Clippy clean
cargo clippy --workspace
# Expected: no errors
```

### Final Checklist
- [ ] All 92 previously-inline tests passing as integration tests
- [ ] No `#[cfg(test)]` in source files
- [ ] `src/tests.rs` and `src/tests/` deleted
- [ ] `.jsonc` auto-detection working in library, CLI, API
- [ ] `--to jsonc` handled gracefully (outputs valid JSON)
- [ ] All "Must NOT Have" guardrails respected
