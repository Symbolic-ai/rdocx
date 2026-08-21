# F-X033, Integrate PR 36 ordered body items

**Status**: approved
**Sprint**: S51
**Size**: S
**Depends on**: F-X038

## Problem

PR 36 adds the first deliberately narrow ordered native reader over direct Word
body children. Existing `paragraphs()` and `tables()` accessors group by type,
so callers cannot recover the source interleaving of paragraphs, tables,
content controls, and preserved unmodelled XML.

The submitted three-file patch is additive and its GitHub checks are green,
but those checks ran against an older base before the S51 document facade
changes. Integration must preserve Pedro Assumpcao's commit and public pull
request record while proving the API over the current sprint result.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions" and body ownership.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and integration tests.
- `docs/hld/14-development-backlog.md`, "F-X033, Integrate PR 36 ordered body
  items".
- GitHub PR 36, `Expose ordered document body items`, contributor commit
  `79390535acba0a116b25ac986b863bdb941c8f15`.

## Approach

Retain the submitted public surface:

```rust
pub enum BodyItemRef<'a> {
    Paragraph(ParagraphRef<'a>),
    Table(TableRef<'a>),
    ContentControl(ContentControlRef<'a>),
    UnsupportedXml(&'a [u8]),
}

pub fn body_items(&self) -> impl Iterator<Item = BodyItemRef<'_>>;
```

After F-168, F-X032, and F-X034 are integrated, push only `sprint/s51` as the
pull-request base, retarget PR 36 from `main` to `sprint/s51`, and let current
GitHub CI run. Merge through GitHub with a merge commit, never squash, rebase,
or cherry-pick, so the contributor commit and PR record remain intact. Do not
push to the contributor branch or merge to `main`.

Keep maintainer hardening separate from the contributor commit. Add an
end-to-end test through the existing rdocx integration binary that opens an
in-code package and traverses only the public facade. Reconcile any semantic
conflict against this plan and the active S51 plans, then run a new microscope
over the integrated PR range. The old-base checks are evidence about the
submitted patch, not a substitute for current-tree verification.

## Rejected alternatives

- Squash or reimplement the patch. That loses the contributor commit and
  weakens the GitHub contribution record.
- Merge directly to `main`. Only `/close-sprint` may merge the sprint to main.
- Treat the submitted unit test as the full gate. It constructs private body
  content and does not prove parse plus public traversal.
- Expand to ordered cell or inline readers. PR 36 intentionally delivers one
  direct-body slice.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `body_items_preserve_paragraph_table_control_and_raw_order` | The submitted direct mapping reports all four variants in order |
| integration, gate | `public_body_items_preserve_opened_document_order` | An opened in-code document reports every direct paragraph, table, control, and raw child once in exact source order |
| regression | existing recursive accessor tests | `paragraphs()` and `tables()` retain their existing recursive semantics |
| integration | current-base GitHub CI | The retargeted PR passes the complete repository check graph before merge |
| packaging | rdocx dry run and archive inventory | The additive public enum and method package without archive growth beyond the limit |

The **test gate** is integration. An opened in-code document with interleaved
body children reports every direct child once in exact source order. The
submitted focused test, current-base GitHub CI, full package gate, and unchanged
hash harness also pass.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Public API of a published crate**. Read HLD 10 and the structural rules.
  The native reader is additive, with no binding expansion. Run the workspace
  package dry run and enforce the 10 MiB archive ceiling.

The GitHub integration also requires a current sprint branch, a current-base
CI result, a merge commit, and a public merge record that retains
`@pedroassumpcao`. These external mutations are explicitly requested by the
user and remain limited to PR 36 and `sprint/s51`.

## Hash harness

Expected unchanged across all 49 entries. The API is read-only and no sample
uses it.

## Implementation checklist

- [ ] Confirm the exact contributor head and current GitHub check state.
- [ ] Integrate F-168, F-X032, and F-X034 before retargeting the PR.
- [ ] Push only the reviewed current sprint base and retarget PR 36.
- [ ] Require current-base GitHub CI and merge with a GitHub merge commit.
- [ ] Add the public open-and-traverse integration gate separately.
- [ ] Run microscope over the current-base integrated result.
- [ ] Run full verification, packaging, and the unchanged hash harness.
- [ ] Update exactly the HLD files listed above.
- [ ] Preserve Pedro Assumpcao's contributor commit and GitHub merge record.

## Open questions

None. The user explicitly requested PR 36 in S51. Its submitted surface stays
narrow, and integration uses a GitHub merge commit on the sprint branch to
retain contributor credit.
