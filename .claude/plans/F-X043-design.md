# F-X043, Reuse bundled-fallback caller-font layouts

**Status**: approved
**Sprint**: S52
**Size**: M
**Depends on**: F-X039, F-X040

## Problem

`Document::layout_with_fonts_and_options` at
`crates/rdocx/src/document.rs:3370` uses the strict caller-only engine. A WASM
editor that supplies one CJK face therefore cannot resolve ordinary Calibri or
Times content, even though deterministic bundled faces are available. PRs 40
and 41 demonstrate the concrete consumer and an unsafe raw-engine handoff.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/08-rendering-spec.md`, "Performance" and the font-mode contract.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "The hash harness".

## Approach

Add `Document::layout_with_fonts_and_bundled_fallback` and its options-taking
twin. They return a shared `WordLayoutResult`, use one private deterministic
reusable engine, and load caller faces at highest priority over bundled faces.
They never consult the system-font snapshot. Add a checked transfer method that
takes mutable source and receiver documents plus the exact caller-font slice,
builds the receiver input, and delegates compatibility to
`Engine::take_if_compatible`. Preserve this private engine through staged
mutations and poison recovery. Do not add low-level forwarding functions or
raw `Engine` take and set accessors.

This is an additive pre-1.0 native Rust API for the existing WASM editor
consumer. The strict caller-only methods remain unchanged.

## Rejected alternatives

- A second low-level layout function would only forward to the existing
  deterministic engine path.
- Public engine take and set access makes stale context and system-font leakage
  representable.
- Falling back to system fonts would make WASM and deterministic output differ.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `bundled_fallback_completes_an_incomplete_caller_font_set` | Caller faces win for matching families, missing families use bundled faces, and strict caller-only layout still fails. |
| regression | `bundled_fallback_engine_reuses_and_transfers_exact_context` | Repeated edits hit retained work, compatible transfer succeeds, and changed font bytes or document context reject while preserving both engines. |
| regression | `bundled_fallback_warm_layout_equals_fresh_layout` | Pages, fonts, diagnostics, provenance, outlines, options, and rendered bytes match a fresh deterministic-base engine. |
| regression | `staged_mutations_preserve_valid_bundled_fallback_work` | Successful staging retains the engine, late failure publishes nothing, and poisoned locks recover. |

The test gate is **regression**. It includes both locked WASM targets, package
dry runs, strict caller-font isolation, and the unchanged hash harness.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout and font resolution: re-read `docs/hld/08-rendering-spec.md`, use
  deterministic font mode for baselines, and require byte-identical hashes.
- Public API of published crates: document the additive pre-1.0 Rust surface,
  run `cargo publish --dry-run` for `rdocx-layout` and `rdocx`, and check both
  archives remain below 10 MiB.
- WASM bindings: run both locked WASM checks and prove the path has no
  system-font dependency.

## Hash harness

Expected to be unchanged. The new mode is additive and existing render paths do
not change.

## Implementation checklist

- [ ] Add the two bundled-fallback facade methods.
- [ ] Retain one private deterministic-base engine across edits.
- [ ] Add exact-font-set checked transfer without exposing `Engine`.
- [ ] Preserve staged mutation and poison recovery behavior.
- [ ] Add isolation, transfer, warm-cold, WASM, package, and hash evidence.

## Open questions

None. The two PRs name the current WASM editor consumer, and the existing
checked transfer boundary decides the safe API shape.
