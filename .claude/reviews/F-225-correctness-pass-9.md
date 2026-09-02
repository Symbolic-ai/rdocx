# F-225, correctness, pass 9

**Reviewed**: current working tree implementation across 17 feature files,
8,654 inserted lines and 27 deleted lines from `597a27c`
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, malformed action contexts bypass active-content rejection

`crates/rpptx/src/pdf.rs:2207`

`ActiveContentScanner::scan_dictionary` converts a missing or wrong-typed
action `S` entry into `None`, then rejects only when `is_some_and` sees a
well-typed non-URI name at line 2215. The generic scanner also performs no
action-context type check for scalar or array objects at
`crates/rpptx/src/pdf.rs:2161`. A catalog `/OpenAction 7`, an action dictionary
with `/S 7`, an empty action dictionary, or a non-URI destination array
therefore survives preflight and is ignored by the remaining importer. Only a
dictionary whose `S` successfully parses as a non-URI name is rejected. The
regression at `crates/rpptx/src/pdf.rs:7578` changes an existing well-typed URI
action to the well-typed name `GoTo`, so it cannot expose these malformed or
non-dictionary action forms. This violates the required non-URI action and
malformed-state rejection at `.claude/plans/F-225-design.md:100`.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 8 disposition

- D1 is closed. `FontEncodingCaches` owns ToUnicode maps by terminal stream ID
  and ordinary encodings by font ID at `crates/rpptx/src/pdf.rs:235`.
  `page_fonts` checks the stream cache before parsing and inserts one shared
  `Arc` only on a unique terminal miss at `crates/rpptx/src/pdf.rs:2978`.
  Embedded preload still decompresses and charges that terminal stream once at
  `crates/rpptx/src/pdf.rs:3315`, so retained bytes, parse work, and owned map
  memory are linear in unique streams across fonts and pages. The 16-font
  adversarial regression asserts one retained stream, cache entry, parse, and
  mapped result for every alias at `crates/rpptx/src/pdf.rs:5698`. Its sibling
  cases preserve two parses for two streams and per-font ordinary caches.

## Amplification and lossy-helper audit

No additional unbounded repeated parse or cache amplification was found.
Content streams, ToUnicode maps, embedded font programs, source widths,
resource chains, XObject tables, annotations, images, source strings, and
retained elements are deduplicated or charged before proportional work and
allocation. No lopdf page, resource-table, content, font-table, or annotation
convenience loader remains. The active-action type normalization above is the
one remaining lossy untrusted-input boundary found in this pass.

## Prior closure and full-diff audit

No additional findings were found in strict page-tree order and ownership,
resource inheritance and caching, content forms, filters, decoding and
aggregate bytes, annotation targets, actions, duplicates and order, explicit
text-position recovery, font field strictness and substitution, source font
widths, affine geometry and page rotation, CTM stroke semantics, dash
lowering, image types and intrinsic-pixel accounting, retained-element
accounting, URI restrictions, link relationship ownership, OOXML child order,
media publication, transactional publication, public API shape, feature
gating, or panic paths reachable from imported bytes.

The raw full-image luminance SSIM gate remains 0.995 and the isolated one-pixel
mutation fails it independently at `crates/rpptx/tests/integration.rs:511`.
The `render` feature retains the optional lopdf edge with `wasm_js` at
`crates/rpptx/Cargo.toml:25`. The diff still changes exactly the nine HLD files
named at `.claude/plans/F-225-design.md:185`, with no unapproved public
feature, trait, generic parameter, wrapper, builder, crate, or integration-test
binary.
