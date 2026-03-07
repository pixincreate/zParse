# Decisions: zParse Refactor

## Test Location
**Decision**: Migrate ALL tests to `crates/zparse/tests/` (integration tests)
**Rationale**: All items used by tests are publicly exported — verified Error::with_message, Span::empty(), all lexer/parser types

## JSONC Behavior
**Decision**: `.jsonc` auto-enables BOTH comments AND trailing commas
**Rationale**: That's the whole point of JSONC — permissive JSON parsing

## API Scope
**Decision**: Minimal — just add `Jsonc` variant to `ApiFormat`, no granular request fields
**Rationale**: User felt API changes were unnecessary complexity, wanted CLI-level change only

## --to jsonc Handling
**Decision**: Treat silently as `--to json` (no error, just produce valid JSON)
**Rationale**: JSONC is input-only; output format doesn't make sense. Graceful degradation.

## Format Detection
**Decision**: Add `is_jsonc_path()` helper alongside `detect_format_from_path()`
**Rationale**: detect_format_from_path returns Option<Format>, can't distinguish json vs jsonc. Callers need this signal to auto-enable flags.

## Parser Getters
**Decision**: Add 3 read-only pub getters (config(), depth(), bytes_parsed())
**Rationale**: Acceptable API surface expansion — tests need read access to verify parser state

## Test Strategy
**Decision**: Implement first, tests after. Migrate ALL existing inline tests.
**Rationale**: User's preference — get functionality working, then add JSONC-specific tests
