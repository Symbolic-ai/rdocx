# F-X072, Keep paragraph caching across note references

**Status**: approved
**Sprint**: S63
**Size**: M
**Depends on**: F-X062

## Problem

Issue [65](https://github.com/tensorbee/rdocx/issues/65) reports that one
footnote disables paragraph-cache reads for every later paragraph. F-X062 made
restart pagination safe with notes and headers, but the paragraph cache still
classifies `FootnoteRef` and `EndnoteRef` as unsafe. Layout therefore sets the
transaction-wide read flag to false after the first reference even though the
cache key contains the complete typed paragraph and the reusable engine context
compares the exact footnote and endnote parts.

## Spec reference

- `docs/hld/08-rendering-spec.md`, incremental layout and deterministic note
  placement.
- `docs/hld/12-testing-strategy.md`, paragraph-cache warm and cold equality.
- `docs/hld/14-development-backlog.md`, "F-X072, Keep paragraph caching across
  note references".
- Issue 65 and its current discussion.

## Approach

Classify a footnote or endnote reference as paragraph-cache safe when every
other run and paragraph property is already safe. Keep the existing complete
typed paragraph key, revision view, width, and exact reusable engine context.
Do not introduce a running note counter or a reduced note fingerprint. A
changed note-reference ID changes the paragraph key. A changed footnote or
endnote part changes the reusable context and invalidates cache reads for the
transaction.

Keep fields, numbering, drawings, raw XML, multilingual state, and other
unsupported content conservative. Add focused regressions to the existing
`rdocx-layout` engine test module and, where facade equality is needed, the
existing `rdocx` integration binary. Do not add a module, file, trait, generic,
feature flag, or dependency.

## Rejected alternatives

- Disabling the paragraph cache after any note reference reproduces the issue.
- Removing note parts from the reusable context can reuse stale note content.
- Adding a running note counter duplicates state already represented by the
  explicit note ID and note parts.
- Clearing the entire cache after a changed note is safe but forfeits valid
  reuse on the following transaction.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `note_reference_does_not_poison_later_paragraph_cache_hits` | Editing one later paragraph in a 700-paragraph body with an early footnote produces 699 hits and one rebuild. |
| regression | `endnote_reference_does_not_poison_later_paragraph_cache_hits` | Endnote references follow the same bounded reuse contract. |
| regression | `changed_note_reference_or_note_part_invalidates_required_cache_entry` | A changed reference ID misses its paragraph key and changed note content invalidates the reusable context. |
| regression | `unsafe_prefix_still_disables_later_paragraph_hits` | Fields, numbering, drawings, raw children, and other unsupported content remain conservative. |
| differential | `note_reference_warm_layout_equals_fresh_layout` | Warm and fresh static page frames are byte-for-byte equal in deterministic font mode. |

The **test gate** is regression. Run the complete `rdocx-layout` and `rdocx`
affected suites, the deterministic hash harness, and every routed rider.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, text shaping**. Read HLD 08, HLD 12, and HLD 15. Use
  deterministic fonts. Require exact warm and fresh equality and run the
  release performance gate.
- **WASM-reachable native rendering**. Run the `wasm32-unknown-unknown` check
  for `rdocx-wasm` and `rpptx-wasm`.

No public API, parser, serializer, dependency, package, feature, binding,
external-oracle, module, or file-move row is triggered.

## Hash harness

Expected unchanged across all 49 entries. The change affects cache reuse only,
not rendered output. Any output delta blocks integration.

## Implementation checklist

- [ ] Add discriminating footnote and endnote cache regressions.
- [ ] Keep note references cache safe only when the remaining paragraph is safe.
- [ ] Prove changed reference IDs and changed note parts invalidate reuse.
- [ ] Preserve conservative handling for every existing unsafe content class.
- [ ] Prove deterministic warm and fresh output equality.
- [ ] Run affected suites, full verification, routed riders, and the unchanged hash harness.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The issue isolates the transaction-wide poison, and the existing complete
paragraph key plus exact note-part context define the safe reuse boundary.
