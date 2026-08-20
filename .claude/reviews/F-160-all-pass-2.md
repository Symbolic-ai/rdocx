# F-160, all, pass 2

**Reviewed**: complete staged and unstaged implementation diff against `HEAD`, 10 tracked files, 2,544 changed lines, with 1,882 additions and 662 deletions. The untracked pass-1 review was read as open history and excluded from the implementation count.
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, HTML and Markdown exports drop every projected complex-field display
`crates/rdocx-html/src/emitter.rs:320`
`crates/rdocx-html/src/markdown.rs:217`

The parser now replaces a valid complex field's result runs with one
`RunContent::Field`, but both text exporters discard that variant. A complex
DATE, AUTHOR, INCLUDETEXT, or other field that previously exported its ordinary
stored-result `w:t` now contributes no HTML or Markdown text. The layout repair
for the same regression does not reach either exporter.

### D2, complex-field projection discards cached-result run formatting during layout
`crates/rdocx-oxml/src/text.rs:744`
`crates/rdocx-layout/src/engine.rs:914`

`field_run` gives every projected complex field a synthetic run with no run
properties. Layout resolves font, size, emphasis, colour, spacing, and baseline
from that synthetic run before it handles the field. A cached result stored in
bold, italic, sized, coloured, or otherwise formatted result runs therefore
renders with the paragraph defaults. The private source retains the original
runs for serialization, but the rendered behavior still changes.

### D3, tabs and breaks in a complex cached result disappear
`crates/rdocx-oxml/src/text.rs:873`
`crates/rdocx-oxml/src/text.rs:881`

Complex result collection emits text only for direct `w:t` children. Empty
`w:tab` and `w:br` children are ignored, after which projection removes the
source runs. A valid cached result such as `left`, tab, `right` becomes
`leftright`, and a stored line or page break is lost from text and layout. The
simple-field path uses `CT_R::text()` and does not have this mismatch.

### D4, clearing dirty does not clear dirty markers on separate or end
`crates/rdocx-oxml/src/text.rs:2244`

Complex parsing merges dirty state from begin, separate, and end markers. The
source-preserving rewrite changes only the outer begin marker and writes every
separate and end marker unchanged. If the producer placed `w:dirty="1"` on the
separator or end marker, setting `field.dirty = Some(false)` writes a new false
begin marker beside the retained true marker. Reopening the output merges the
field back to dirty true, so the public mutation does not round-trip.

### D5, cached-result mutation is ignored when the outer result has no direct text
`crates/rdocx-oxml/src/text.rs:2208`

The complex source rewrite can replace only a direct outer-result `w:t`. It has
no fallback corresponding to the simple-field insertion when no such element
exists. A valid outer field whose stored display consists entirely of a nested
field, or of non-text result content, accepts a new `cached_result` in memory
but serializes the old result because `wrote_result` remains false.

### D6, canonical instruction output loses empty and backslash-leading operands
`crates/rdocx-oxml/src/text.rs:2507`

The canonical token writer quotes only values containing whitespace or a quote.
An empty quoted operand is emitted as no token, while a quoted UNC path such as
`\\server\share\file.docx` is emitted without quotes and is parsed back as a
switch. Any structured instruction edit that selects canonical output can
therefore corrupt operands that the repaired lexer correctly parsed.

### D7, legal complex fields inside an explicit hyperlink are never projected
`crates/rdocx-oxml/src/text.rs:1693`
`crates/rdocx-oxml/src/text.rs:956`

Runs parsed from `w:hyperlink` are inserted into the paragraph with every raw
run source set to `None`, and the complex-field stack skips every such run. A
begin, instruction, separator, result, and end sequence wholly inside a
hyperlink remains ordinary raw markers instead of becoming the shared `Field`
grammar. The field corpus covers direct paragraph runs only, so this valid run
placement does not exercise the promised unified complex parser.

### D8, an invalid nested field in a result does not invalidate its outer field
`crates/rdocx-oxml/src/text.rs:1012`

An invalid nested field propagates failure to its parent only while the parent
is still reading its instruction. If the parent is already in its result, the
invalid child is discarded and the parent remains valid. For example, a nested
result field with two separators is malformed, but the enclosing field is still
projected and absorbs the malformed sequence into typed source. This contradicts
the required opaque fallback for malformed complex sequences.

## Smells

None.

## Nitpicks

None.

## Not found

Panics and structure produced no additional findings. The pass-1 same-run
range panic is removed, and the diff adds no trait, generic parameter, crate,
module, file, or feature flag. Prefix scoping for direct run children and the
malformed-simple fallback are repaired. Contract, correctness, OOXML
preservation, and test coverage findings are recorded above.
