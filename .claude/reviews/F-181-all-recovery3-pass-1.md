# F-181, all recovery3, pass 1

**Reviewed**: the complete pre-review working tree diff across 19 files with
6,973 additions and 4 deletions, including the 5,585-line private EPUB writer,
the prior review evidence, the approved plan, and the cited HLD and risk
contracts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, a repeated JPEG start marker passes structural validation

`crates/rdocx/src/epub.rs:3587`

`valid_jpeg_structure` accepts `SOI` as a standalone marker anywhere after the
required leading `SOI`. JPEG permits that marker only once at the start of the
datastream. A byte sequence containing the leading marker, a second `SOI`, a
legal `SOF`, a legal `SOS`, scan data, and a terminal `EOI` therefore reaches
the successful return at line 3585. `oxml_media::probe` also accepts that shape
because it returns after the frame header. The malformed bytes are packaged as
supported `image/jpeg` instead of being diagnosed and omitted, contrary to the
HLD claim that every malformed image is rejected.

### D2, invalid GIF LZW code sizes and empty image data pass validation

`crates/rdocx/src/epub.rs:3668`

After an image descriptor and optional local colour table,
`valid_gif_structure` skips the LZW minimum code-size byte without validating
it. It then accepts an immediate zero-length sub-block as the whole image data
sequence. An otherwise valid one-pixel GIF whose code-size byte is changed to
zero, or whose image data is reduced to only the terminator, still reaches the
successful trailer return at line 3685. `oxml_media::probe` checks only the
logical screen descriptor, so these malformed bytes are packaged as supported
`image/gif` rather than diagnosed and omitted.

## Smells

None.

## Nitpicks

None.

## Not found

- Targeted recovery3 remediation: direct `Heading7+` and direct outline levels
  above six are projected as `h6` and diagnosed as reduced. A custom
  style-derived level above six remains a paragraph and is now diagnosed as
  flattened, matching its XHTML. PNG chunk validation checks all four type
  bytes for ASCII letters and requires the reserved third byte to be uppercase.
  A valid ancillary private `teXt` chunk remains accepted.
- Archive and EPUB structure: no additional defect was found in the stored
  first `mimetype`, container, package metadata, manifest, spine, navigation,
  stylesheet, XHTML flow structure, fixed timestamps, compression choices, or
  ZIP entry order.
- Loss diagnostics and source semantics: no additional defect was found in
  headings, styles, defaults, numbering, revisions, fields, hyperlinks,
  tables, drawings, shading, underline, spacing, breaks, or sibling retention.
- Bounds, panics, determinism, atomicity, and preservation: no unchecked
  production panic, overflow, recursion escape, unbounded export allocation,
  unstable output choice, destination truncation, staging leak, document
  mutation, or retained DOCX XML mutation was found.
- Contract and structure: the additive native API, approved private module,
  existing `zip` dependency, unchanged Python, WASM, and CLI surfaces, and six
  modified HLD files match the approved plan.
- Focused evidence: all 33 ordinary EPUB tests passed, with the external test
  ignored in the ordinary run. The source-built oracle passed the exact
  checksum-verified EPUBCheck 5.3.0 JAR separately. Prose and diff checks also
  passed. The media fixture exercises the repaired PNG cases but has no JPEG or
  GIF structural-validation case covering D1 or D2.
