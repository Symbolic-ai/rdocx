# F-278, microscope, pass 1

**Reviewed**: Working implementation against approved F-278 design on work/f-278-codex. Three files, 770 insertions and 17 deletions. Source remained frozen during review.
**Verdict**: 4 defects, 0 smells, 0 nitpicks.

## Defects

### D1, typed switch operands are restricted by an incomplete parser whitelist

`crates/rdocx-oxml/src/text.rs:1016`

The checked constructor rejects any explicit switch operand absent from
`switch_takes_argument` at `crates/rdocx-oxml/src/text.rs:11183`. For example,
an unknown producer field with a typed q switch and Text operand cannot be
authored, although the approved contract permits unknown names and switches.
This also blocks nested operands on those switches. Checked construction must
preserve explicit typed operands, with a round-trip grammar policy that retains
their positions. Merely accepting an operand while reparsing it as a positional
argument does not satisfy the contract. Add coverage for text and nested switch
operands, including an unknown producer switch.

### D2, legal page and column cache breaks fail checked attachment

`crates/rdocx-oxml/src/text.rs:587`

Ordered cached runs containing Page or Column breaks are accepted during
construction, then projected to U+000C or U+000B at
`crates/rdocx-oxml/src/text.rs:1092`. Attachment unconditionally validates that
projection as XML text and rejects it. Those controls serialize as legal w:br
nodes, not literal XML characters. Thus Run::add_field_value and embedding the
field as a nested operand fail for valid caches. Validate the actual serialized
cache and distinguish an unchanged display projection from a caller's invalid
replacement string. Test both break kinds through attachment and reopen.

### D3, replacing an equal-text nested field silently retains the old source

`crates/rdocx-oxml/src/text.rs:807`

The unchanged-source shortcut compares instructions using Field::PartialEq,
which ignores selected form and ordered cache properties at
`crates/rdocx-oxml/src/text.rs:968`. Replace a parsed nested operand with a newly
authored field having identical instruction, display, lock and dirty values but
different form or cache formatting. The parent still compares unchanged, so
`crates/rdocx-oxml/src/text.rs:7854` writes its old raw XML and discards the
replacement. The later source-identity guard cannot run. Account for nested
source replacement before the unchanged shortcut, and test both nested argument
and nested switch operand replacement with preserved display text.

### D4, opaque foreign elements are mistaken for Word field delimiters

`crates/rdocx-oxml/src/text.rs:1119`

Cached opaque XML is rejected solely by local name. A well-formed producer
extension such as p:fldChar with xmlns:p="urn:producer" is not a Word field
control but is rejected as one. This violates the namespace-aware preservation
rider. Resolve namespaces when identifying actual Word delimiters, retaining
foreign lookalikes verbatim. Test a foreign lookalike and a real Word delimiter
under an aliased prefix to prove both sides of the boundary.

## Smells

None found.

## Nitpicks

None found.

## Not found

Panics: no new panic path found in checked construction or the lock writer.
Structure: no new trait, generic, source file or forwarding wrapper found.
OOXML child ordering: no new ordering defect found. Namespace handling has D4.
Tests: existing tests exercise the new surface, but miss D1 through D4. The
ordered-cache test covers line breaks only and the nested instruction test
covers an argument only. These gaps are included with their defects above.
Contract and correctness: D1 through D3 prevent the declared round-trip gate.
Lock and dirty preservation: no additional defect found in the three-state
attribute edits and unchanged-source preservation.
