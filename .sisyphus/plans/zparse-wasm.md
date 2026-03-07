# zParse WASM Support Plan (Client-Side, No Server)

## TL;DR

> Build a new `crates/zparse-wasm` crate that exposes `zparse` parsing/conversion to browser JavaScript via `wasm-bindgen`, so your personal dev-utils site can run fully client-side with zero server cost.
>
> **Deliverables**:
> - New WASM crate (`crates/zparse-wasm`)
> - JS-friendly parse/convert API (including JSONC input handling)
> - WASM build/test pipeline and CI integration
> - README/docs for static-site usage
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES (4 implementation waves + final verification wave)
> **Critical Path**: WASM compatibility gate → crate scaffold → API contract + normalization behavior → tests/CI/docs

---

## Context

### Original Request
- You want to use zparse on a personal website without running the axum API server.
- You asked for a new crate (e.g. `zparse-wasm`) and a full implementation plan first.
- Explicit constraint: **plan only now**, do not implement immediately.

### Interview + Research Summary
- Workspace currently has `zparse`, `zparse-cli`, `zparse-api`, and auto-includes `crates/*`.
- Best-practice browser approach: `wasm-bindgen` + `wasm-pack --target web` + `serde-wasm-bindgen`.
- Architecture recommendation (Oracle + Metis):
  - keep `zparse` core as source of truth
  - make `zparse-wasm` a **thin facade**
  - treat JSONC as **input dialect** (no core `Format::Jsonc`)

### Metis Review (Addressed)
- **Critical gap addressed**: CI currently uses workspace-wide native checks and needs explicit WASM handling.
- **Scope lock** added: no worker abstraction, no npm publishing workflow, no core format model changes in MVP.
- **Edge behavior lock** added: JSONC→JSON must produce strict JSON in WASM too.

---

## Work Objectives

### Core Objective
Introduce a production-grade browser WASM frontend for zparse that supports parse/convert workflows (including JSONC input) without backend infrastructure.

### Concrete Deliverables
- `crates/zparse-wasm/Cargo.toml`
- `crates/zparse-wasm/src/lib.rs`
- `crates/zparse-wasm/tests/*` (wasm-bindgen tests)
- CI workflow updates for wasm target jobs
- README and crate docs for browser usage

### Definition of Done
- [ ] `cargo check --target wasm32-unknown-unknown -p zparse` passes (compat gate)
- [ ] `wasm-pack build --target web crates/zparse-wasm` passes and emits wasm/js/d.ts
- [ ] `wasm-pack test --node crates/zparse-wasm` passes
- [ ] Existing workspace CI remains green with wasm crate integrated
- [ ] `cargo +nightly fmt --all` is run and clean
- [ ] `cargo clippy --all-targets --all-features` is run and all lints are addressed (no suppressions)
- [ ] Browser-facing docs show no-server usage clearly

### Must Have
- Thin wrapper only; no duplicated parsing/conversion logic
- Structured error model across JS boundary
- JSONC input support parity with CLI/API behavior
- CI support for native + wasm checks

### Must NOT Have (Guardrails)
- No `Format::Jsonc` in core `zparse::Format`
- No web server dependency for wasm usage
- No MVP scope creep into web workers/npm publishing pipeline
- No hidden behavior drift from core conversion semantics

---

## Verification Strategy

> **Zero human-only verification**. All checks are command/tool executable.

### Test Decision
- **Infrastructure exists**: YES (`cargo test`, CI matrix)
- **Automated tests**: YES (tests-after style)
- **Frameworks**: `cargo test`, `wasm-bindgen-test`, `wasm-pack test --node`

### QA Policy
- WASM API contract tested through wasm-bindgen tests
- Conversion parity tested via golden cases (JSON/TOML/YAML/XML/JSONC)
- Negative cases include invalid payloads and unknown formats

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation + compatibility gate):
├── Task 1: wasm compatibility spike for core zparse
├── Task 2: zparse-wasm crate scaffold + Cargo wiring
├── Task 3: workspace dependency/profile setup for wasm
├── Task 4: JS-facing contract spec (input/output/error types)
└── Task 5: CI impact analysis and native-job exclusion strategy

Wave 2 (Core WASM API implementation):
├── Task 6: export format enums/helpers for wasm boundary
├── Task 7: implement convert API bindings
├── Task 8: implement parse/validate API bindings
├── Task 9: implement JSON/JSONC config plumbing
├── Task 10: implement structured error mapping (kind/message/span)
└── Task 11: initialize panic hook + wasm ergonomics guards

Wave 3 (Tests + compatibility hardening):
├── Task 12: wasm unit tests for happy-path conversions
├── Task 13: wasm tests for JSONC normalization + edge cases
├── Task 14: wasm tests for invalid input/unknown formats
├── Task 15: size/perf baseline capture for wasm artifact
└── Task 16: native regression checks against existing crates

Wave 4 (CI + docs + integration examples):
├── Task 17: CI workflow updates for wasm jobs
├── Task 18: README/docs: browser usage + no-server flow
├── Task 19: sample TS/JS integration snippets for dev-utils site
└── Task 20: release/maintenance notes for future worker split

