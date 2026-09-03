# F-225, correctness, pass 8

**Reviewed**: current working tree implementation across 17 feature files,
8,397 inserted lines and 27 deleted lines from `597a27c`
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a shared ToUnicode stream is parsed and retained once per font object

`crates/rpptx/src/pdf.rs:2971`

The encoding cache is keyed by the concrete font object ID. On a miss,
`page_fonts` reparses the referenced ToUnicode stream into an owned Unicode
map at lines 2989 to 3008, then inserts that allocation under the font ID at
line 3026. `embedded_fonts` correctly decompresses and charges a shared stream
only once by stream ID at `crates/rpptx/src/pdf.rs:3273`, and normalizes every
referencing font back to that same terminal stream at line 3372. Distinct valid
font dictionaries can therefore reference one large ToUnicode stream, yet the
map is parsed and retained once for every font. With `N` fonts and an `M`-byte
map, input, object work, and aggregate retained bytes are proportional to
`N + M`, while parse work and owned map memory are proportional to `N * M`.
No aggregate charge covers those repeated parses or allocations. The cache
regression at `crates/rpptx/src/pdf.rs:5561` repeats shows from one font, and
the multi-font regression at `crates/rpptx/src/pdf.rs:5827` gives each font no
shared ToUnicode map. Neither exposes this valid shared-object amplification.
This violates the bounded font-parser contract at
`.claude/plans/F-225-design.md:100`.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 7 disposition

- D1 is closed. Present BaseFont, Encoding, ToUnicode, Widths,
  FontDescriptor, FontFile2, and MissingWidth values are distinguished from
  absence and strictly type-checked beginning at
  `crates/rpptx/src/pdf.rs:3214`. One shared bounded resolver rejects missing
  targets, cycles, depth, and work. ToUnicode and FontFile2 streams use the
  strict raw-or-Flate decoder at `crates/rpptx/src/pdf.rs:2581`. Width arrays
  are charged before allocation and cached by font identity at
  `crates/rpptx/src/pdf.rs:3063`. An absent embedded program now always reports
  substitution, including Carlito to Carlito, at
  `crates/rpptx/src/pdf.rs:1357`. The regression beginning at
  `crates/rpptx/src/pdf.rs:5647` covers absent optionals and malformed scalar,
  target, cycle, stream, filter, CMap, width, and descriptor cases.

## Lossy-conversion and allocation audit

No additional finding was found in remaining `.ok()`, `unwrap_or`, default,
or `filter_map` sites. The single-byte lookup is total for its fixed 256-entry
cache, Unicode-map decoding supplies its defined replacement behavior, and
the text-string collector is only a bounded preload whose operands are later
validated by the interpreter. Font field defaults now occur only after strict
prevalidation. Image omissions occur through explicit unsupported-state
diagnostics rather than malformed supported-state normalization.

No lopdf page, content, font-table, or annotation convenience loader remains.
No other proportional allocation was found before its applicable input,
object, graph-work, decompression, pixel, shape, or diagnostic limit.

## Prior closure and full-diff audit

No additional findings were found in strict page-tree and inherited-resource
handling, content forms and order, content filters and aggregate bytes,
annotation targets, actions, duplicates and order, explicit text-position
recovery, source font widths, affine geometry and page rotation, CTM stroke
semantics, dash lowering, image types and intrinsic-pixel accounting,
active-content scanning, retained-element accounting, URI restrictions, link
relationship ownership, OOXML child order, media publication, transactional
publication, public API shape, feature gating, or panic paths reachable from
imported bytes.

The raw full-image luminance SSIM gate remains 0.995 and the isolated one-pixel
mutation fails it independently at `crates/rpptx/tests/integration.rs:511`.
The `render` feature retains the optional lopdf edge with `wasm_js` at
`crates/rpptx/Cargo.toml:25`. The diff still changes exactly the nine HLD files
named at `.claude/plans/F-225-design.md:185`, with no unapproved public
feature, trait, generic parameter, wrapper, builder, crate, or integration-test
binary.
