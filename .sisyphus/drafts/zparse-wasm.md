# Draft: zparse WASM Support

## Requirements (confirmed)
- Need a browser/client-side way to use zparse without running an axum server.
- Add a new workspace crate similar to other frontends: `zparse-wasm`.
- Goal: use zparse on a personal dev-utils website for file conversion/parsing.
- Explicitly requested: plan only now, do not implement yet.

## Technical Decisions
- Keep `zparse` core as source of truth for parsing/conversion logic.
- `zparse-wasm` should be a thin wasm-bindgen facade, not a logic fork.
- JSONC remains an input dialect (consistent with current architecture), not a new core enum variant.

## Research Findings
- Workspace uses `crates/*` membership, so `crates/zparse-wasm` auto-joins workspace.
- Best practice for static websites: `wasm-pack build --target web` + ESM import.
- Recommended JS boundary: string/object inputs, structured error outputs via serde-wasm-bindgen.
- Architecture guidance: phase rollout (MVP convert/parse, then diagnostics/testing/size optimization).

## Open Questions
- Whether to add optional Web Worker helper in initial release or follow-up.
- Whether to include direct file/bytes APIs in first wasm version or keep string-first API only.

## Scope Boundaries
- INCLUDE: new wasm crate, browser-facing API, build/docs/testing plan, CI checks for wasm target.
- EXCLUDE: replacing CLI/API, changing core format model to add `Format::Jsonc`.
