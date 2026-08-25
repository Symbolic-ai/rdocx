# F-X055, all, pass 1

**Reviewed**: the complete 23-file working-tree diff, 199 additions and 105
deletions, plus the approved plan, progress record, four-file HLD impact,
authenticated GitHub inventory, rendered release body, package metadata,
publication workflow, and generated archives
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Defects

### D1, the release notes omit the completed image export addition

`CHANGELOG.md:16`

The `Added` inventory does not mention F-183 even though it is a user-visible
stable-family addition in the `v0.9.0..HEAD` evidence range. F-183 added
selected-page export as transparent PNG, quality-controlled JPEG, and
multi-page TIFF across native Word, Python, and both general CLI paths at
`docs/sprints/AS_BUILT.md:8796`. The sprint acceptance contract requires the
v0.10.0 notes to name every addition at
`docs/sprints/CURRENT_SPRINT.md:80`. Publishing the current body would omit a
shipped release capability, and the new release-note regressions would still
pass because they assert only the nine external records. Add the F-183 outcome
to the reviewed release body and bind that addition in the v0.10.0 regression.

### D2, two release-note bullets collapse format-specific guarantees into false claims

`CHANGELOG.md:20`

RTF is a deterministic byte stream, not an archive, but the current writer
bullet says both RTF and OpenDocument Text have deterministic archives. The
next bullet at `CHANGELOG.md:22` jointly says EPUB and SVG retain headings,
lists, tables, and accessibility structure. EPUB retains those document
semantics, while the SVG backend lowers marked content to an ordinary `<g>` at
`crates/rdocx/src/svg.rs:233` and preserves searchable fixed-page geometry,
images, and safe links rather than document accessibility structure. Split
the format-specific outcomes so the exact published body does not attribute
ODT packaging to RTF or EPUB semantics to SVG.

### D3, the HLD promises a post-publication check that the release workflow does not perform

`docs/hld/12-testing-strategy.md:1001`

The updated current-state HLD says the stable 0.10.0 release gate verifies
non-empty rendered HTML at every crates.io README endpoint. The canonical
release command verifies registry versions and owners, the tag target, the
byte-identical GitHub release body, and record notifications at
`.claude/commands/release.md:112`, but contains no README endpoint check. The
approved F-X055 release test plan also does not add one. As written, the HLD
describes an unenforced future action as current reality. Either add the check
to the approved release contract and its workflow regression, or remove the
unsupported HLD claim.

## Smells

### S1, the old next-stable inventory regression now passes against the wrong section

`scripts/test_sprint_workflow.py:4225`

`test_next_stable_inventory_credits_layout_follow_up_records` names its slice
`unreleased`, but terminates it at `v0.9.0`. With `v0.10.0` inserted first,
that slice now includes the entire versioned release body even though the real
Unreleased section contains no record links. The test therefore passes for a
condition it no longer checks and will keep accumulating later stable
sections. Retire it or make it target the intended section. The two exact
v0.10.0 contribution checks at `scripts/test_sprint_workflow.py:4159` and
`scripts/test_sprint_workflow.py:4178` already cover the moved records.

## Nitpicks

None.

## Not found

- Version carriers, pins, and publication authority: metadata confirms all 11
  shared-version packages and both Python project versions at 0.10.0, exactly
  nine workspace pins at 0.10.0, and exactly the seven approved stable crates
  publishable. All 16 incubating preparation packages and 15 publishable
  incubating crates remain at 0.5.0.
- Cargo lock and literals: all 11 inherited lockfile packages, rdocx WASM
  assertions, stable CI literal, seven README requirements, README gate, and
  publication preflight name agree at 0.10.0.
- Publication workflow: the stable and incubating predicates are disjoint,
  allowlists contain exactly 7 and 15 dependency-ordered packages, real
  publish commands are bare and failure-propagating, and registry waits remain
  between layers.
- Contribution inventory: all nine records were authenticated read-only on
  GitHub. Issue 44, PR 45, and Issue 46 are closed, PR 45 is unmerged, and PRs
  47 through 52 are open and unmerged. The authenticated handles are
  `@emptinessform` and `@pedroassumpcao`. Every record is accurately classified
  as a hardened equivalent, appears twice in the rendered notes, has specific
  contributor credit, and has one unposted release-bound comment with the tag,
  release link, outcome, classification, and thanks.
- Compatibility: the notes explicitly name `ST_NumberFormat::Other(String)`,
  removal of `Copy`, the exhaustive-match break, and the borrow-or-clone
  migration action.
- Packaging: the patched dirty-tree dry run staged exactly 22 packages and
  uploaded nothing. No archive exceeded 10 MiB. `oxml-layout` contained all 20
  TTFs, three licence files, and the Caladea notice. `rdocx-layout` contained no
  font copy. `oxml-pdf` contained the ICC profile and legal file, and `rpptx`
  contained `assets/default.pptx`.
- HLD scope: exactly the four plan-listed HLD files changed. Apart from D3,
  they describe the prepared stable boundary, unchanged incubating family,
  unpublished bindings, and separate final approval as current reality.
- External mutation: no tag, push, publication, release, comment, closure, or
  record-state mutation ran during this review.
- Gates observed: release-note check and render passed, all 71 workflow tests
  passed, the focused allowlist mutation tests passed, prose reported zero
  violations, all 26 generated skills were in sync, and `git diff --check`
  passed.
- Panics, OOXML, and structural API additions: this release-preparation diff
  adds no parser, serializer, trait, generic, crate, module, or runtime API.
