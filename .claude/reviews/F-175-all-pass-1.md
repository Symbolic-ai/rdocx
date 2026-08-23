# F-175, all, pass 1

**Reviewed**: the working tree on `work/f-175-codex`, 7 tracked modified
files plus the approved new `crates/rdocx/src/redaction.rs`, 1,373 additions
and 4 deletions
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, split rich text survives in charts and embedded workbooks

`crates/rdocx/src/redaction.rs:377`

The generic XML pass replaces a selector only when it is wholly contained in
one text event. A chart label such as two DrawingML runs containing `sec` and
`ret`, or a rich shared string containing the same two SpreadsheetML `t`
nodes, therefore receives zero replacements. The later raw scan also sees XML
markup between the fragments, so it does not find contiguous `secret` bytes
and the mutation commits with the sensitive semantic value still present.
Word paragraphs have a separate cross-run pass, but the approved contract also
requires exact literals to disappear from DrawingML labels, shared strings,
and inline strings.

### D2, property-change revision authors are not redacted

`crates/rdocx/src/redaction.rs:757`

The Word attribute allowlist recognizes only `ins`, `del`, `moveFrom`, and
`moveTo` as revision owners. This repository also models `rPrChange`,
`pPrChange`, `tblPrChange`, and `sectPrChange` as revisions, all with
`w:author`. A valid `w:pPrChange w:author="secret"` is left untouched and then
causes the residual scan to reject the complete operation. Redaction therefore
cannot handle all revision metadata promised by the design and HLD.

### D3, foreign drawing lookalikes are modified

`crates/rdocx/src/redaction.rs:770`

The `docPr` and `cNvPr` branch checks only the local element name and an
unqualified attribute. It does not check the element namespace. A preserved
producer element such as `<x:docPr descr="secret"/>` is consequently rewritten
even when `x` is a foreign namespace. This contradicts expanded-name matching
and the explicit foreign same-local-name preservation requirement.

### D4, editing one sensitive attribute rewrites the whole start tag

`crates/rdocx/src/redaction.rs:661`

Once one attribute matches, the rewriter rebuilds the complete start tag and
all of its attributes. Unrelated lexical bytes such as whitespace, single
quotes, and entity spellings change along with the sensitive value. For
example, `<w:ins  w:id='1' w:author='secret'>` is normalized rather than having
only the author value span patched. The contract requires unaffected byte
ranges and unrelated attributes in preserved XML to remain byte-identical.

### D5, Word matching crosses semantic text boundaries

`crates/rdocx/src/redaction.rs:526`

Every sensitive text node in a paragraph is concatenated without boundaries.
This joins field instructions to displayed text, text on opposite sides of a
hard break, nested text-box paragraphs, and accepted and rejected revision
branches. For example, inserted text `sec` followed by deleted text `ret` is
removed even though neither the accepted nor rejected view contains the exact
literal `secret`. Each revision branch and logical text flow must be searched
independently while ordinary contiguous runs remain joinable.

### D6, CDATA replacement can emit malformed XML

`crates/rdocx/src/redaction.rs:405`

`BytesCData::new` requires that its content not contain `]]>`, but deleting a
selector can create that sequence across the deletion boundary. Valid content
such as `]]secret>` becomes `]]>` and is written as an invalid CDATA section.
Chart and workbook XML is not reparsed as typed XML by the outer document
reopen, so this can publish a package whose related XML is malformed. The
replacement must split CDATA safely or emit escaped ordinary text, followed by
an XML reparse of every rewritten sensitive part.

### D7, the named regression tests do not prove their recorded contracts

`crates/rdocx/tests/regression_test.rs:5513`

The atomic test compares serialized bytes for the failure variants, but it
does not prime or compare layout caches and does not compare typed views after
the malformed, residual, limit, or external-workbook failures. The round-trip
test at `crates/rdocx/tests/regression_test.rs:5611` checks one unrelated binary
part and one relationship serialization, but not content types, schema child
order, or unchanged same-part XML ranges. The raw helper at
`crates/rdocx/tests/regression_test.rs:5449` scans only ordinary part payloads,
not the inflated content-types and relationship entries. The foreign-name test
also covers `x:t` but not the `docPr` and `cNvPr` attribute path that D3 exposes.
These tests can stay green while several explicit test-plan assertions are
false.

## Smells

None.

## Nitpicks

None.

## Not found

- Atomic staging structure beyond D6 and D7: the live `Document` is not
  replaced until package serialization, residual scanning, bounded reopen, and
  relationship validation succeed. The reusable layout engines are retained
  only at commit.
- OPC relationship handling beyond D2: chart and story targets are resolved
  relative to their owning parts. External chart and workbook relationships
  fail closed. Missing internal targets and missing content types are rejected.
- Package limits: outer and nested OPC opens use explicit entry, part-size, and
  total-size limits, and nested ZIP depth is capped.
- Panic handling: zero findings. Production indexing and slicing introduced by
  the feature are guarded by parser positions, non-empty selectors, match
  ranges, or collection membership.
- Public API isolation: zero findings. The additive native method and report
  type appear only in `rdocx`. Python, WASM, and CLI wrappers gain no redaction
  method.
- Structure: zero findings. The only new module is the explicitly approved
  `crates/rdocx/src/redaction.rs`. No new trait, generic parameter, feature
  flag, crate, forwarding wrapper, or dependency-family edge was introduced.
- HLD scope and voice: zero findings. Exactly the four plan-listed HLD files
  change, their additions describe current behavior, and the reviewed prose
  contains no prohibited punctuation.
- Hash-harness scope: no sample or baseline file changes are present.
