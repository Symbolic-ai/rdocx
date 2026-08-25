# F-181, all recovery3, pass 2

**Reviewed**: the complete pre-review working tree diff across 20 files with
7,156 additions and 4 deletions, including the 5,692-line private EPUB writer,
all ten prior review records, the approved plan, the progress note, and the
cited HLD and risk contracts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, a JPEG scan can precede its frame header

`crates/rdocx/src/epub.rs:3585`

`valid_jpeg_structure` treats `saw_sof` and `saw_sos` as unordered booleans at
the terminal EOI check. A source-built datastream containing a leading SOI, a
length-correct SOS, a valid SOF, and a terminal EOI makes
`oxml_media::probe` return image information at the SOF and makes this
validator return true. JPEG requires a frame header before the first scan, so
the malformed bytes are packaged instead of receiving the required diagnostic
and omission.

### D2, a zero-sized GIF image descriptor passes validation

`crates/rdocx/src/epub.rs:3656`

`valid_gif_structure` reads the nine-byte image descriptor but inspects only
its packed field. A GIF with a valid one-pixel logical screen and global colour
table, a zero image width or height in the image descriptor, a valid LZW
minimum code size and nonempty data block, and a terminal trailer passes both
`oxml_media::probe` and this validator. GIF image dimensions must be nonzero,
so this malformed core image is packaged instead of diagnosed and omitted.

## Smells

None.

## Nitpicks

None.

## Not found

- Targeted recovery3 remediation: a second SOI marker is rejected before a
  frame and when encountered in scan data. GIF LZW minimum code sizes outside
  2 through 8 and an image-data sequence containing only its terminator are
  rejected. The exact valid JPEG and GIF controls remain accepted and packaged.
- Archive and EPUB structure: no additional defect was found in the stored
  first `mimetype`, container, package metadata, manifest, spine, navigation,
  stylesheet, XHTML flow structure, fixed timestamps, compression choices, or
  ZIP entry order.
- Loss diagnostics and source semantics: no additional defect was found in
  metadata, styles, defaults, numbering, revisions, fields, hyperlinks, tables,
  drawings, shading, underline, spacing, breaks, or supported-sibling retention.
- Bounds, panics, determinism, atomicity, and preservation: no unchecked
  production panic, overflow, recursion escape, unbounded export allocation,
  unstable output choice, destination truncation, staging leak, document
  mutation, or retained DOCX XML mutation was found.
- Contract and structure: the additive native API, approved private module,
  existing `zip` dependency, unchanged Python, WASM, and CLI surfaces, and six
  modified HLD files match the approved plan.
- Focused evidence: all 33 ordinary EPUB tests passed, with the external test
  ignored in the ordinary run. The source-built oracle passed the exact
  checksum-verified EPUBCheck 5.3.0 JAR separately. Formatting, prose, and diff
  checks passed. The current media fixture does not exercise D1 or D2.
