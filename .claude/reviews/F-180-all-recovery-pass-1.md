# F-180, all aspects, recovery pass 1

**Reviewed**: Entire uncommitted F-180 implementation diff, 9 files, 5,946 additions and 1,996 deletions, plus all three original reviews and the recovery progress record
**Verdict**: 9 defects, 1 smell, 0 nitpicks

## Defects

### D1, writer output can still exceed reader run and XML-node ceilings
`crates/rdocx/src/odt.rs:480`

The recovery adds block, row, and emitted-cell counters, but the writer still
does not count the runs or XML nodes that its output creates. A paragraph with
100,001 alternating styled one-character runs remains far below the XML byte
limit and below the 300,000-node limit, so the writer succeeds. Reopening then
fails at the reader's 100,000-run ceiling at
`crates/rdocx/src/odt.rs:3620`. Independently, one run containing more than
300,000 spaces emits one `text:s` element per space at
`crates/rdocx/src/odt.rs:1791`, then fails the reader's XML-node ceiling at
`crates/rdocx/src/odt.rs:2416`. The byte API can still publish a package that
cannot pass its declared F-179 write-read boundary.

### D2, an interrupted numbered list silently restarts at one
`crates/rdocx/src/odt.rs:1104`

The body writer closes a list whenever any non-list body item interrupts the
consecutive paragraph run. If later paragraphs resume the same Word `numId`,
it emits a new `text:list` without `text:continue-numbering` or
`text:continue-list`. ODF 1.3 section 19.786 gives an absent continuation
attribute the non-continuing behavior, so the visible marker restarts at one.
The writer neither preserves the numbering sequence nor diagnoses this
producer numbering semantic that F-179 cannot recover.

### D3, non-ASCII whitespace does not survive the round trip
`crates/rdocx/src/odt.rs:1791`

`write_odf_text` special-cases only ASCII space, tab, carriage return, and line
feed. It writes a non-breaking space or another Unicode whitespace character
as ordinary XML text. F-179 feeds ordinary text through `char::is_whitespace`
at `crates/rdocx/src/odt.rs:3677`, which collapses that character to an ASCII
space or drops it at a boundary. A supported run such as `"a\u{a0}b"`
therefore reopens as `"a b"`, contrary to the exact text and whitespace
round-trip contract.

### D4, malformed relationship target modes are treated as internal
`crates/rdocx/src/odt.rs:813`

The image scan rejects only the exact string `External`. The relationship
parser retains arbitrary `TargetMode` strings, so a malformed value such as
`external` or `Bogus` reaches `relationship_bytes`, resolves against the
package, and can copy a probeable part into the ODT. The approved design
requires malformed pictures to be diagnosed and omitted. Only `None` and the
valid explicit `Internal` value should take the internal target path.

### D5, unresolved final header and footer references can disappear silently
`crates/rdocx/src/odt.rs:1629`

`final_section_has_unsupported_properties` clears every typed header and footer
reference before comparing the final section with the default. The separate
document scan diagnoses only relationships whose type is already `HEADER` or
`FOOTER`. A retained `w:headerReference` whose relationship is missing or has
the wrong type therefore triggers neither path and is dropped without the
stable loss diagnostic promised for unsupported document stories.

### D6, vertical-merge validation can accept a continuation after an overlap
`crates/rdocx/src/odt.rs:1989`

For a continuation cell, validation searches backward across all earlier rows
until it finds any cell starting at the same column. It skips an immediately
preceding row whose horizontally spanning cell covers that column but starts
elsewhere. A restart followed by one valid continuation, then a full-width
ordinary cell, then another continuation is accepted. Emission gives the
restart a two-row span, writes the overlapping full-width cell, and later
writes an unbacked covered cell. F-179 reopens the last cell as an ordinary
empty cell, so the table span contract is lost rather than rejected.

### D7, a used but undefined numbering level falls back without a diagnostic
`crates/rdocx/src/odt.rs:1334`

When the abstract numbering definition exists but has no entry for the
paragraph's selected level, `list_level_is_bullet` silently returns decimal.
This is reachable through the public facade because a paragraph accepts any
level from 0 through 8 even when the definition was built from fewer levels.
The existing deep-list regression uses a one-level bullet definition at level
8 and checks only the level, so it misses the silent bullet-to-decimal
fallback. The producer numbering loss must be named.

