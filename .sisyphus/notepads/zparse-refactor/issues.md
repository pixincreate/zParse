# Issues & Gotchas: zParse Refactor

## Pre-Existing Issues
- yaml_property_tests::yaml_roundtrip fails (InF parsing) — KNOWN, DO NOT FIX

## Blockers & Dependencies
- Tasks 3-6 (test migration) BLOCKED by Task 2 (parser getters must exist first)
- Task 7 (delete inline tests) BLOCKED by Tasks 3-6 (migration must complete first)
- Tasks 10-11 (CLI/API JSONC) BLOCKED by Task 9 (library is_jsonc_path() helper)

## Critical Warnings
- src/tests/ was never compiled — may have drifted from inline tests. MUST diff before using.
- API has TWO call sites: convert() handler + parse_to_json() helper. Easy to miss parse_to_json.
- Must manually update /api/formats hard-coded list when adding Jsonc variant.
- json/parser_tests.rs has ~5 private field accesses needing getter syntax conversion.

## File Conflicts to Watch
- None expected — tests/ directory already has fixtures/convert/property tests, no naming conflicts with migrated unit tests
