# F-X033, all, pass 2

**Reviewed**: complete integrated PR 36 range plus current maintainer hardening, 6 files and 243 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a foreign self-closing body lookalike is silently deleted

`crates/rdocx-oxml/src/document.rs:757`
`.claude/plans/F-X033-design.md:63`
`crates/rdocx-oxml/src/document.rs:1063`

The remediated `Event::Empty` branch correctly uses expanded names for `p`,
`tbl`, and `sectPr`, but its raw fallback still excludes any element whose
local name is `body`. An unmodelled `<ext:body/>` inside the Word body therefore
matches no typed arm and is not appended as raw XML. A save silently drops it.
The revised contract says every foreign or otherwise unsupported empty element
remains raw, and the preservation rider requires exact retention. The new unit
test covers only the foreign `p` lookalike, so it remains green. The body
boundary does not arrive in this nested parser as an empty child. The fallback
can preserve this element, with an exact foreign-body round-trip regression.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1's modeled-empty defect is otherwise remediated. Empty Word paragraphs,
tables, and section properties use namespace-aware recognition at
`crates/rdocx-oxml/src/document.rs:750`. Foreign `p`, `tbl`, and `sectPr`
lookalikes fall through to raw capture. The private empty section constructor
also preserves the paired-element parser's prior field values without adding
a public construct.

Typed empty paragraphs, tables, and section properties serialize with fixed
`w:` names and in the existing schema positions. Raw children remain at their
body-content positions, and section properties remain schema-final at
`crates/rdocx-oxml/src/document.rs:774`. The new unit gate checks typed initial
parsing, exact foreign `p` bytes, fixed-prefix output, and successful reparse at
`crates/rdocx-oxml/src/document.rs:1061`.

The public opened-package gate now includes both `<w:p/>` and `<w:sectPr/>` at
`crates/rdocx/tests/integration_test.rs:64` and verifies the empty paragraph's
position and recursive accessor compatibility. The contributor unit gate, the
public integration gate, the namespace-aware parser gate, and the existing
recursive-accessor regression all passed locally.

No additional findings were found in public lifetime safety, direct ordering,
raw-byte borrowing, recursive accessor behavior, panic safety, semver scope,
public documentation, binding isolation, contributor attribution, or merge
record retention. The reported changed-crate tests, Clippy, 49-entry hash
harness, prose check, and diff checks are green.
