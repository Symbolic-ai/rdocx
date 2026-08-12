# F-136, all, pass 1

**Reviewed**: the complete working diff from `4b128371`, 26 files and 2,367 changed lines including the 13-file untracked `rpptx-py` crate
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, placeholder index zero does not apply the OOXML default
`crates/rpptx/src/lib.rs:2358`

`SlideRef::placeholder()` matches only an explicit `p:ph@idx`, so the common
title placeholder whose omitted index has the OOXML default of zero cannot be
found as index zero. The Python collection delegates directly to this lookup at
`crates/rpptx-py/src/shape.rs:513`. With a fresh wheel, the title slide produced
by `Presentation().slides.add_slide(prs.slide_layouts[0])` raises `IndexError`
for `slide.placeholders[0]`, while pinned `python-pptx==1.0.2` returns its title
placeholder. The documented-example test only asks for placeholder index one at
`crates/rpptx-py/tests/test_documented_examples.py:35`, so it does not expose
the incompatible default-index behavior.

### D2, text replacement silently retargets held descendant handles
`crates/rpptx-py/src/text.rs:77`

`TextFrame.text` replaces every paragraph and run without bumping the revision.
Likewise, `Paragraph.text` replaces every ordered text choice at
`crates/rpptx-py/src/text.rs:148`, and `Shape.text` can replace the shape text
body at `crates/rpptx-py/src/shape.rs:117`. A previously held paragraph or run
therefore remains revision-valid and resolves to the newly created node at the
same index. A fresh-wheel reproduction held a run, assigned `paragraph.text`,
and then read the new text through the old run instead of receiving
`StaleElementError`. These assignments are structurally destructive in the
facade, as shown by the replacement implementations at
`crates/oxml-drawing/src/text/mod.rs:256` and
`crates/oxml-drawing/src/text/paragraph.rs:1445`. This violates the plan's
path-handle invalidation contract at `.claude/plans/F-136-design.md:29`.

### D3, the differential gate covers only one writer and one example
`crates/rpptx-py/tests/test_documented_examples.py:230`

The sole differential test authors only the table example with `rpptx`, then
opens that file with both readers. It does not compare normalized records for
the other six promised examples, and it never authors a presentation with the
pinned oracle and opens it with `rpptx`. The HLD requires the rpptx comparison
in both directions at `docs/hld/12-testing-strategy.md:357`, while the plan
names normalized example records at `.claude/plans/F-136-design.md:65`.
Consequently, the gate cannot detect reader incompatibilities in files emitted
by python-pptx or structural divergence in the title, bullet, textbox, picture,
preset-shape, and extraction examples.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML boundary, facade layering,
PyO3 ownership, lazy indexing and slicing, revision-counter bump-count,
unit-conversion, oracle-version pin, release-metadata, dependency-direction,
packaging, HLD-impact, formatting, prose, skill-drift, hash-harness, or artifact
findings were found. A fresh cp39-abi3 wheel built successfully, and its focused
Python suite passed 5 tests with exact `python-pptx==1.0.2`. The focused
`oxml-py-support` suite passed 6 tests, the facade regression passed, and
`cargo check -p rpptx-py --all-targets` passed. The unchanged hash gate matched
all 28 entries, and the workflow suite passed all 24 tests.
