# F-X073, Restart ordinary-prose pagination within the aggregate cache

**Status**: completed
**Sprint**: S63
**Size**: L
**Depends on**: F-202, F-X062, F-X072

## Problem

Issue [66](https://github.com/tensorbee/rdocx/issues/66) reports two independent
restart-pagination cliffs. The record gate rejects an entire body when any
paragraph produces more than two lines, is a heading, or uses `keepNext` or
`keepLines`, even when a complete block-boundary checkpoint represents the
effect. A separate 8 MiB restart-cache ceiling rejects useful candidates even
when the existing 64 MiB aggregate cache budget has room.

## Spec reference

- `docs/hld/08-rendering-spec.md`, incremental layout, page completion, and
  cache accounting.
- `docs/hld/12-testing-strategy.md`, restart-pagination edit, insert, delete,
  undo, bounded-work, and equality gates.
- `docs/hld/14-development-backlog.md`, "F-X073, Restart ordinary-prose
  pagination within the aggregate cache".
- Issue 66 and its current discussion.

## Approach

Separate restart-record representability from checkpoint placement. Admit
ordinary multi-line paragraphs, headings, and keep-together paragraphs when
the paginator records a complete block boundary after pending notes, wraps,
page state, and resolved state are finalized. Keep rejecting content whose
effects are not fully represented, including numbering, fields, drawings,
multilingual state, raw XML, anchored empty paragraphs, and unsupported line
items.

Replace the independent restart byte ceiling with a checked admission helper
that charges the candidate against current and pending paragraph, table,
header or footer, and restart cache bytes under the existing 64 MiB aggregate
budget. Retain every cache entry cap and fail closed on arithmetic overflow.
Do not weaken exact context fingerprints or publish a partial transaction.

Add source-built regressions to the existing `rdocx-layout` engine test module.
Exercise late edit, insert, delete, undo, and cache-pressure paths. Do not add a
module, file, trait, generic, feature flag, or dependency.

## Rejected alternatives

- Raising the fixed restart ceiling merely moves the cliff and can violate the
  aggregate budget.
- Removing all record-safety checks can cache state that the checkpoint does
  not represent.
- Recording split-paragraph checkpoints requires new continuation state and is
  outside this fix.
- Using output-only equality without bounded-work assertions can hide a cache
  that never reuses anything.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `ordinary_multiline_heading_and_keep_paragraphs_publish_restart_records` | Multi-line prose, headings, `keepNext`, and `keepLines` no longer poison an otherwise representable body. |
| regression | `restart_candidate_uses_available_aggregate_cache_budget` | A candidate above 8 MiB is retained when all cache bytes stay at or below 64 MiB. |
| regression | `restart_candidate_over_aggregate_budget_fails_closed` | A candidate above the aggregate budget is rejected without overflow or output change. |
| differential | `ordinary_prose_late_edit_insert_delete_and_undo_match_fresh_layout` | Warm edit, insert, delete, and undo outputs are byte-for-byte equal to fresh deterministic layout. |
| performance | `ordinary_prose_restart_bounds_recomputed_page_work` | A 700-paragraph ordinary-prose body reuses a complete prefix and bounds page-layout invocations after late mutations. |
| regression | `unrepresented_restart_content_remains_rejected` | Numbering, fields, drawings, multilingual state, raw XML, anchored empties, and unsupported line items remain fail-closed. |

The **test gate** is regression. Run the complete `rdocx-layout` and `rdocx`
affected suites, the deterministic hash harness, and every routed rider.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout, pagination, text shaping**. Read HLD 08, HLD 12, and HLD 15. Use
  deterministic fonts. Require exact edit, insert, delete, undo, warm, and
  fresh equality. Run the release performance gate.
- **WASM-reachable native rendering**. Run the `wasm32-unknown-unknown` check
  for `rdocx-wasm` and `rpptx-wasm`.

No public API, parser, serializer, dependency, package, feature, binding,
external-oracle, module, or file-move row is triggered.

## Hash harness

Expected unchanged across all 49 entries. The change affects cache admission
and reuse only, not rendered output. Any output delta blocks integration.

## Implementation checklist

- [x] Add discriminating ordinary-prose restart and aggregate-budget regressions.
- [x] Separate record representability from complete checkpoint placement.
- [x] Admit multi-line, heading, and keep-together ordinary prose safely.
- [x] Charge restart candidates against current and pending aggregate cache bytes.
- [x] Preserve entry caps, exact contexts, checked arithmetic, and fail-closed unsafe content.
- [x] Prove bounded late edit, insert, delete, and undo work with exact fresh equality.
- [x] Run affected suites, full verification, routed riders, and the unchanged hash harness.
- [x] Update exactly the listed HLD files.

## Open questions

None. The existing paginator already records complete block-boundary state, and
the existing aggregate cache budget defines the only accepted byte ceiling.
