# F-175, all, pass 2

**Reviewed**: the remediated working tree on `work/f-175-codex`, 7 tracked
feature files plus the approved new `crates/rdocx/src/redaction.rs`, 1,795
additions and 4 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, matches spanning a revision boundary still survive

`crates/rdocx/src/redaction.rs:631`

Every revision container is now an unconditional text-flow boundary. That
prevents the pass-1 false match between mutually exclusive insertion and
deletion branches, but it also prevents valid matches in either projected
view. For example, a regular run containing `sec` followed by an insertion
containing `ret` displays `secret` in the accepted view. The boundary flushes
the two fragments separately, the raw scan sees markup between them, and the
candidate commits with the sensitive accepted text intact. The equivalent
regular-text plus deletion case survives in the rejected view. Redaction needs
accepted and rejected flow projections whose matching edits are combined, not
one boundary around every tracked container.

### D2, the modeled pass still redacts across non-text run content

`crates/rdocx/src/document.rs:3049`

The public operation first delegates body, header, footer, comment, and
footnote work to the general placeholder replacement helper. That helper
concatenates only `RunContent::Text` and ignores intervening hard breaks,
drawings, note references, and other visible run content. It therefore removes
`secret` from `sec`, a hard break, and `ret` before the new raw XML boundary
logic runs. The focused pass-2 tests call the private XML rewriter directly,
so their hard-break assertion does not exercise this public path. Either the
modeled traversal must use the same semantic flows or the staged flush should
be followed solely by the validated package rewriter and reopen.

### D3, the atomic and round-trip test contract remains incomplete

`crates/rdocx/tests/regression_test.rs:5551`

The strengthened helper primes and compares only the ordinary `layout()`
result. It does not prime or compare the deterministic cache or the bundled
fallback retained engine, although the design and HLD require layout caches in
the plural to survive every failure. None of the failure cases induces the
serialization failure named in the approved test plan. The round-trip test at
`crates/rdocx/tests/regression_test.rs:5674` also leaves normal untouched parts,
such as styles, out of its byte comparison. It checks selected relationships,
content types, two edited raw parts, and main-document order, so an unrelated
part rewrite can still keep the named gate green.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1: split DrawingML rich text, shared strings, and inline strings now
  use surface-specific cross-node flows. Focused coverage includes split chart
  and workbook values.
- Pass-1 D2: modeled property-change revisions and the other enumerated Word
  revision containers now redact their namespace-qualified author attribute.
- Pass-1 D3: drawing non-visual attributes are gated by their
  WordprocessingDrawing, picture, WordprocessingShape, or WordprocessingGroup
  expanded names. Foreign `docPr` and `cNvPr` lookalikes remain unchanged.
- Pass-1 D4: only the raw value span of a matching attribute is patched. Tag
  whitespace, quote choice, unrelated attributes, and their entity spellings
  remain byte-identical.
- Pass-1 D5 within one branch: instruction text, displayed text, self-closing
  hard breaks, nested paragraphs, and mutually exclusive revision containers
  no longer share one flow. D1 and D2 describe the remaining projection and
  public-path failures.
- Pass-1 D6: changed CDATA is emitted as escaped ordinary text and every
  rewritten sensitive XML part is reparsed before acceptance.
- OPC integrity and bounds: zero findings. Targets are resolved relative to
  their owning relationship scope. External chart and workbook relationships,
  missing internal targets, absent content types, oversized parts and entry
  sets, and nested ZIP depth beyond one fail closed.
- Atomic implementation structure beyond D3: zero findings. All typed and
  package mutation occurs on a complete staged clone, and live state changes
  only after serialization, scan, bounded reopen, and relationship validation.
- Panic handling: zero findings. New production indexing, slicing, and
  arithmetic are guarded by parser positions, match ranges, collection checks,
  or saturating totals.
- Public API isolation: zero findings. The additive method and report remain
  native to `rdocx`. Python, WASM, and CLI wrappers do not gain a redaction
  surface.
- Structure: zero findings. The sole new module is explicitly approved. No new
  trait, generic parameter, crate, feature flag, forwarding wrapper, or
  dependency-family edge appears.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, and no sample or hash baseline file changes.
