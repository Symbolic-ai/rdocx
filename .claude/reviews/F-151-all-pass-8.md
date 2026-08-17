# F-151, all, pass 8

**Reviewed**: complete remediated working-tree diff against `HEAD`, 16 files, 1,461 changed lines, with 1,388 additions and 73 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, revision-only hyperlinks serialize stale modeled owner attributes
`crates/rdocx-oxml/src/text.rs:1107`
`crates/rdocx-oxml/src/text.rs:1128`
`crates/rdocx-oxml/src/text.rs:2135`

When a hyperlink has revision children but no direct runs, parsing records its
typed `rel_id`, `anchor`, and extra attributes, then also stores the entire
hyperlink subtree as paragraph raw XML. Serialization writes that raw subtree
and explicitly skips the typed empty hyperlink. A caller that changes the
public modeled owner fields after parsing therefore gets the original
attributes back on write instead of the changed values. Revision ownership and
source round-trip order are retained, but the new preservation path makes the
typed hyperlink state cease to be the serialization authority for exactly this
class of hyperlinks.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-7 D1 and D2 are resolved: revision-only hyperlink runs now retain their
relationship during layout and produce link annotations, and both PDF save
paths include the process id. Revision enumeration, raw-child ordering,
nested and scoped resolution, comment-removal remapping, and process-unique
path behavior produced no additional findings. Contract, panic safety,
tracked decoration, pagination, test coverage, and structural-rule review also
produced no findings. The `rdocx-oxml` and `rdocx-layout` unit suites, the
`rdocx` regression and integration suites, and the `rdocx-html` injection suite
pass. `git diff --check HEAD` passes.
