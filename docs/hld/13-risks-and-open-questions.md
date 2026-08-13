# 13, Risks and open questions

## Settled decisions

### Preset shape definition provenance

The 187 preset geometries use the official
[ECMA-376 fifth-edition Part 1 archive](https://ecma-international.org/publications-and-standards/standards/ecma-376/),
`ECMA-376-1_5th_edition_december_2016.zip`. The precise source is
`OfficeOpenXML-DrawingMLGeometries.zip/presetShapeDefinitions.xml`, whose
SHA-256 is
`2f7c868d857c1e3c4b5a6068759fe0e07d77ad58377a6618d1b02ba3507b6939`.
The exact count and digest are generator inputs, not advisory metadata.

The [Ecma software policy](https://ecma-international.org/policies/by-ipr/ecma-international-policy-on-submission-inclusion-and-licensing-of-software/)
classifies XML data sets as source code. Software incorporated in an Ecma
standard is available under the policy's three-clause BSD licence. Any checked-in
table derived from this XML must retain the Ecma copyright notice, all three
conditions, and the disclaimer. This third-party notice does not change the
repository's MIT OR Apache-2.0 licence.

LibreOffice's preset table must not be used. Its MPL-2.0 file-level copyleft is
outside the repository's approved licensing model. Derivation from the
specification text remains the fallback if the official archive, count, digest,
or notice cannot be reproduced exactly.

## Open questions, to settle before the milestone that needs them

### Q2, PyPI name availability

The future crates.io names in the publishing graph are controlled by
`mantissaman` through 0.0.0 placeholders. PyPI has not been checked. If either
`rdocx` or `rpptx` is taken there, maturin's `module-name` allows shipping a
distribution such as `rdocx-python` while keeping `import rdocx`. Claim both
PyPI names as soon as the decision to ship wheels is confirmed.

## Risks, ranked

### R1, silent output drift during the extraction

**The top risk.** M4 and M5 change unit conversion, text-shaping input types and
content-type derivation. All three alter output without failing to compile, and
the existing 320 tests cannot see it.

*Mitigation*: the hash harness, built first and gating every PR. Every
intentional change lands as its own labelled commit with a reviewed delta. Do
not fold behaviour changes into moves. PDF alpha is additionally gated on
ExtGState structure and a deterministic midpoint compositing pixel.

### R2, the PDF coordinate-system flip

Replacing the per-element Y flip with one global CTM touches every element type
in a shipped, working renderer.

*Mitigation*: its own reviewable commit, landed **before any pptx code exists**,
gated on golden-PNG diffs comparing pixels rather than PDF bytes. The reviewed
Poppler 26.01.0 baseline declares exactly four stroke-antialias pixel changes
across `invoice` and `quote`. Normal check mode remains exact with no tolerance.

### R3, `Group`-blind collection passes

Three passes in the PDF backend must stay aligned with recursive content
emission. Missing one produces PDFs with absent fonts, images or link
annotations for grouped content.

*Mitigation*: all three passes use the mandatory `walk()` helper. Image and
annotation resources share depth-first leaf ordinals with recursive emission,
link rectangles apply the accumulated transform, and each pass has an explicit
nested-target regression test.

### R4, inheritance correctness

The layout-to-master match is by type, not `idx`. The text chain is seven levels
deep and level-indexed. The colour map is per-master and a dark master inverts
it. Wrong here means subtly wrong fonts, positions and colours on real
templates, which users notice and cannot describe.

*Mitigation*: M9 is a standalone milestone with visual differential tests and a
table of sampled theme-colour resolutions asserted to exact RGB.

### R5, schema child ordering

Diffuse, because it touches every writer, and violations are silent until
PowerPoint refuses the file.

*Mitigation*: `OrderedRawChildren`, plus the corpus-wide "opens without repair"
gate at M8 and M11.

### R6, raw-XML preservation against relationship remapping

The preservation strategy that makes lossless round-tripping possible is exactly
what makes deep copy dangerous, because `r:id` attributes hide inside preserved
blobs.

*Mitigation*: `rewrite_rel_ids`, and `add_slide` synthesising rather than
deep-copying so the common path never needs it.

### R7, scope

Charts plus full parity in one release is **17 to 18 months solo** with nothing
shipping until the end. Both halves of that were chosen deliberately. The
duration was not: the plan was estimated at phase granularity and said nine to
twelve months, and the story-level sizing in `14-development-backlog.md`
disagrees. The story-level number is the trustworthy one.

*Mitigations, neither requiring rework*: a second developer takes it to roughly
9 to 11 months, since M7 and M8 parallelise once M6 lands. Or cut a
read-plus-render release at the end of M10, about 12 months solo, which is the
point where the library becomes useful. Charts are self-contained and nothing
depends on them, so M12 can move independently at any time.

### R8, `oxml-layout` packaging

The bundled fonts are 6.8 MB outside `src/`, published today only because
`cargo publish --no-verify` skips the build-from-archive check, and there is no
`include` or `exclude`.

*Mitigation*: an explicit `include`, drop `--no-verify`, and assert the `.crate`
size against the 10 MiB crates.io limit in CI. Roughly 3.5 to 4 MB compressed is
expected, but verify rather than assume.

### R9, index-path aliasing in the Python bindings

An index path addresses a position, not an object, so a handle held across a
structural mutation would silently read the wrong element.

*Mitigation*: the revision counter making it a loud `StaleElementError`, lazy
collections so the idiomatic loop never holds a stale handle, and stable ids in
v0.2.

### R10, the `rpptx-oxml` scope surface

PresentationML is enormous and there is no natural stopping point.

*Mitigation*: the raw-XML passthrough discipline. **If you do not render it, do
not model it.** Stated as a rule in `06-presentationml-model.md` and enforced in
review.

## Assumptions that would invalidate the plan if wrong

- **That a slide is a page.** The entire rendering reuse argument rests on it.
  Verified: `output.rs` is docx-free and `rdocx-pdf` depends on nothing else.
- **That `OpcPackage` reads a `.pptx` unmodified.** Verified through
  `main_document_part()` keying off `officeDocument`. Worth an actual test in M2
  rather than continued confidence.
- **That preset geometry is a data problem once the evaluator exists.** Verified
  by the official 187-definition ECMA-376 data set. The specification-text
  derivation remains available if the pinned source cannot be reproduced.