Wave FINAL (parallel independent review):
├── F1: Plan compliance audit
├── F2: Code quality review
├── F3: Real manual QA of wasm package flow
└── F4: Scope fidelity check
```

### Dependency Matrix (Abbreviated)
- 1 blocks 2, 7, 8
- 2,3 block 6-11
- 4 blocks 7,8,10,18
- 6-11 block 12-14
- 12-14 block 17,18
- 15/16 inform 18/20
- 17-20 block FINAL wave

---

## TODOs

- [ ] 1. Validate core zparse compiles for wasm target

  **What to do**:
  - Run `cargo check --target wasm32-unknown-unknown -p zparse`
  - Record compatibility blockers (if any)

  **Must NOT do**:
  - Do not patch core behavior during this spike

  **Recommended Agent Profile**:
  - Category: `quick`
  - Skills: `[]`

  **Parallelization**:
  - Can Run In Parallel: YES (with Tasks 4,5)
  - Blocks: 2,7,8
  - Blocked By: None

  **Acceptance Criteria / QA**:
  - Scenario (happy): command exits 0
  - Scenario (error): if fails, artifact with exact errors captured
  - Evidence: `.sisyphus/evidence/task-1-wasm-check.txt`

- [ ] 2. Create `crates/zparse-wasm` scaffold and Cargo metadata

  **What to do**:
  - Add crate directory, manifest, lib target (`cdylib` + `rlib`)
  - Wire workspace metadata and dependencies

  **Must NOT do**:
  - No implementation logic yet

  **Parallelization**:
  - Can Run In Parallel: YES (with 3,4,5)
  - Blocks: 6-11
  - Blocked By: 1

  **Acceptance Criteria / QA**:
  - `cargo check -p zparse-wasm` works for native metadata stage
  - Evidence: `.sisyphus/evidence/task-2-scaffold.txt`

- [ ] 3. Add workspace-level wasm dependencies/profile settings

  **What to do**:
  - Add workspace deps (wasm-bindgen, serde-wasm-bindgen, wasm-bindgen-test)
  - Add wasm-target-appropriate profile notes/config

  **Must NOT do**:
  - No forced global settings that break native crates

  **Acceptance Criteria / QA**:
  - Cargo metadata resolves all packages
  - Evidence: `.sisyphus/evidence/task-3-workspace-wasm-deps.txt`

- [ ] 4. Lock JS API contract (format/options/result/error)

  **What to do**:
  - Define export functions and parameter schema
  - Define error schema (`kind`, `message`, `span?`)

  **Must NOT do**:
  - No ad-hoc undocumented response shapes

  **Acceptance Criteria / QA**:
  - Contract documented in crate docs + README draft
  - Evidence: `.sisyphus/evidence/task-4-contract.md`

- [ ] 5. Design CI strategy for wasm + existing workspace jobs

  **What to do**:
  - Specify `--exclude zparse-wasm` where native job assumptions apply
  - Add dedicated wasm job commands

  **Must NOT do**:
  - No regression to existing CI matrix

  **Acceptance Criteria / QA**:
  - CI command plan validated against workflow files
  - Evidence: `.sisyphus/evidence/task-5-ci-plan.txt`

- [ ] 6. Implement wasm format conversion helpers

  **What to do**:
  - Map JS format strings/enums to core `zparse::Format`
  - Ensure output formats exclude jsonc where required

  **Must NOT do**:
  - Do not introduce core `Format::Jsonc`

  **Acceptance Criteria / QA**:
  - Unknown format returns structured error
  - Evidence: `.sisyphus/evidence/task-6-format-map.txt`

- [ ] 7. Implement exported `convert` wasm API

  **What to do**:
  - Bind convert call with options and return converted output
  - Preserve core conversion semantics

  **Must NOT do**:
  - No conversion logic fork in wasm crate

  **Acceptance Criteria / QA**:
  - Happy: json->toml and toml->json pass
  - Error: invalid input yields structured error
  - Evidence: `.sisyphus/evidence/task-7-convert-api.txt`

- [ ] 8. Implement exported `parse/validate` wasm API

  **What to do**:
  - Expose parse/validate functionality (string-first boundary)
  - Return deterministic output contract

  **Acceptance Criteria / QA**:
  - Happy: valid payload returns success
  - Error: malformed payload returns structured error
  - Evidence: `.sisyphus/evidence/task-8-parse-api.txt`

- [ ] 9. Implement JSONC config propagation in wasm API

  **What to do**:
  - Ensure JSONC-style options enable comments + trailing commas
  - Align behavior with CLI/API design

  **Must NOT do**:
  - No separate core format model for JSONC

  **Acceptance Criteria / QA**:
  - JSONC input converts successfully; jsonc->json returns strict JSON
  - Evidence: `.sisyphus/evidence/task-9-jsonc-wasm.txt`

- [ ] 10. Implement standardized error mapping to JS

  **What to do**:
  - Map core errors into JS-friendly object/JsValue
  - Include message + kind + optional location

  **Acceptance Criteria / QA**:
  - Error shape stable across convert/parse APIs
  - Evidence: `.sisyphus/evidence/task-10-error-model.txt`

- [ ] 11. Add panic hook and wasm runtime ergonomics

  **What to do**:
  - Setup `console_error_panic_hook::set_once()` via start/init path

  **Acceptance Criteria / QA**:
  - Panic stack visibility in debug path verified
  - Evidence: `.sisyphus/evidence/task-11-panic-hook.txt`

- [ ] 12. Add wasm-bindgen tests for standard conversions

  **What to do**:
  - Add wasm tests for JSON/TOML/YAML/XML conversion happy paths

  **Acceptance Criteria / QA**:
  - `wasm-pack test --node crates/zparse-wasm` passes these cases
  - Evidence: `.sisyphus/evidence/task-12-wasm-tests-happy.txt`

- [ ] 13. Add wasm tests for JSONC and normalization semantics

  **What to do**:
  - Test JSONC comments/trailing commas acceptance
  - Test strict JSON output after normalization path

  **Acceptance Criteria / QA**:
  - JSONC→JSON removes comments/trailing commas
  - Evidence: `.sisyphus/evidence/task-13-wasm-tests-jsonc.txt`

- [ ] 14. Add wasm negative-path tests

  **What to do**:
  - Invalid payload
  - Unknown format
  - Invalid option combination

  **Acceptance Criteria / QA**:
  - Structured errors returned, no panic crash
  - Evidence: `.sisyphus/evidence/task-14-wasm-tests-errors.txt`

- [ ] 15. Capture wasm artifact size/perf baseline

  **What to do**:
  - Record generated artifact size (raw + gzip if available)
  - Track baseline in docs/notepad

  **Acceptance Criteria / QA**:
  - Baseline values captured in evidence file
  - Evidence: `.sisyphus/evidence/task-15-size-baseline.txt`

- [ ] 16. Run native regression checks

  **What to do**:
  - Ensure existing crates unaffected by wasm additions

  **Acceptance Criteria / QA**:
  - Native workspace checks pass under updated CI command strategy
  - Evidence: `.sisyphus/evidence/task-16-native-regression.txt`

- [ ] 17. Implement CI workflow updates for wasm jobs

  **What to do**:
  - Add wasm jobs (`cargo check --target wasm32-unknown-unknown`, `wasm-pack test --node`)
  - Ensure native workspace jobs remain stable

  **Acceptance Criteria / QA**:
  - CI config lint + dry-run command validation
  - Evidence: `.sisyphus/evidence/task-17-ci-updates.txt`

- [ ] 18. Update README/docs with browser/static-site usage

  **What to do**:
  - Add no-server workflow
  - Add API examples for parse/convert and JSONC input notes

  **Acceptance Criteria / QA**:
  - Docs include runnable snippets and constraints
  - Evidence: `.sisyphus/evidence/task-18-docs.txt`

- [ ] 19. Add integration snippets for personal dev-utils site

  **What to do**:
  - TS/JS snippets for file input → convert → output
  - Include initialization and error handling example

  **Acceptance Criteria / QA**:
  - Snippets compile/lint in sample env or minimal smoke check
  - Evidence: `.sisyphus/evidence/task-19-site-snippets.txt`

- [ ] 20. Add maintenance notes / future worker split proposal

  **What to do**:
  - Document when to move heavy conversions to Web Worker
  - Record future enhancement boundaries

  **Acceptance Criteria / QA**:
  - Follow-up roadmap note exists and is scoped
  - Evidence: `.sisyphus/evidence/task-20-roadmap.txt`

---

## Final Verification Wave

- [ ] F1. Plan Compliance Audit (`oracle`)
  - Verify all must-have deliverables exist and no guardrails violated

- [ ] F2. Code Quality Review (`unspecified-high`)
  - Verify formatting, linting, test quality, and duplication controls

- [ ] F3. Real Manual QA (`unspecified-high`)
  - Execute wasm-pack build/test flow and browser-facing sample checks

- [ ] F4. Scope Fidelity Check (`deep`)
  - Ensure no scope creep into server/worker/publishing beyond MVP

---

## Commit Strategy

- Commit 1: `feat(zparse-wasm): scaffold wasm crate and workspace wiring`
- Commit 2: `feat(zparse-wasm): add parse/convert wasm bindings`
- Commit 3: `test(zparse-wasm): add wasm bindgen tests and edge coverage`
- Commit 4: `ci(zparse): add wasm jobs and preserve native matrix`
- Commit 5: `docs(readme): add zparse-wasm browser usage guide`

---

## Success Criteria

### Verification Commands
```bash
cargo +nightly fmt --all
cargo clippy --all-targets --all-features
cargo check --target wasm32-unknown-unknown -p zparse
wasm-pack build --target web crates/zparse-wasm
wasm-pack test --node crates/zparse-wasm
cargo test --workspace
```

### Final Checklist
- [ ] WASM crate exists and builds for web target
- [ ] parse/convert APIs exposed for browser JS
- [ ] JSONC input behavior aligned with existing system contracts
- [ ] Structured errors available on JS boundary
- [ ] Native and wasm CI both green
- [ ] Docs clearly explain no-server website usage
