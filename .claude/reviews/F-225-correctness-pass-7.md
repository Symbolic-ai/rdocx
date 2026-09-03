# F-225, correctness, pass 7

**Reviewed**: current working tree implementation across 17 feature files,
8,063 inserted lines and 27 deleted lines from `597a27c`
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, present malformed embedded-font objects are silently treated as absent

`crates/rpptx/src/pdf.rs:3220`

For a TrueType font, `embedded_fonts` converts every `FontDescriptor`
dereference or dictionary failure into `None` at lines 3220 to 3226. It does
the same for a present `FontFile2` that fails dereference or is not a stream at
lines 3228 to 3234. The font is then silently treated as unembedded. A font
without a `Widths` entry bypasses the only later descriptor validation through
the early return at `crates/rpptx/src/pdf.rs:3046`. For example, a TrueType
font with `/BaseFont /Carlito`, no `Widths`, and `/FontDescriptor 7` reaches
text shaping through the bundled Carlito. Because the resolved family still
equals the requested family, the substitution check at
`crates/rpptx/src/pdf.rs:1342` emits no diagnostic. Malformed present font
state is therefore neither rejected nor disclosed, and the source program can
be replaced silently. The embedded-font regression at
`crates/rpptx/src/pdf.rs:5500` covers only valid descriptors and streams. This
contradicts strict font parsing, malformed-state rejection, and explicit font
substitution at `.claude/plans/F-225-design.md:100`.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 6 disposition

- D1 is closed. `strict_page_content` accepts only absent content, one
  indirect stream, or an ordered array of indirect streams at
  `crates/rpptx/src/pdf.rs:2475`. It charges array work before reserving IDs,
  resolves bounded reference chains, accepts only raw and strict
  `FlateDecode`, rejects decode parameters and malformed filters, never falls
  back to raw compressed bytes, and charges decoded bytes plus separators
  against the aggregate limit. The regression at
  `crates/rpptx/src/pdf.rs:6299` covers order, both accepted forms, aggregate
  bytes, malformed members and targets, cycles, filters, and decode failure.
- D2 is closed. `strict_annotation_ids` charges the complete array and checks
  the remaining shape budget before allocating IDs at
  `crates/rpptx/src/pdf.rs:2638`. `strict_annotation` shares the bounded
  resolver for targets and actions, rejects duplicate terminal targets and
  malformed type, subtype, action, and URI state, preserves order, and never
  clones annotation dictionaries at `crates/rpptx/src/pdf.rs:2690`. The
  regression at `crates/rpptx/src/pdf.rs:6456` covers ordered links, malformed
  state, cycles, duplicates, work amplification, and the pre-allocation shape
  bound.

## Prior closure and full-diff audit

No additional findings were found in explicit text-position recovery, strict
page-tree order and ownership, page or resource cycle and duplicate handling,
page and resource depth and work bounds, shared reference caching, XObject
whole-table charging, Form traversal, content or annotation collection, text
encoding and source widths, affine geometry and page rotation, CTM stroke
semantics, dash lowering, image types and intrinsic-pixel accounting,
active-content scanning, retained-element accounting, URI restrictions, link
relationship ownership, OOXML child order, media publication, transactional
publication, public API shape, feature gating, or panic paths reachable from
imported bytes.

No lossy lopdf page, content, font-table, or annotation convenience call
remains. The defect above is the remaining local lossy extraction branch. No
other proportional allocation was found before its applicable input, object,
work, decompression, pixel, shape, or diagnostic cap.

The raw full-image luminance SSIM gate remains 0.995 and the isolated one-pixel
mutation fails it independently at `crates/rpptx/tests/integration.rs:511`.
The `render` feature retains the optional lopdf edge with `wasm_js` at
`crates/rpptx/Cargo.toml:25`. The diff still changes exactly the nine HLD files
named at `.claude/plans/F-225-design.md:185`, with no unapproved public
feature, trait, generic parameter, wrapper, builder, crate, or integration-test
binary.
