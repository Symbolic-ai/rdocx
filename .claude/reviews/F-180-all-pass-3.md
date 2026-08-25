# F-180, all aspects, pass 3

**Reviewed**: Entire uncommitted F-180 implementation diff, 9 files, 4,446 additions and 1,254 deletions, plus both prior reviews and the remediation record
**Verdict**: 8 defects, 2 smells, 0 nitpicks

## Defects

### D1, direct numId zero turns a list cancellation into a decimal list
`crates/rdocx/src/odt.rs:1363`

`paragraph_numbering_properties` treats every present `num_id` as a list,
including zero. The existing Word exporters treat zero as no list, and a
direct `w:numId w:val="0"` is how a paragraph cancels numbering inherited from
its style. This writer instead allocates an unknown decimal list, emits the
paragraph inside `text:list`, and reports only an unknown-definition fallback.
The inherited-numbering remediation therefore has the inverse cancellation
case wrong and can add a visible marker that was not in the source.

### D2, image relationships are resolved without checking type or target mode
`crates/rdocx/src/odt.rs:701`

`image_bytes` selects a relationship only by id, then resolves its target as an
internal package path. It never requires the image relationship type and never
rejects `TargetMode="External"`. A malformed drawing whose embed id names a
non-image relationship can therefore copy any probeable internal part. An
external relative target that happens to collide with an internal image part
is also copied instead of receiving the promised external-image diagnostic.

### D3, inline images are emitted with paragraph anchoring semantics
`crates/rdocx/src/odt.rs:1110`

The emitted `draw:frame` has no `text:anchor-type="as-char"`. Under ODF 1.3
Part 3 section 19.759, the missing value defaults to `paragraph`, whose frame
must occur at the start of the paragraph. This writer can place the frame after
arbitrary run text, and even a leading frame is no longer an inline object in
another ODF consumer. The F-179 reader ignores anchoring, so the local
write-read test masks both the conformance error and the changed image flow.

### D4, large positive image dimensions exceed the reader domain
`crates/rdocx/src/odt.rs:626`

The image scan validates only that each EMU dimension is positive. It then
serializes any `i64` value as points, while the F-179 length parser rejects
values above 1,000,000 points at `crates/rdocx/src/odt.rs:3726`. For example,
an otherwise supported inline PNG at `Length::emu(12_700_000_001)` writes
successfully, but reopening the result fails. The small positive boundary fix
does not cover the upper boundary of the same declared truncating-EMU domain.

### D5, writer limits permit output that the reader refuses structurally
`crates/rdocx/src/odt.rs:295`

The writer charges an approximate XML-byte budget but never charges the
existing block, row, or cell ceilings. A source with 100,001 empty paragraphs
stays below the 64 MiB part budget and writes successfully, then F-179 rejects
it through `bump_blocks` at `crates/rdocx/src/odt.rs:3388`. The same mismatch
exists for more than 10,000 compact table rows and more than 50,000 cells,
whose reader checks are at `crates/rdocx/src/odt.rs:3186` and
`crates/rdocx/src/odt.rs:3318`. The public byte method can therefore publish a
bounded package that cannot satisfy its own write-read gate.

### D6, font sizes outside the reader domain are serialized as supported
`crates/rdocx/src/odt.rs:1469`

`text_style_xml` emits every `HalfPoint`, including zero and values above the
reader's 1,000,000-point ceiling. The public run setter can construct zero
through a nonpositive or non-finite point value, and retained OOXML can carry
the same typed value directly. F-179 reports the generated `fo:font-size` as
invalid and reopens the run without its size, while the writer returns no loss
diagnostic. Paragraph domains are validated before emission, but the equally
public run-size domain is not.

### D7, retained run colour and shading semantics still disappear silently
`crates/rdocx/src/odt.rs:1508`

The run projection silently omits a valid OOXML automatic or otherwise
non-hex `color`, ignores a shading pattern and foreground colour, and chooses a
valid shading fill over a simultaneous highlight. None of those cases is
reported by `run_properties_have_unsupported` at
`crates/rdocx/src/odt.rs:1661`. Patterned shading changes visible output, and
even an unrepresentable colour is a retained source property. This leaves the
promised complete run-property loss matrix incomplete after the pass-2
remediation.

