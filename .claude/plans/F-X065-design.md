# F-X065, Expose tracked table grid changes

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-X064

## Problem

PR 56 exposes the presence of `w:tblGridChange`, but its submitted parser
matches local names without proving WordprocessingML namespace identity and
silently keeps only the first modeled change. A grid change stores a historical
table grid. It must round-trip without replacing or influencing the active
`w:gridCol` values used by layout.

The current `CT_TblGrid` parser in `crates/rdocx-oxml/src/table.rs` and the
native table reader in `crates/rdocx/src/table.rs` do not expose this historical
snapshot.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, namespace-aware WordprocessingML parsing,
  raw subtree preservation, and schema child order.
- `docs/hld/08-rendering-spec.md`, active table-grid ownership in layout.
- `docs/hld/10-bindings-spec.md`, native Word inspection and pre-1.0 API policy.
- `docs/hld/12-testing-strategy.md`, parser, package, and layout regression evidence.
- `docs/hld/14-development-backlog.md`, "F-X065, Expose tracked table grid changes".

## Approach

Make `tblGrid`, `gridCol`, and `tblGridChange` recognition URI-aware through
the existing in-scope Word namespace machinery. Reject foreign elements that
reuse the same local names. Preserve exactly one historical
`w:tblGridChange` subtree and serialize it after the active `w:gridCol`
children in schema order. A second modeled change is an error rather than
silently discarded history.

Keep the active `columns` vector as the only layout grid. Add the submitted
native reader outcome as `TableRef::has_grid_change()`. The historical bytes
remain inspection and round-trip data only. If the implementation adds
`grid_change_xml: Option<Vec<u8>>` to public `CT_TblGrid`, record the intentional
pre-1.0 exhaustive-literal source impact and update every existing literal.

Use PR 56 at commit `8b79c4cd0452defafe0a58e86b332c98e7fe52d7`
as contribution evidence, then implement the hardened equivalent from the
integrated F-X064 head. Do not merge, retarget, comment on, or close the PR.

## Rejected alternatives

- Cherry-pick PR 56 unchanged. Local-name matching and first-only retention are not safe OOXML semantics.
- Treat the historical grid as active. That changes current column widths and contradicts the revision meaning.
- Drop duplicate modeled changes. Silent data loss violates the preservation contract.
- Expose the complete historical grid through another public wrapper. The submitted requirement needs presence inspection and preserved bytes, not a forwarding surface.
- Add a new module or test binary. Existing table parser, facade, and layout test modules cover the complete path.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| parser | `tracked_table_grid_change_is_namespace_aware` | Canonical and aliased Word elements parse, while foreign same-local elements remain unmodelled |
| negative | `duplicate_modeled_table_grid_changes_fail_closed` | A second Word grid change is rejected without discarding bytes |
| round trip | `tracked_table_grid_change_round_trips_after_active_columns` | Historical XML survives save and reopen in valid child order |
| facade | `table_ref_reports_preserved_grid_change` | `TableRef::has_grid_change()` reports the modeled historical snapshot |
| layout | `historical_table_grid_never_changes_active_column_widths` | Conflicting historical widths do not affect current layout |
| integration | current Word corpus gate | Tracked tables open and render after the exact locked offline no-default build |

The **test gate is regression**. The focused OXML, facade, and layout tests,
the current Word corpus job, and `/verify --full` must pass.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Require URI-aware alias and foreign-namespace
  cases, valid child-order output, duplicate rejection, and raw subtree
  round-trip evidence.
- **Layout, pagination, line breaking, text shaping**. Use deterministic fonts
  and prove the historical grid cannot alter active layout.
- **Public API of a published crate**. State the pre-1.0 exhaustive-literal
  impact, run the patched publish dry run, and enforce archive limits.
- **An external oracle comparison**. Use the pinned Word corpus and package
  save-reopen evidence under the differential-testing contract.

## Hash harness

Expected unchanged at 49 of 49. Historical grid data is preserved for native
inspection and save-reopen only. It is never consumed by layout.

## Implementation checklist

- [ ] Add failing URI, foreign-element, duplicate, and schema-order tests in existing files.
- [ ] Preserve one exact historical grid-change subtree.
- [ ] Keep active columns as the only layout input.
- [ ] Add the native presence accessor and document its additive API impact.
- [ ] Run focused OXML, facade, layout, corpus, package, and risk-rider gates.
- [ ] Run microscope and `/verify --full`.
- [ ] Record PR 56 and its exact source SHA in the handoff and delivery evidence.

## Open questions

None. The historical grid is preserved and inspectable, while the active grid
remains the sole layout authority.
