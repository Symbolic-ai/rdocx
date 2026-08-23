# F-X051, Honor caller-supplied font family aliases

**Status**: completed
**Sprint**: S54
**Size**: M
**Depends on**: F-X043

## Problem

`FontFile::family` is public and documented as meaningful
(`crates/oxml-layout/src/font.rs:19`), but both additional-font loaders discard
the label and load only bytes (`crates/oxml-layout/src/font.rs:314` and
`crates/oxml-layout/src/font.rs:340`). Resolution tries an embedded family,
static mapped alternatives, and generic fallbacks with no caller alias tier
(`crates/oxml-layout/src/font.rs:640`). Callers must therefore repeat one large
font byte buffer for every document-facing family name.

Reusable compatibility includes exact font files but not aliases
(`crates/rdocx-layout/src/engine.rs:435`). Alias changes can otherwise reuse
resolution-dependent paragraph, table, header, and footer state under the
wrong context. Issue 44 and PR 45 identify the real workload, but the proposed
patch predates the current bounded cache and checked-transfer contract.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and the native Word retained
  engine contract.
- `docs/hld/08-rendering-spec.md`, "Performance".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and "WASM".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and the bundled-fallback
  regression contract.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".
- `docs/hld/14-development-backlog.md`, "F-X051, Honor caller-supplied font
  family aliases".
- GitHub Issue 44 and PR 45, with authenticated reporter and contributor
  `@emptinessform` retained for the next release containing the behavior.

## Approach

Teach `FontManager` to record a label-derived alias whenever a caller font's
label differs from its embedded family. A label equal to the embedded family
adds no alias. Add a byte-free concrete mapping setter:

```rust
pub fn set_caller_aliases(&mut self, aliases: &[(String, String)]) -> bool;
```

Resolution order is exact embedded family, explicit caller alias,
label-derived caller alias, existing mapped alternatives, then existing generic
fallbacks. Many aliases may target one loaded face without cloning bytes.

Store exact alias identity in the reusable engine context. An equal update is a
no-op. A changed mapping clears bounded resolution and coverage state while
retaining loaded faces and shaping entries whose `FontId` stays valid. Every
reusable block and restart decision rejects state resolved under a different
alias context. Checked engine transfer compares both caller fonts and aliases
and preserves both engines after an incompatible rejection.

Existing strict and bundled-fallback methods retain their signatures and gain
label-derived behavior automatically. Add alias-aware bundled-fallback methods
for default layout, option-taking layout, and checked transfer. Python, WASM,
CLI, system-font, and unrelated rendering entry points remain unchanged.

## Rejected alternatives

- Repeating `FontFile` values for aliases duplicates large byte buffers.
- Trying aliases before exact embedded names violates the required precedence.
- Exposing raw reusable engines makes stale context representable.
- Adding aliases to `LayoutInput` churns unrelated callers and widens the wrong
  public boundary.
- Merging PR 45 unchanged omits exact context identity and complete facade
  variants.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `caller_font_labels_resolve_after_exact_embedded_families` | Exact embedded families win, differing labels alias to one face, equal labels add none, and mapped and generic fallbacks retain order. |
| unit | `caller_alias_updates_preserve_bytes_and_invalidate_resolution_state` | Many aliases share one byte buffer, equal updates are no-ops, and changed mappings cannot return stale resolutions. |
| regression | `document_facing_aliases_share_one_caller_font` | Multiple document names resolve through one caller font with correct bytes, diagnostics, and provenance. |
| regression | `changed_alias_context_cannot_reuse_stale_layout_work` | Equal aliases hit safe work, changed aliases miss affected state, and incompatible transfer preserves both engines. |
| regression | `caller_alias_warm_layout_equals_cold_layout` | Pages, fonts, diagnostics, provenance, PDF, and raster results are equal warm and cold. |

The **test gate** is regression. Multiple document-facing aliases resolve to
the intended caller font without repeated bytes, exact embedded-family
requests retain priority, and unmapped requests keep the existing fallback
order. Unchanged aliases reuse safe work, changed aliases miss affected caches,
warm and cold pages, fonts, diagnostics, and provenance are equal, both WASM
targets pass, and the deterministic hash harness remains unchanged.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Layout, pagination, line breaking, or text shaping**. Read
  `docs/hld/08-rendering-spec.md`. Use deterministic caller fonts for every
  comparison, require warm and cold equality, and keep the hash harness
  byte-identical.
- **Public API of a published crate**. The new mapping and facade methods are
  additive pre-1.0 API. Run verified package dry runs and archive size checks
  for `oxml-layout`, `rdocx-layout`, and `rdocx`.
- **WASM or PyO3 bindings**. The public binding surface stays unchanged, but
  the dependency graph is shared. Run both WASM target checks and retain the
  binding excludes in workspace tests.

## Hash harness

Expected unchanged, 49 of 49. Existing samples do not provide differing
caller labels or explicit aliases. Equal labels preserve current behavior.
Any delta blocks the sprint and the baseline is not edited.

## Implementation checklist

- [x] Add label-derived and explicit byte-free alias state.
- [x] Preserve exact-family priority and current fallback order.
- [x] Bound alias and resolution state without cloning font bytes.
- [x] Add exact alias identity to reusable compatibility and transfer.
- [x] Add alias-aware default, option, and checked-transfer facade paths.
- [x] Add named unit and regression coverage for cache and output equality.
- [x] Run scoped crates, both WASM targets, packaging, and unchanged harness.
- [x] Retain Issue 44, PR 45, and `@emptinessform` for release credit.

## Open questions

- Resolved. Expose the complete alias-aware facade trio for default layout,
  option-taking layout, and checked engine transfer.
