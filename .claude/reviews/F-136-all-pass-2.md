# F-136, all, pass 2

**Reviewed**: the complete current working diff from `4b128371`, 26 implementation files and 2,862 changed lines, plus pass-1 review and remediation notes
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the differential assertion is insensitive to writer-only drift
`crates/rpptx-py/tests/test_documented_examples.py:466`

The expanded test now authors all seven records in both directions, but it only
compares the two readers on each writer's file. It never compares the normalized
rpptx-authored records with the normalized python-pptx-authored records or with
fixed expectations. Both readers can therefore agree on incorrect output from
one writer. An independent mutation check added an extra slide only to the
rpptx-authored Hello World deck. Both assertions at this line still passed,
while the two writers' normalized Hello World records differed. The same gap
allows ignored bullet levels, font formatting, picture geometry, and preset
shape semantics to pass whenever both readers report what was actually written.
Pass-1 D3 is expanded to all seven examples and both read directions, but its
writer-parity sensitivity is not resolved.

### D2, owner refresh bypasses the approved global revision contract
`crates/rpptx-py/src/shape.rs:94`

A stale shape is accepted when its separately stored owner slide has been
refreshed, and shape collections bypass revision validation through the same
rule at `crates/rpptx-py/src/shape.rs:250`. Structural append operations update
that owner revision at `crates/rpptx-py/src/shape.rs:314`. Consequently, after
adding a second textbox through a held shape collection, an independently held
first shape, the old collection, and the old slide all remain readable. The
HLD instead says every handle captures its revision at construction and no
snapshot accessor keeps working after invalidation at
`docs/hld/10-bindings-spec.md:59`. It also says rpptx handles store only a
presentation reference and `ContentPath` at
`docs/hld/10-bindings-spec.md:171`, while the remediation adds owner pyclass
references to shapes, collections, text frames, and paragraphs. This is an
unapproved alternate identity and invalidation model, even though the currently
exposed append operations do not shift earlier indices.

### D3, stale errors do not provide complete handle-specific recovery paths
`crates/rpptx-py/src/text.rs:22`

Every text handle uses the same recovery hint ending at
`prs.slides[i].shapes[j].text_frame`. That is incomplete for a paragraph, run,
font, paragraph collection, or run collection because it omits the remaining
`paragraphs[k]`, `runs[l]`, or property step. Shape validation similarly ends
at the collection at `crates/rpptx-py/src/shape.rs:104`, and table validation
ends at the table at `crates/rpptx-py/src/table.rs:19`. The plan promises stale
errors with a complete recovery path at `.claude/plans/F-136-design.md:32`.
The remediation helper asserts only the two revision numbers at
`crates/rpptx-py/tests/test_documented_examples.py:215`, so replacing or
truncating these hints does not fail the gate.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1 is resolved. Omitted `p:ph@idx` now maps to zero in both placeholder
lookup and exposed placeholder identity, and its regression test passes.
Pass-1 D2's direct failure mode is resolved. `Shape.text`, `TextFrame.text`, and
`Paragraph.text` each bump once after success, refresh the mutation receiver,
and make previously held descendants stale. The focused remediation selection
passed 3 tests, the complete fresh-wheel Python suite passed 7 tests with exact
`python-pptx==1.0.2`, `oxml-py-support` passed 6 tests, the facade regression
passed, and `cargo check -p rpptx-py --all-targets` passed.

No additional panic, OOXML child-order or preservation, facade layering, PyO3
borrow safety, indexing, slicing, iteration, scalar-write bump, unit conversion,
oracle pin, release metadata, dependency direction, package layout, HLD impact,
formatting, diff-hygiene, or generated-artifact findings were found. Review-run
pytest and bytecode caches were moved to `/private/tmp` and are absent from the
worker.