### D8, recovery adds an unapproved public facade method
`crates/rdocx/src/odt.rs:149`

`Document::numbering_level_is_bullet` is a new public method on the published
facade. The approved plan's public API block contains only `OdtWriteResult`,
`to_odt_bytes`, and `save_odt`, and its risk routing forbids public surface the
story did not request. None of the six approved HLD files documents this new
numbering API. Using it only to build the integration record does not remove
the semver and contract expansion.

### D9, out-of-range outline levels are silently clamped
`crates/rdocx/src/odt.rs:1271`

The writer clamps every retained paragraph outline level to 0 through 8 before
emitting `text:outline-level`. The OOXML parser retains the source value as an
unbounded `u32`, while neither projection validation nor loss scanning rejects
or diagnoses a larger value. A paragraph with `w:outlineLvl w:val="9"`
therefore exports as Heading 9 and reopens with level 8, silently changing a
retained paragraph property.

## Smells

### S1, the conformance walk proves only local element shapes
`crates/rdocx/src/odt.rs:4110`

The recursive assertion checks allowed child element names and selected local
attributes for one fixture. It does not validate state that crosses sibling or
row boundaries, including list continuation and whether every covered table
cell is backed by a live horizontal or vertical span. That is why D2 and D6
remain green under the test described as exhaustive ODF subset conformance.
The original pass-3 S2 is improved, but its semantic validation gap is not
closed.

## Nitpicks

None.

## Recovery findings verified

- Original pass-3 D1 is fixed for direct `numId=0` cancellation, including an
  inherited numbering definition.
- Original pass-3 D2 is fixed for wrong relationship types and the exact valid
  `External` target mode. D4 above covers the remaining malformed-mode case.
- Original pass-3 D3 is fixed. Every emitted inline frame carries
  `text:anchor-type="as-char"`.
- Original pass-3 D4 is fixed at the tested upper truncating-EMU boundary.
- Original pass-3 D5 is fixed for blocks, rows, and emitted cells. D1 above
  covers the reader ceilings that remain unmirrored.
- Original pass-3 D6 is fixed for zero, maximum, and above-maximum half-point
  font sizes.
- Original pass-3 D7 is fixed for the tested color, shading pattern,
  foreground, invalid fill, and valid fill plus highlight combinations.
- Original pass-3 D8 is fixed for normal document background, changed final
  section, and valid header and footer relationship cases. D5 above covers
  retained references that do not have a matching typed relationship.
- Original pass-3 S1 now records the nested level's actual bullet or numbered
  kind. D8 above records the resulting unapproved facade expansion.
- Original pass-3 S2 now recursively inspects the emitted element vocabulary,
  required inline anchor, dimensions, links, and local child order. S1 above
  records the remaining cross-element semantic gap.

## Not found

- **Panics**: no new reachable panic was found in scanned styles, media,
  ordinary table geometry, ZIP construction, or atomic replacement. Writer
  `expect` sites are backed by immutable scan facts.
- **Determinism and packaging**: no defect was found in ZIP entry order,
  compression choice, fixed metadata, mimetype local-header extras, manifest
  membership, generated media order, fixed namespace prefixes, or repeated
  write bytes.
- **Atomic save and ownership**: serialization completes before staging,
  staging collisions preserve the destination, failed replacement removes the
  temporary file, and ODT export does not mutate source DOCX bytes or retained
  XML.
- **Formatting and bounds already covered**: the tested paragraph length,
  line-height, font-size, image-size, field display, XML character, media-part,
  entry, diagnostic, block, row, cell, and checked span-arithmetic boundaries
  are correct.
- **Structure and HLD file scope**: no new crate, module, source file,
  dependency, feature, trait, generic parameter, or binding surface was added.
  Apart from D8, the six modified HLD files exactly match the approved HLD
  impact list and describe current state rather than history.
- **Verification evidence**: all 18 focused writer unit tests and the exact
  public writer round-trip integration test pass. `git diff --check` and the
  tracked prose check also pass.
