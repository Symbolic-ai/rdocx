# S17 sprint review, pass 2

**Reviewed**: `sprint/s17` against `bdb89af`, 38 files, 6,488 changed
lines, crates: `oxml-drawing`, `rpptx-oxml`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M8 end gate is: "all 50 corpus decks round-trip, and every one opens in
PowerPoint without a repair prompt."

S17 does not complete M8, so the manual PowerPoint half is not yet due and was
not claimed. The S17 slice holds. All 40 `rpptx-oxml` integration tests pass,
and each of the placeholder, picture, graphic-frame, and table corpus gates
passes across all 50 pinned decks. The hash harness reports all 28 entries
unchanged when run with an isolated target directory.

## Not found

No findings in interaction, duplication, layering, harness attribution, sprint
gate coverage, HLD alignment, dependency consumers, or unrequested public API
surface. Pass 1 B1 is fixed by giving `Event::Empty` an explicit one-element
namespace scope, and its regression proves that a later sibling retains the
ancestor binding after a local empty-element shadow. Pass 1 S1 is fixed by the
single concrete application-properties parser in `placeholder.rs`, used by
both shapes and pictures. No Cargo manifest changed, and the integrated
dependency tree retains the documented direction.
