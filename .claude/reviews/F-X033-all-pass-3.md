# F-X033, all, pass 3

**Reviewed**: complete integrated PR 36 range plus current maintainer hardening, 6 files and 248 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 2's preservation defect is remediated. Every self-closing body child that
does not match an intended typed Word name now reaches raw capture at
`crates/rdocx-oxml/src/document.rs:757`. The regression includes a foreign
`<ext:body/>`, asserts its exact stored bytes, and confirms it remains present
in canonical output at `crates/rdocx-oxml/src/document.rs:1073`. No unsupported
empty child has a remaining drop branch in `CT_Body`.

The intended Word `p`, `tbl`, and `sectPr` cases remain expanded-name checks at
`crates/rdocx-oxml/src/document.rs:750`. Default and aliased Word prefixes are
typed, foreign same-local-name elements remain raw, and the private empty
section constructor preserves the paired parser's prior field values. Typed
output uses the existing fixed `w:` writers, raw children retain body order,
and modeled section properties remain schema-final at
`crates/rdocx-oxml/src/document.rs:774`. The parser regression serializes and
reparses the complete result at `crates/rdocx-oxml/src/document.rs:1080`.

The public opened-package gate exercises a self-closing paragraph and section
properties at `crates/rdocx/tests/integration_test.rs:64`. Its exact sequence
assertion proves that the empty paragraph is a `Paragraph`, that section
properties do not become an extra item, and that ordinary paragraphs, a table,
a content control, and producer XML retain source order. The recursive
paragraph assertion remains compatible with the existing nested-control gate.

No findings were found in the borrowed public lifetime, direct iterator
projection, raw-byte borrowing, parse-to-facade ownership, panic safety,
recursive paragraph or table access, additive semver scope, Rust API docs,
binding isolation, contributor attribution, or merge record retention. The
namespace parser, public integration, contributor unit, and recursive-accessor
focused tests passed locally. The reported full changed-crate tests, Clippy,
49-entry hash harness, prose check, and diff checks are also green.
