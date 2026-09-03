# F-225, correctness, pass 6

**Reviewed**: current working tree implementation across 17 feature files,
7,470 inserted lines and 27 deleted lines from `597a27c`
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, page content loading remains lossy rather than strict

`crates/rpptx/src/pdf.rs:463`

The importer delegates the complete `Contents` boundary to lopdf's
`get_page_content_with_limit`. That helper silently omits wrong-typed array
members, missing targets, and targets that are not streams. It also falls back
to a stream's raw bytes when decoding fails for a reason other than the byte
limit. A page whose filtered stream contains syntactically valid PDF operators
as its raw payload can therefore be interpreted as if the unsupported or
malformed filter were absent. A malformed array member can instead disappear
while valid sibling streams are published. `Content::decode_strict` at
`crates/rpptx/src/pdf.rs:475` is strict only over the bytes that survive this
lossy collection and fallback, so it cannot restore the missing structure or
decoding failure. The malformed-input regression beginning at
`crates/rpptx/src/pdf.rs:6192` does not exercise malformed `Contents`, stream
targets, or filters. This violates the strict content-stream and malformed
state contract at `.claude/plans/F-225-design.md:100`.

### D2, annotation collection is lossy and allocates before a public bound

`crates/rpptx/src/pdf.rs:1703`

`parse_annotations` uses lopdf's `get_page_annotations`, which treats a
wrong-typed `Annots` value as absent and silently drops non-reference array
members, missing references, and targets that are not dictionaries. The
importer then clones every returned dictionary into a new vector at lines
1713 to 1715 before applying the retained-shape limit at line 1791. Repeating
one indirect annotation reference many times can therefore clone the same
large dictionary once per array entry while adding few PDF objects and no
retained shape. This bypasses the caller's `max_objects` and
`max_shapes_per_page` intent, while malformed annotation state can disappear
without rejection. Wrong-typed `Subtype` is also normalized into a skipped
non-link at `crates/rpptx/src/pdf.rs:1717`. The tests contain only one valid
annotation construction at `crates/rpptx/src/pdf.rs:4584` and no malformed or
adversarial annotation-list case. This violates the strict bounded annotation
contract at `.claude/plans/F-225-design.md:100`.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 5 disposition

- D1 is closed. `Td`, `TD`, and `T*` restore the known line position at
  `crates/rpptx/src/pdf.rs:958`, while both quote operators restore it before
  showing text at `crates/rpptx/src/pdf.rs:1072`. The regression re-taints and
  recovers each operator at `crates/rpptx/src/pdf.rs:5882`.
- D2 is closed. `strict_pages` performs an iterative ordered traversal with
  active and visited sets, exact Parent checks, strict `Type`, `Kids`, and
  `Count`, depth and work bounds, and early page limiting at
  `crates/rpptx/src/pdf.rs:2290`. No `get_pages` call remains.
- D3 is closed. Each page resolves its inherited Resources, Font table, and
  XObject graph through one bounded resolver at
  `crates/rpptx/src/pdf.rs:3022`. Font table types, indirect members, target
  dictionaries, reference cycles, depth, and work fail closed at
  `crates/rpptx/src/pdf.rs:3332`. No `get_page_fonts` call remains.
- D4 is closed. The complete XObject table length is charged before the
  capacity allocation at `crates/rpptx/src/pdf.rs:3476`, and the adversarial
  regression proves rejection below the table budget at
  `crates/rpptx/src/pdf.rs:5604`.

## Prior closure and full-diff audit

No additional findings were found in page order, Parent ownership, page-tree
cycle and duplicate detection, page and resource depth or work arithmetic,
resource-reference caching, Form traversal, Font and XObject table strictness,
text-state isolation, ToUnicode ownership and accounting, source font widths,
affine text and image geometry, page rotation, CTM stroke semantics, dash
lowering, image type and intrinsic-pixel checks, active-content scanning,
retained-element accounting, source-string deduplication, URI schemes, link
relationship ownership, OOXML child order, media publication, transactional
publication, public API shape, feature gating, or panic paths reachable from
imported bytes.

The raw full-image luminance SSIM gate remains 0.995 and the isolated one-pixel
mutation fails that same gate independently at
`crates/rpptx/tests/integration.rs:511`. The `render` feature retains the
private optional lopdf edge with `wasm_js` at `crates/rpptx/Cargo.toml:25`.
The implementation still changes exactly the nine HLD files named at
`.claude/plans/F-225-design.md:185`, with no unapproved public feature, trait,
generic parameter, wrapper, builder, crate, or integration-test binary.
