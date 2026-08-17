# F-151, all, pass 3

**Reviewed**: complete remediated working-tree diff against `HEAD`, 12 files, 945 changed lines, with 898 additions and 47 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, empty revision wrappers draw a change bar
`crates/rdocx-oxml/src/revision.rs:96`
`crates/rdocx-layout/src/engine.rs:187`

A self-closing or otherwise empty revision wrapper, such as an empty `w:ins`,
is parsed as `RevisionContent::Marker` and contributes no run to either text
projection. `revision_is_visible` nevertheless treats every marker as visible.
In tracked view, a paragraph containing only that empty wrapper is therefore
marked for a margin bar even though its tracked projection contains no visible
revision content. This violates the design contract that bars identify visible
revisions.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-2 D1 and D2 are fully resolved. Pass-2 S1 is fully resolved. Panic safety,
OOXML preservation and schema ordering, test coverage apart from D1, and
structural-rule compliance were checked and produced no findings. Contract and
correctness produced only D1 above.
