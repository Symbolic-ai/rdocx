# F-X016, correctness, pass 2

**Reviewed**: the uncommitted working tree after the pass 1 remediation.
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Defects

None. D1 was found and fixed during pass 1 and is covered by
`a_drawing_anchored_to_a_later_paragraph_still_pushes_text_aside`, which fails
against the look-ahead reverted.

## Smells

None outstanding.

### S1 from pass 1, filed rather than fixed

A wrapping drawing anchored to a later paragraph and positioned relative to that
paragraph still does not push earlier text aside. Filed as **F-X019,
Paragraph-relative drawings in later blocks should wrap (M)**, depending on
F-X016, with its own test gate.

Closing it means paginating twice, since such a drawing has no position until
its own paragraph is placed. That is a design change rather than a review-time
patch, and no sample or corpus document reaches the gap. The narrower case the
look-ahead does cover is the one the external contribution's own document needs.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs`, a drawing straddling the centre of the
  text area picks a side rather than splitting the line. Carried from pass 1 and
  kept: Word does the same for square wrapping.

## Not found

Re-checked after remediation, all still clean: **correctness**, **panics**,
**ooxml**, **structure**, **performance**, **contract**, **tests**. The
remediation added a backlog entry and changed no code. The full suite, clippy,
formatting, the harness at 28 of 28, the prose rules, the Codex adapter check,
the WASM targets and the bundled-fonts-off path all pass.