### D8, visible document-level stories are outside the loss scan
`crates/rdocx/src/odt.rs:223`

The first pass starts and ends with `body.content`. It never inspects the final
body section properties, document background, header and footer relationships,
or their visible parts. A document with a header, footer, changed page section,
or background therefore exports without any diagnostic for that dropped
content. The HLD now promises stable diagnostics for unsupported Word content,
and the approved plan specifically requires unsupported content outside the
F-179 boundary to be named rather than silently omitted.

## Smells

### S1, the round-trip record reads list kind from level zero only
`crates/rdocx/tests/integration_test.rs:90`

The normalized record calls `numbering_is_bullet(id)`, which examines the
first numbering level rather than the paragraph's recorded level. The gate's
source deliberately uses bullet level zero and decimal level one, but both
paragraphs are recorded as bullets. The writer can regress the nested level's
bullet-versus-numbered kind while the declared round-trip gate remains green.

### S2, the conformance test uses the permissive importer as its XML validator
`crates/rdocx/src/odt.rs:4241`

The test parses generated XML with F-179's bounded tree parser, which checks
well-formed namespaces but does not enforce the ODF schema or element
semantics. That is why D3 passes the test. The assertions cover many concrete
package requirements added after pass 2, but the test named for conformance
still needs an ODF-aware validation or an exhaustive assertion of the emitted
subset's required semantics.

## Nitpicks

None.

## Prior findings verified

- Pass-1 D1 through D9 remain fixed. Small EMU dimensions, cell lists, deep
  starts, inherited numbering, merge-continuation content, width diagnostics,
  inherited losses, CRLF, and checked table arithmetic are present and covered.
- Pass-1 S2 remains fixed by exhausting every sibling staging candidate while
  preserving existing destination bytes.
- Pass-1 S1 has substantially broader exact diagnostics, but it is not complete
  because of D7 and D8.
- Pass-2 D1 through D7 are fixed for their cited cases. Simple fields retain
  cached display, paragraph domains reject unsupported values, media is charged
  before cloning, retained numbering containers are named, table and row
  controls are named, drawing metadata is named, and caller strings are checked
  for XML 1.0 characters.
- Pass-2 S1's listed formatting and cell-record omissions are filled. S1 above
  is a separate per-level numbering-kind error in the expanded record.
- Pass-2 S2's listed ZIP, namespace, order, manifest, media, and bound assertions
  are present. S2 above is the remaining schema and semantic validation gap.
- Pass-2 S3's named numbering, control, and inline-drawing cases are present.
  The wider loss claim remains incomplete because of D7 and D8.
- Pass-2 S4 is fixed. Body and cell lists share
  `collect_list_paragraphs`, `collect_list_item_paragraphs`, and
  `build_paragraph`.

## Not found

- **Panics**: no reachable panic was found on untrusted table geometry or
  package content. Writer `expect` sites depend on immutable facts established
  by the completed scan, and string-formatting unwraps are infallible.
- **ODF package and XML**: apart from D3, no additional defect was found in
  namespace prefixes, top-level child order, list or table child order,
  manifest membership, MIME agreement, ZIP entry order, compression, local
  mimetype extra fields, or deterministic metadata.
- **Correctness and source ownership**: no additional defect was found in
  exact whitespace, CRLF normalization, supported formatting projection,
  shared body and cell list import, table coverage cells, media ordering,
  source-document mutation, staging cleanup, or atomic replacement.
- **Structure**: no new crate, module, source file, dependency, trait, generic
  parameter, feature flag, wrapper-only abstraction, or binding surface was
  introduced.
- **HLD and API**: the six modified HLD files exactly match the approved impact
  list, describe current behavior, and introduce only the approved additive
  native Rust API. No unlisted HLD contradiction was found.
- **Verification**: all 13 focused writer unit tests and the public writer
  integration test pass. `git diff --check` and the tracked prose check pass.
