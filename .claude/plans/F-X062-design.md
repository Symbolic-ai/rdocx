# F-X062, Reuse restart pagination with notes and headers

**Status**: approved
**Sprint**: S58
**Size**: M
**Depends on**: F-202

## Problem

GitHub Issue 53 reports two restart-pagination cliffs in 0.10.1. F-202 already
fixes the independent 32-page limit by raising the restart capacity to 1,024
pages under the existing byte ceiling. The remaining predicate in
`crates/rdocx-layout/src/engine.rs` still rejects any document containing a
footnote, endnote, header, or footer, so one unchanged related story disables
bounded body relayout document-wide.

The retained engine context already compares sections, headers, footers,
footnotes, and endnotes exactly. The paginator already records checkpoints only
after pending and current-page notes drain, and it records the next displayed
header page number. Those existing identities and safe boundaries can support
restart without a second cache design.

## Spec reference

- `docs/hld/08-rendering-spec.md`, restart cache, related stories, safe checkpoints, and exact fresh equality.
- `docs/hld/12-testing-strategy.md`, incremental-layout regressions and source-built performance fixtures.
- `docs/hld/14-development-backlog.md`, "F-X062, Reuse restart pagination with notes and headers".
- `docs/hld/15-build-and-toolchain.md`, deterministic layout, retained-memory ceilings, and release performance gates.
- [GitHub Issue 53](https://github.com/tensorbee/rdocx/issues/53), reporter setup, corrected attribution, and remaining reproducible cliffs.

## Approach

Remove only the global footnote, endnote, and header-footer disqualifiers from
restart-record eligibility. Continue requiring one section, no background,
no wrapping drawing, safe body blocks, exact retained context, exact font
trace, provenance compatibility, and the existing entry and byte ceilings.

Admit otherwise-safe paragraph note references into restart-record and restart
safety. Restart only from the paginator's existing page-boundary checkpoints,
which are emitted after `pending_notes` and `page_note_ids` are empty. Keep
note-bearing tables and any traversal-sensitive block as conservative full
fallback.

When restarted pagination completes the body, append unchanged endnote pages
exactly once. Do not append them again when an exact cached tail already
contains them. Complete warm-versus-fresh `LayoutResult` equality remains the
authority for page elements, note areas, fields, outlines, source maps, and
diagnostics.

If a footnote, endnote, header, footer, section, or relevant relationship
changes, the existing exact retained-context comparison invalidates restart
state and performs safe full pagination. Do not add fingerprints, public API,
dependencies, modules, traits, or new test binaries.

## Rejected alternatives

- Treat the 32-page limit as unfixed. F-202 already closes it with a measured thousand-page gate.
- Store a second related-story fingerprint. Exact retained-context equality already owns that identity.
- Restart from a page with pending note continuation. The checkpoint intentionally refuses that state.
- Reuse note-bearing tables. Their traversal state is not represented by the current checkpoint.
- Use timing alone. Page invocation counts and complete equality prove bounded correct work deterministically.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `unchanged_footnote_and_endnote_context_restarts_only_at_note_clean_boundaries` | A 700-paragraph body edit uses note-clean restart, bounded page invocations, retained page identity, and exact fresh equality |
| unit | `unchanged_header_and_footer_context_keeps_restart_pagination_eligible` | A default header and page-number footer no longer disable safe restart |
| unit | `changed_related_story_context_invalidates_restart_state` | Same-body mutations to each note, header, and footer story force safe full pagination |
| unit | `a_footnote_continuation_never_creates_a_dirty_restart_boundary` | No checkpoint captures pending note state |
| regression | source-built Issue 53 facade workload in `regression_test.rs` | The public reusable path preserves bounded work and fresh equality for 700 paragraphs with one note and header/footer |
| regression | existing F-201 ignored release gate | Thousand-page layout and PDF performance remain within declared limits |

The **test gate is regression**. Source-built 700-paragraph note and
header-footer workloads retain bounded page work and exact warm-versus-fresh
output, while changed related stories and dirty note continuations invalidate
reuse.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout and pagination**. Use deterministic bundled fonts, exact warm and
  fresh structural equality, retained `Arc` identity, bounded invocation
  counts, and the unchanged F-201 release gate.
- **External reporter evidence**. Preserve the reporter's correction that
  F-202 fixed the page ceiling and that browser-wide attribution was withdrawn.
  Attribute only the note and header-footer cliffs this story proves.
- **WASM**. Run both WASM target checks because the reported editor path uses
  the reusable bundled-fallback facade.

## Hash harness

Expected unchanged across all 49 entries. This changes retained work only.
Any visible output delta or fresh-result difference blocks integration.

## Implementation checklist

- [ ] Add real failing engine and facade regressions in existing test files.
- [ ] Remove only the three global related-story disqualifiers.
- [ ] Admit safe paragraph note references and retain conservative table fallback.
- [ ] Append endnote pages exactly once on restarted completion.
- [ ] Prove changed related stories invalidate restart state.
- [ ] Prove note continuations never create dirty checkpoints.
- [ ] Run focused suites, F-201 release gate, WASM, hash, full verification, and microscope.
- [ ] Update exactly the four listed HLD files.

## Open questions

None. Issue 53's corrected report defines the remaining scope, and F-202 owns
the already completed 1,024-page capacity fix.
