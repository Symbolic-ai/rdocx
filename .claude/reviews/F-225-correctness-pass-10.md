# F-225, correctness, pass 10

**Reviewed**: current working tree implementation across 17 feature files,
8,949 inserted lines and 27 deleted lines from `597a27c`
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, additional-actions dictionaries are validated as action dictionaries

`crates/rpptx/src/pdf.rs:2272`

The active-content scanner sends the value of every `AA` entry directly into
an action context. In PDF, `AA` is an additional-actions dictionary whose event
keys contain actions. It is not itself an action dictionary with `S` and `URI`.
This happens to reject an ordinary conforming `AA` dictionary because its outer
dictionary lacks `S`, but it accepts the malformed shape
`/AA << /S /URI /URI (https://example.com) >>`. The accepted value is neither a
valid additional-actions dictionary nor an inert field under the scanner's own
classification. It therefore bypasses the required active-content and
malformed-state boundary at `.claude/plans/F-225-design.md:103`. The only `AA`
regression constructs the same direct action-shaped value with JavaScript at
`crates/rpptx/src/pdf.rs:7656`, while the strict action table exercises only
Catalog `OpenAction` and annotation `A` at `crates/rpptx/src/pdf.rs:7750`.

### D2, malformed UTF-16 URI strings are normalized before validation

`crates/rpptx/src/pdf.rs:4185`

`decode_pdf_string` uses `chunks_exact(2)` and `String::from_utf16_lossy` for a
BOM-marked PDF string. An unmatched trailing byte is discarded, and invalid
surrogate sequences are replaced rather than rejected. The strict URI path
calls this decoder before `safe_uri` at `crates/rpptx/src/pdf.rs:4673`, so an
odd-length encoding of an otherwise allowed URL is shortened to that valid URL
and accepted. An invalid surrogate in its path is likewise converted to the
replacement character and can pass the scheme and whitespace checks. The
malformed URI regression at `crates/rpptx/src/pdf.rs:7741` covers an ordinary
space but does not cover malformed string encoding. This is silent malformed
state normalization at the action security boundary.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 9 disposition

- D1 is closed for the action contexts it now recognizes. Catalog
  `OpenAction`, annotation `A`, and `Next` inside an already validated action
  enter the strict path at `crates/rpptx/src/pdf.rs:2269`. Direct scalar, array,
  stream, and name action values fail at `crates/rpptx/src/pdf.rs:2211`. Indirect
  chains use raw object lookup with separate active and completed sets at
  `crates/rpptx/src/pdf.rs:2164`, so missing targets, cycles, depth, and graph
  work fail within explicit bounds. Action dictionaries require a name `S`
  equal to `URI` and a string URI in an allowed scheme at
  `crates/rpptx/src/pdf.rs:4648`. Incidental `S`, `OpenAction`, `A`, and `Next`
  entries outside their semantic owners remain ordinary data. D1 and D2 above
  are residual `AA` shape and URI string-decoding defects, not reopenings of
  the direct action-form cases reported in pass 9.

## Final full-diff audit

No additional lossy action or resource convenience path was found. Page-tree,
resource, XObject, content, annotation, font, and embedded-stream references
remain strictly resolved with the previously reviewed cycle, depth, work,
type, filter, decompression, retention, and allocation limits. Prior closures
remain intact for ToUnicode sharing, ordinary font caches, explicit text
positioning, source widths, affine geometry, stroke and dash semantics, image
accounting, retained elements, transactional publication, relationship
ownership, public feature gating, WebAssembly selection, and raw oracle
sensitivity.

The focused action-context regression passes. The current diff still changes
exactly the nine HLD files named at `.claude/plans/F-225-design.md:185`, with no
unapproved public feature, trait, generic parameter, wrapper, builder, crate,
or integration-test binary.
