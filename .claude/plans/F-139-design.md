# F-139, Rewrite rdocx-wasm

**Status**: approved
**Sprint**: S35
**Size**: L
**Depends on**: F-029

## Problem

`WasmDocument` owns a partial `CT_Document` and `CT_Styles`, while the original
package bytes are stored but never used. `toDocxBytes` then creates a fresh
two-part package, so images, headers, numbering, properties, content types, and
relationships are silently discarded. The real `rdocx::Document` already owns
the complete package and flushes modeled changes back into it.

The WASM dependency graph also activates host font discovery indirectly.
`oxml-layout` already has the required default-on `system-fonts` leaf feature,
but `rdocx-layout`, `rdocx`, and their consumers do not forward a switch that
the WASM crate can disable.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions" and "Dependency rules".
- `docs/hld/10-bindings-spec.md`, "WASM".
- `docs/hld/12-testing-strategy.md`, "Gaps being closed".
- `docs/hld/13-risks-and-open-questions.md`, "The rdocx-wasm save path".
- `docs/hld/14-development-backlog.md`, "F-139, Rewrite rdocx-wasm".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags" and "CI job matrix".

## Approach

Replace the mini-model with `WasmDocument { inner: rdocx::Document }`. Preserve
the existing JavaScript names: constructor, `fromBytes`, `addParagraph`,
`addHeading`, `addBoldParagraph`, `addTable`, `getText`, `paragraphCount`,
`toDocxBytes`, `toHtml`, `toHtmlFragment`, `toMarkdown`, and
`replacePlaceholder`. Delegate every operation to the facade and map concrete
errors to string-valued `JsValue`s at the binding boundary.

Add `Document::text(&self) -> String` as the one additive published-facade API
needed to preserve ordered body and table text without reaching into
`rdocx-oxml`. Add default-on `system-fonts` forwarding through `rdocx-layout`
and `rdocx`, with their `oxml-layout` dependencies disabling defaults locally.
`rdocx-wasm` depends on `rdocx` with defaults off. Bundled fonts remain
unconditional, so no speculative `bundled-fonts` feature is added.

Add exact workspace `wasm-bindgen-test = "=0.3.76"` infrastructure and a
crate-root Node round-trip test so F-140's future `wasm-pack test --node` gate
is non-vacuous. Remove the obsolete nested `crates/rdocx-wasm/Cargo.lock`,
which describes a stale standalone 0.1.0 graph while the crate is a workspace
member.

## Rejected alternatives

- Merge original ZIP parts into the mini-model at save time. This preserves two
  competing package authorities and duplicates the facade.
- Reach into `rdocx-oxml` for `getText`. The binding belongs above the facade.
- Add `toBytes` beside `toDocxBytes`. The story requires existing JavaScript
  names to remain stable, not an expanded alias surface.
- Add a separate test file. Inline native and wasm-bindgen tests avoid another
  test binary and keep the wrapper contract in one source file.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `document_with_images_headers_and_numbering_round_trips_every_part_intact` | WASM facade round-trip retains parts, content types, relationships, opaque bytes, and reopenable semantics |
| unit | `document_text_preserves_body_and_table_order` | The additive facade getter preserves the existing `getText` order |
| integration | `wasm_round_trip_preserves_the_complete_package_in_node` | The actual exported wrapper executes under wasm-bindgen-test without losing package content |
| regression | feature-tree assertions | Native defaults include system fonts and the WASM graph excludes `fontdb/fs` and `fontconfig` |

The test gate is the backlog requirement that a document with images, headers,
and numbering round-trips through `fromBytes` and `toDocxBytes` with every part
intact. Sensitivity temporarily serializes a fresh document, proves that the
named gate fails, restores byte-identically, and reruns green.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Parser and serializer boundary. Read HLD04 and HLD06. Prove complete package,
  relationship, content-type, and opaque-part preservation through the existing
  facade serializer.
- Crate dependency graph. Read HLD03. Prove `rdocx-wasm -> rdocx`, no forbidden
  outward shared edge, and no host-font or unexpected `getrandom` path in the
  wasm32 feature tree.
- Public API of published `rdocx`. State additive semver impact, run the exact
  workspace publication dry run, and enforce the archive size ceiling.
- WASM binding. Read HLD10. Run native tests, wasm32 checks, and the Node suite,
  retaining both PyO3 workspace exclusions.
- New feature forwarding and default behavior. Native `rdocx` and
  `rdocx-layout` are existing consumers, while `rdocx-wasm` is the concrete
  defaults-off consumer. Run all no-default feature gates.
- Bundled fonts. Read HLD15 and retain the exact font and legal-file inventory.
- Version and lock changes. Inspect root metadata, lockfile, and normal and
  feature dependency trees. No tag or publication action is authorized.

## Hash harness

Expected unchanged. Native defaults remain system-font enabled, and the WASM
package-preservation fix does not alter native sample generation.

## Implementation checklist

- [ ] Add the bounded `Document::text` facade API and regression.
- [ ] Forward `system-fonts` through the native facade graph with defaults unchanged.
- [ ] Replace the WASM mini-model with `rdocx::Document` delegation.
- [ ] Add the exact wasm-bindgen-test dependency and Node round-trip gate inline.
- [ ] Delete the obsolete nested lock and update the workspace lock.
- [ ] Run package, dependency, no-default, WASM, publication, and hash riders.

## Open questions

None. The additive facade method, nested lock deletion, existing JavaScript
name, and F-139 ownership of the exact Node-test infrastructure are approved.
