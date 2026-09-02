# F-225, correctness, pass 5

**Reviewed**: current working tree implementation across 17 feature files,
6,714 inserted lines and 27 deleted lines from `597a27c`
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, isolated composite text does not recover at every explicit line-position operator

`crates/rpptx/src/pdf.rs:958`

An unsupported composite text show taints `text_position_supported` at
`crates/rpptx/src/pdf.rs:1268`. `BT` and `Tm` clear that taint, but `Td`, `TD`,
and `T*` replace the text matrix from the independently known line matrix at
lines 958 to 981 without clearing it. The quote operators perform the same
explicit line move and then immediately call `show_text` at
`crates/rpptx/src/pdf.rs:1070`. That call still returns at line 1272. A valid
sequence that selects a supported simple font after omitted Type0 text, then
uses `Td`, `TD`, `T*`, `'`, or `"` to establish the next position therefore
drops supported sibling text. The regression at `crates/rpptx/src/pdf.rs:5422`
proves recovery only through `Tm`, so the composite-state isolation remains
incomplete.

### D2, a cyclic page-tree subtree is silently omitted instead of rejected

`crates/rpptx/src/pdf.rs:406`

The importer obtains pages through lopdf's lossy `get_pages` map and accepts
any nonempty result at lines 406 to 420. That traversal ends when its internal
iteration allowance is exhausted rather than returning an error. A catalog
page tree containing one ordinary page followed by a cyclic `Pages` subtree
therefore yields the ordinary page, silently omits the cyclic subtree, and can
publish a partial presentation. The later inheritance check at
`crates/rpptx/src/pdf.rs:421` visits only page IDs already returned by the
lossy traversal. The cycle regression at `crates/rpptx/src/pdf.rs:5561`
instead makes one returned page's `Parent` point to itself, so it does not
exercise a cycle in `Kids`. This violates the required cyclic page-graph
rejection at `.claude/plans/F-225-design.md:100`.

### D3, font discovery bypasses the strict bounded resource resolver

`crates/rpptx/src/pdf.rs:2647`

Embedded-font collection calls lopdf's recursive `get_page_fonts` before any
page is parsed at lines 425 to 432. The same helper is called again for text
decoding at `crates/rpptx/src/pdf.rs:2385`. It recursively follows inherited
resources without this importer's `ResourceReferenceResolver` depth and work
charges, and it silently drops wrong-typed font tables and members. An
adversarial `Parent` chain can consequently consume recursive work or stack
before the shared bound applies. A malformed `/Font` resource can instead
disappear and become a missing-font diagnostic, rather than the required
malformed-state rejection. The later strict resource path protects XObjects,
but cannot repair work or type information already lost by these two font
helper calls. This contradicts the bounded and strict font/resource contract
at `.claude/plans/F-225-design.md:100`.

### D4, XObject-table allocation precedes the caller's resource-work limit

`crates/rpptx/src/pdf.rs:3041`

`xobject_entries` allocates a vector with capacity for the complete untrusted
XObject dictionary before charging even its first member. The per-entry work
charge starts only at line 3043. A caller can set a very small `max_objects`,
yet a large direct XObject table still causes allocation proportional to every
entry before the resolver rejects the table. The shared reference resolver
therefore bounds subsequent traversal but not this allocation, so the public
object/work limit does not fail before resource consumption.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 4 disposition

- D1 is closed. Text-only grouped shapes move the SVD scale into the resolved
  transform at `crates/rpptx-layout/src/context.rs:1140`, while shapes with a
  visible outline retain scaled bounds and invariant stroke width.
- D2 is closed. A valid ToUnicode stream takes ownership over an ordinary
  simple Encoding at `crates/rpptx/src/pdf.rs:2401` and is cached once.
- D3 is closed for `Tm`. Unsupported composite shows taint later relative text
  at `crates/rpptx/src/pdf.rs:1268`, and `Tm` explicitly restores position at
  line 956. The remaining explicit line-position operators are pass-5 D1.
- D4 is closed. `PageFont` retains `MissingWidth` and source widths are used
  for `Tj` and `TJ` advancement at `crates/rpptx/src/pdf.rs:206`.
- D5 is closed. DCT images are charged from intrinsic JPEG dimensions at
  `crates/rpptx/src/pdf.rs:1585`.
- D6 is closed. `ImageMask` and `BitsPerComponent` now reject wrong scalar
  types at `crates/rpptx/src/pdf.rs:3540`.
- D7 is closed for the active-content scan. The object cap now precedes that
  scan and one visited set plus an explicit work allowance bound it at
  `crates/rpptx/src/pdf.rs:348`.
- D8 is closed under the user-approved editable dash subset. Zero members,
  interior phases, and unrepresentable DrawingML stops isolate the stroke at
  `crates/rpptx/src/pdf.rs:839`.
- D9 is closed for distinct font programs. Concrete BaseFont aliases survive
  subset collection and shaping lookup at `crates/rpptx/src/pdf.rs:2679`.

## Not found

No additional findings were found in affine geometry and rotation, CTM stroke
semantics, source font widths, ToUnicode ownership and aggregate accounting,
JPEG intrinsic pixel charging, typed image state, active-content scanning,
retained-element accounting, rendering-mode taint, bounded source-string
deduplication, dash lowering, strict Filter parsing, URI scheme restriction,
link relationship ownership, OOXML child order, media relationship
publication, transactional candidate publication, native public API shape,
feature gating, or panic paths reachable from imported bytes.

The `render` feature activates the optional lopdf edge and its `wasm_js`
support at `crates/rpptx/Cargo.toml:25` without adding a public feature or a
binding API. The raw full-image SSIM floor remains 0.995, the unchanged source
passes, and the isolated one-pixel mutation fails the same predicate while
the other acceptance facts remain true at
`crates/rpptx/tests/integration.rs:511`. The final plan checklist and complete
verification evidence cover native and render WASM graphs, publish dry runs,
package ceilings, dependency policy, hash stability, rustdoc, prose, and skill
drift. The implementation changes exactly the nine HLD files named by the
approved plan and adds no unapproved trait, generic parameter, wrapper,
builder, feature flag, crate, or integration-test binary.
