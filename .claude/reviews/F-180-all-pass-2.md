# F-180, all aspects, pass 2

**Reviewed**: Uncommitted F-180 implementation diff, 9 files, 2,382 additions and 42 deletions, plus pass-1 findings and remediation notes
**Verdict**: 7 defects, 4 smells, 0 nitpicks

## Defects

### D1, simple-field display text is dropped instead of flattened
`crates/rdocx/src/odt.rs:972`

The write path emits a field result only through `Field::projected_text()`,
which returns text only for a parsed complex field at
`crates/rdocx-oxml/src/text.rs:203`. A parsed `w:fldSimple`, or the
`Field::new("PAGE", "7")` value used by the loss regression, therefore emits
no visible `7` at all. The diagnostic says the field was flattened, but the
safe cached display was actually discarded. This violates the approved rule
to flatten a field when it has a safe display value.

### D2, public negative paragraph spacing does not round trip
`crates/rdocx/src/odt.rs:1168`

The public paragraph facade accepts signed lengths for spacing and indentation.
For example, `set_space_before(Length::twips(-20))` is serialized here as a
negative `fo:margin-top`. The F-179 reader parses that attribute through the
unsigned length path at `crates/rdocx/src/odt.rs:3401`, rejects the negative
value at `crates/rdocx/src/odt.rs:3518`, records a diagnostic, and reopens the
paragraph without its spacing. Zero or negative exact line height has the same
domain mismatch. The writer succeeds even though the declared write-read gate
cannot preserve supported paragraph formatting.

### D3, output limits are checked after all media bytes have been cloned
`crates/rdocx/src/odt.rs:624`

Every inline image occurrence clones its complete package part into `media`
without first charging the total-output or entry budget. The 128 MiB total and
entry-count checks run only after scanning finishes at
`crates/rdocx/src/odt.rs:232`. A document can reference one near-64 MiB image
thousands of times while keeping only one source media part. Export then clones
hundreds of gigabytes before returning the intended limit error. The output
path is therefore not bounded by the existing ODT defaults during construction.

### D4, most unsupported numbering semantics disappear without a diagnostic
`crates/rdocx/src/odt.rs:989`

The loss scan for a used numbering level examines only `num_fmt` and `start`.
It silently discards the level suffix, custom `lvl_text` or bullet glyph,
marker justification, level paragraph and run properties, retained level XML,
abstract-number metadata, and numbering-instance overrides. Those fields are
part of the retained numbering model at
`crates/rdocx-oxml/src/numbering.rs:2136`. Several affect the visible marker or
its layout. The plan explicitly requires producer numbering semantics that
F-179 cannot recover to be diagnosed.

### D5, table and row content-control wrappers are silently omitted
`crates/rdocx/src/odt.rs:336`

The table scan diagnoses raw table and row XML, then walks only `table.rows`
and `row.cells`. It never inspects `CT_Tbl::content_controls` or
`CT_Row::content_controls`, which are distinct typed collections at
`crates/rdocx-oxml/src/table.rs:1611` and
`crates/rdocx-oxml/src/table.rs:1434`. The writer likewise emits only rows and
cells at `crates/rdocx/src/odt.rs:822`. Supported siblings survive, but these
typed content controls vanish with no path-aware loss report.

### D6, inline image metadata and retained drawing details are lost silently
`crates/rdocx/src/odt.rs:570`

The drawing scan checks anchor presence, relationship resolution, dimensions,
and media format, but never reports a populated inline `description`, `name`,
or retained `raw_xml`. Emission replaces the source metadata with a generated
`draw:name` and writes no title or description at
`crates/rdocx/src/odt.rs:963`. A normal parsed image can therefore keep its
bytes and dimensions while losing accessibility text, crop or other retained
inline details without the diagnostic required for unsupported source content.

### D7, emitted font-family values are not validated as XML 1.0 text
`crates/rdocx/src/odt.rs:1204`

Run text is checked with `valid_xml_character`, but the public font-family
string is only XML-escaped before being placed in an attribute. Escaping does
not remove forbidden control characters. A run created with a font name such
as `"Bad\u{1}Font"` therefore makes `to_odt_bytes` return a package whose
`content.xml` cannot be parsed by the F-179 reader. Every caller-controlled
string that reaches XML needs the same character validation as run text.

## Smells

### S1, the declared round-trip gate omits most supported formatting facts
`crates/rdocx/tests/integration_test.rs:25`

`OdtParagraphRecord` checks text, alignment, numbering, and only bold, italic,
and color on runs. It does not record paragraph spacing, indentation, line
height, font family, size, underline, strike, highlight, or vertical position.
`OdtTableRecord` at `crates/rdocx/tests/integration_test.rs:33` reduces every
cell to joined text and spans, so it cannot distinguish multiple cell
paragraphs from one paragraph containing a line break, and it does not inspect
cell paragraph formatting or lists. The public test can stay green when the
approved normalized round-trip contract is broken, as D2 demonstrates.

### S2, the conformance test does not inspect the XML or manifest contract
`crates/rdocx/src/odt.rs:4004`

The test named `odt_writer_emits_conforming_deterministic_package` checks byte
repeatability, three entry names, and stored compression for `mimetype`. It
does not assert the local `mimetype` extra-field requirement, fixed namespace
and child order, manifest root and media entries, image entry order, or the
actual XML output bound. Most of the acceptance statement in the approved unit
test remains unproved.

### S3, the pass-1 loss matrix is still not the promised complete matrix
`crates/rdocx/src/odt.rs:4516`

The exact expected vector covers several body, paragraph, run, table, and media
losses, but it has no table or row content-control wrapper, no inline image
metadata or retained inline XML, and no numbering suffix, marker text, marker
formatting, or override case. D4 through D6 all remain invisible to the test,
so pass-1 S1 is not fully remediated.

### S4, cell-list import duplicates the complete body-list parser
`crates/rdocx/src/odt.rs:3037`

`build_cell_list` repeats the depth check, continuation diagnostics, style
lookup, fallback behavior, nine-level definition construction, item traversal,
start-value diagnostic, and recursion already implemented by `project_list` at
`crates/rdocx/src/odt.rs:2572`. The two copies already have different output
plumbing and must now receive every future list correction twice. This
increases the number of places a reader must inspect for one behavior.

## Nitpicks

None.

## Pass-1 remediation verified

- D1 through D9 are fixed in the current implementation. The small positive
  EMU cases, cell lists, level-8 starts, inherited numbering, substantive merge
  continuations, all three width locations, inherited formatting loss, CRLF,
  and checked span accumulation were inspected and exercised by the focused
  tests.
- S2 is fixed. The staging-collision test occupies all 128 sibling candidates
  while an existing destination remains byte-identical.
- S1 remains incomplete as described in pass-2 S3.

## Not found

- No additional defect was found in fixed namespace prefixes, ODF top-level
  child order, required ZIP entry order, deterministic ZIP metadata under the
  pinned feature set, table span emission, basic list nesting, media MIME and
  extension selection, atomic replacement cleanup, or source-document
  mutation.
- No reachable panic was found on untrusted table geometry after the checked
  accumulation remediation. The remaining `expect` and `unreachable` sites
  depend on invariants established by the immutable scan.
- No new crate, module, file, dependency edge, trait, generic parameter,
  feature flag, Python surface, WASM surface, or CLI surface was introduced.
- The six modified HLD files exactly match the approved HLD impact list and
  describe current behavior rather than change history. No unlisted HLD
  contradiction was found.
- Focused verification passed: 9 writer unit tests and the public writer
  integration test. `git diff --check` and the tracked prose check also passed.
