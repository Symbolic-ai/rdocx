# F-X075, Preserve restart pagination across page-spanning paragraphs

**Status**: completed
**Sprint**: S64
**Size**: M
**Depends on**: F-X073

## Problem

Issue 67 reports that the F-X073 recorded pagination path always discards its
result when any ordinary paragraph crosses a page boundary. The engine runs
the recorded pass, sees `had_split_paragraph`, runs the full paginator again,
and prevents restart-cache publication. A document with ordinary four-line
paragraphs therefore pays for two complete pagination passes after every edit
and can never reach the restart path F-X073 introduced.

The paginator already emits restart checkpoints only through
`finish_page_before`, after a complete block and only when note, wrap, and
resolved state is clean. A split continuation uses `finish_page` and cannot
publish a mid-paragraph checkpoint. The document-wide split veto is therefore
broader than the state it protects.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Performance", specifically retained block
  and restart state, transactional publication, complete-boundary restart
  checkpoints, and the 1,024-page scale boundary.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", specifically the
  editor-scale paragraph-cache and restart-pagination regression gates.
- `docs/hld/12-testing-strategy.md`, "What CI runs", for deterministic
  release-mode performance evidence.
- `docs/hld/14-development-backlog.md`, "F-X075, Preserve restart pagination
  across page-spanning paragraphs".
- GitHub Issue 67, the 175 and 700 four-line paragraph reproducer and pinned
  `0582da0` phase-timing evidence.

## Approach

Remove the `Engine::layout_transaction` branch that rejects every recorded
result with `had_split_paragraph`. Remove the now-unused private flag and its
propagation if no remaining test or implementation consumer exists. Keep
checkpoint creation unchanged in `Pager::finish_page_before`. A split may
finish one or more pages through `finish_page`, but no restart checkpoint is
eligible until the entire paragraph has completed and normal block-boundary
processing reaches clean state.

Keep every preexisting restart eligibility test, exact context fingerprint,
aggregate cache cap, checkpoint cap, page substitution rule, note-state rule,
and transactional publication boundary unchanged. The implementation remains
private to `rdocx-layout` and adds no public API, dependency, feature flag,
crate, module, file, trait, or generic parameter.

Replace the obsolete blanket split fallback assertion with source-built
regressions that prove the narrower contract. Use deterministic bundled fonts
for all layout comparisons. Record release-mode timing as completion evidence,
not as a normal unit-test wall-clock assertion.

## Rejected alternatives

- Add mid-paragraph restart checkpoints. The current checkpoint state does not
  model split continuation and no such model is needed for the reported fix.
- Keep the second full pagination pass but publish its result. This preserves
  the CPU regression and still prevents bounded warm restart work.
- Disable restart pagination for the whole document after the first split.
  Ordinary multi-page prose would remain permanently outside the feature.
- Add a new cache or public control. The existing recorded paginator and
  aggregate cache already own the required state.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `page_spanning_prose_publishes_complete_boundary_restart_records` | A deterministic 175-paragraph, four-line source fixture spans 16 pages, runs one recorded pass, retains restart state, and has no checkpoint inside a paragraph. |
| regression | `page_spanning_prose_restarts_warm_edits_exactly` | Ten sourced middle edits retain 174 paragraph-cache hits and one build, bound the repaginated page range, and equal every fresh deterministic layout, metadata, structure, and provenance field exactly. |
| regression | complete-boundary split siblings | A split paragraph followed by ordinary blocks records its first eligible checkpoint only after the paragraph completes. |
| regression | edit operation matrix | Late edit, insert, delete, and undo remain warm-to-fresh exact for page-spanning prose. |
| regression | notes and displayed page numbers | A note-bearing split and a displayed page-number footer preserve exact warm output and clean checkpoint rules. |
| regression | unsafe restart matrix | Fields, numbering, drawings, raw XML, note-bearing tables, backgrounds, multiple sections, and dirty note state remain excluded. |
| scale | interleaved release comparison | Native and bundled-fallback 175 and 700 paragraph fixtures are no worse than 1.25 times v0.11.1 and at most 0.75 times the pinned `0582da0` regression median. |
| integration | changed and consuming crates | Complete `rdocx-layout` and `rdocx` suites pass, both Word WASM graphs check, and the existing 1,000-page restart gate remains green. |

The **test gate is regression**. A deterministic 175-paragraph source-built
document whose four-line paragraphs span 16 pages completes one recorded pass,
retains a restart record, and records no checkpoint inside a paragraph. Ten
warm sourced middle edits produce 174 paragraph-cache hits and one rebuild,
paginate only a bounded affected page range, and equal every fresh layout,
metadata, structure, and provenance field exactly. Late edit,
insert, delete, undo, note-bearing split, and displayed page-number footer
cases remain exact. Existing unsafe inputs still reject restart publication.
An interleaved release-mode comparison for the 175 and 700 paragraph native
and bundled-fallback paths is no worse than 1.25 times v0.11.1 and materially
faster than the pinned `0582da0` regression boundary.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Read
  `docs/hld/08-rendering-spec.md`. Use deterministic font mode for every
  fixture and require complete warm-to-fresh equality plus bounded page work.
- **External oracle comparison**. Pin v0.11.1 and `0582da0`, use identical
  source-built fixtures and alternating release-mode rounds, report medians and
  classify variance. Authenticate the complete measured crate graph,
  surrounding test source, and exact injected harness by content before every
  measurement. Keep timing evidence outside the published crate and out of the
  normal unit-test pass threshold.
- **WASM verification rider**. No binding code changes, but `rdocx-wasm`
  consumes the changed engine. Run default and bundled-fallback wasm32 checks
  at the integrated gate.

## Hash harness

Expected unchanged across all 49 entries. The sample documents do not exercise
interactive restart state, and cold rendering output is unchanged. Any delta
blocks integration.

## Implementation checklist

- [x] Add the exact source-built 175-paragraph Issue 67 regression and observe the current double-pass failure.
- [x] Remove the blanket split fallback and any private state made dead by that removal.
- [x] Prove complete-block checkpoint placement and the absence of mid-paragraph checkpoints.
- [x] Prove ten middle edits, late edit, insert, delete, and undo against fresh deterministic layouts.
- [x] Prove note-bearing split and displayed page-number footer behavior.
- [x] Preserve every existing unsafe-state exclusion and aggregate bound.
- [x] Run the interleaved pinned release-performance comparison.
- [x] Run focused crates, both Word WASM checks, full verification, and the 49-entry hash harness.
- [x] Update exactly the three listed HLD files.

## Open questions

None. The user approved addressing Issue 67 in S64 and releasing the stable
Word family after the separate `rpptx-v0.9.0` release. The release itself keeps
its own reviewed story and final external-mutation approval.
