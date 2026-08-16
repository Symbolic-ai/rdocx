# S43 sprint review, pass 1

**Reviewed**: `sprint/s43` against `b2efea5`, 42 files, 3,678 insertions and 194
deletions. Crates: `oxml-layout`, `oxml-pdf`, `rdocx`, `rdocx-layout`,
`rdocx-oxml`. Plus `scripts/hash_harness.py`,
`scripts/test_sprint_workflow.py`, `.claude/commands/verify.md` with its
generated adapter, and four HLD sections.
**Verdict**: 0 blocking, 0 should-fix, 2 nice-to-have

## Blocking

None.

## Should-fix

None. The two candidates below were considered at this level and are recorded
as nice-to-have with backlog homes, because neither blocks the merge and both
are gaps the sprint narrowed rather than opened.

## Nice-to-have

### N1, CI does not run the release regressions the local gate now runs
`.github/workflows/ci.yml:206`

F-X025 put `python3 -m unittest scripts.test_sprint_workflow` into `/verify`
step 6, which closes the gap the story was written for: the preflights no longer
run for the first time on a tag. The `prose` job runs the sprint's other two
standard-library checks, `prose_check.py` and `sync_agent_skills.py --check`,
and does not run this one. A contributor who does not run `/verify` can still
move a version carrier and see a green pull request.

This is narrower than the defect S42 hit, since S42's F-X022 was authored
through the local gate, and the story's own test gate is satisfied. Filed as
**F-X026** rather than fixed here, because adding a CI job is a change to a
workflow file that the sprint's own release regressions assert over, and that
belongs in a story with its own gate rather than in a review remediation.

### N2, the golden-PNG gate is wired into nothing
`docs/hld/12-testing-strategy.md:78`

`scripts/golden_png_harness.py` generates deterministic PDFs, rasterises page
one at 150 DPI with the pinned Poppler oracle, and compares decoded pixels
against `scripts/golden_pixel_manifest.json`. The specification describes it in
full. It appears in no `/verify` step and no CI job, so it runs only when
somebody remembers it.

Pre-existing, and not caused by this sprint. It is recorded here because F-X021
went looking for what watches PDF output and found this one watching nothing,
which is the same class of gap the sprint exists to close. Filed as **F-X027**.

The two instruments are complementary rather than redundant: the hash harness
now covers PDF structure and bytes everywhere with no external tool, and the
golden gate covers rasterised pixels on page one with a pinned oracle. Neither
subsumes the other.

## Milestone gate

The sprint's definition of done, from `docs/sprints/CURRENT_SPRINT.md`, item by
item, with evidence rather than assertion.

**"A document carrying an unmodelled value for any of the nine enumerations
opens, keeps every sibling property, and renders the default for the unmodelled
one."** Holds. F-X018, `a_document_with_an_unmodelled_enumerated_value_still_
opens`, which fails against a single reverted call site.

**"A note is broken to the width of the section that references it."** Holds.
F-X017, `a_note_is_broken_to_the_width_of_its_own_section`. Confirmed to fail
against reverted code: registering only the final width makes it report equal
line counts for a wide and a narrow section.

**"A wrapping drawing anchored to a later paragraph pushes earlier text aside
even when it is positioned relative to its own paragraph."** Holds. F-X019,
`a_paragraph_relative_wrapping_drawing_pushes_earlier_text_aside`. Confirmed to
fail against the unfixed look-ahead: with the second pass disabled the earlier
paragraph takes 18 lines against 18 and the assertion fires.

**"The harness records a stable fingerprint for PDF output, and a deliberate
change to the PDF writer moves it while leaving the PNG entries untouched."**
Holds, and was **performed rather than asserted**. Perturbing the TJ adjustment
in `emit_glyphs` by one thousandth of an em moved 14 entries, `pdf/pages` and
`pdf/bytes` for all seven samples, and left every `pdf/resources`, `page1.png`
and `word/*.xml` entry untouched. Perturbing the `/Producer` string alone moved
only the seven `pdf/bytes` entries. Both were reverted, and the tree returns to
49 of 49.

**"`/verify --full` fails on a stale version literal in the release regressions
or the workflow files."** Holds, and was performed. Moving
`crates/rpptx/Cargo.toml` to 0.3.1 fails both preflights. Putting `ci.yml`'s
`@tensorbee/rpptx-wasm` literal back to 0.2.0, which is exactly the S42 defect,
fails three tests including both WASM job assertions. Both were reverted.

**"Every harness delta in the sprint is stated and justified in the commit that
causes it, including the deliberate re-record F-X021 requires."** Holds. The
sprint contains exactly one baseline change, `3913e88` on
`work/f-x021-claude`, preserved through the squash into `f9a560c`. Its message
states the delta as 21 added and lists the three entry kinds, and the
`--reason` recorded in the baseline file says the same. The other three stories
declared "unchanged" and the integrated gate confirms it.

## Harness reconciliation

The step that matters most, reconciled end to end:

| | Entries |
|---|---|
| Sprint base `b2efea5` | 28, verified green before any story landed |
| F-X017 declared | unchanged |
| F-X019 declared | unchanged |
| F-X025 declared | unchanged |
| F-X021 declared | 21 added, 0 changed, 0 removed |
| Integrated `c28a792` | **49 of 49 match** |

28 plus 21 is 49. No entry that existed before the sprint holds a different
value after it, which is checkable in the diff:
`git diff b2efea5 HEAD -- scripts/hash_baseline.json` is 22 added lines and one
removed, and the removed line is the previous `reason` string. No story produced
a delta it did not predict, and no delta lacks a story.

## Not found

- **interaction**. F-X017 and F-X019 both rewrote `paginator.rs`, which is why
  they were put in separate waves. Checked where they meet: F-X019 creates a
  fresh `Pager` per pass, so F-X017's `page_note_ids` and `pending_notes` cannot
  leak between passes, and the `NoteRegistry` is immutable and shared, so a
  second pass reserves the same note heights the first did. F-X021's writer fix
  and F-X019's second pass touch disjoint code, and no sample has a
  paragraph-relative wrap, so neither can move the other's output. All 53 test
  binaries pass on the integrated tree, against 80, 84 and 87 in `rdocx-layout`
  at the three worker heads.
- **duplication**. No helper was written twice. F-X017 added
  `note_baseline_count` and `make_two_section_input`, F-X019 added
  `body_line_count` and `make_lookahead_document`, and the second pair reuses
  the pre-existing `text_extents` rather than restating it.
- **layering**. No `oxml-*` manifest gained an `rdocx-*` or `rpptx-*`
  dependency. The only matches for those names in `crates/oxml-*/Cargo.toml` are
  `tag-name = "rpptx-v{{version}}"` release metadata.
- **deps**. No dependency added, removed or moved. `Cargo.toml` and `Cargo.lock`
  are byte-identical to the sprint base, which also satisfies the rider the
  release-scripting risk row demands of F-X025.
- **surface**. Two public changes, both declared in their plans before this
  review. `rdocx-layout`'s `NoteRegistry::build` and `get` changed
  incompatibly, a minor bump under 0.x. `oxml_layout::FontId` gained
  `PartialOrd` and `Ord`, additive. F-X019 added no public surface at all:
  `ResolvedWraps`, `PassContext`, `PassResult`, `paginate_pass`,
  `has_paragraph_relative_wrap` and `is_paragraph_relative_wrap` are all private
  to the module. Nothing was added that no story called for.
- **docs**. Every HLD section the sprint contradicted was updated by the story
  that contradicted it: `03-architecture.md` twice, for notes and for the
  two-pass rule, `08-rendering-spec.md` for writer reproducibility,
  `12-testing-strategy.md` for the harness, and `15-build-and-toolchain.md` for
  the local preflight gate. The stale release figures in `15` and `12` were
  corrected by F-X025. The "What CI runs" table still describes CI accurately,
  since CI did not change.
- **gate**. Covered above, in full, with evidence for all six items.

## Pass verdict

Zero blocking findings. **This pass is clean and no confirmation pass follows.**
The two nice-to-have findings have backlog homes, F-X026 and F-X027, created in
the same commit as this review.
