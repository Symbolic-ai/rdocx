# F-175, all, pass 4

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 1,949 additions and 4 deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, a boundary inside hidden revision content still splits the projection

`crates/rdocx/src/redaction.rs:475`

Known hidden revision containers no longer create a boundary themselves, but
the parser still applies every descendant boundary without considering the
hidden ancestor. For example, accepted text containing regular `sec`, a
deleted run with `<w:br/>`, and regular `ret` displays `secret` because the
entire deletion is absent. The break inside that hidden deletion flushes the
accepted flow anyway. The rejected projection includes the break and correctly
does not match, so the candidate commits with the selector in the accepted
view. The inverse failure occurs for a break inside an insertion hidden by the
rejected projection.

### D2, accepted and rejected passes do not share one fixed point

`crates/rdocx/src/redaction.rs:309`

`crates/rdocx/src/redaction.rs:311`

Each projection now reaches a local fixed point, but the accepted pass runs
only once before the rejected pass. A rejected-view removal can expose a new
accepted-view match. For selector `abc`, use an inserted `a`, deleted `a`,
regular `b`, deleted `c`, and inserted `bc` in that order. The initial accepted
view is `abbc`, while the rejected view is `abc`. The rejected pass removes its
match, including the regular `b`, and leaves the two insertion fragments. They
now form `abc` in the accepted view, which is not rerun. XML markup defeats the
raw byte scan, so sensitive accepted text remains after commit.

### D3, foreign revision lookalikes control Word projection visibility

`crates/rdocx/src/redaction.rs:575`

The hidden-ancestor check calls `word_revision_visibility` for every stack
element by local name without checking its namespace. A preserved producer
element such as `x:del` therefore hides descendant Word text in the accepted
projection and can make regular text on either side join and redact falsely.
The same problem applies to foreign `ins`, `moveFrom`, and `moveTo` elements.
Revision projection is an expanded-name decision and must require the Word
namespace.

### D4, visible run objects and references do not break Word text flow

`crates/rdocx/src/redaction.rs:629`

The semantic boundary allowlist covers breaks, tabs, symbols, and field
characters, but not other visible run content such as `w:drawing`, `w:object`,
`w:footnoteReference`, or `w:endnoteReference`. Text `sec` before one of those
items and `ret` after it is joined and removed as `secret`, even though the
rendered run contains an object or note marker between the fragments. The
public modeled helper was removed, but the package rewriter still needs the
same complete visible-content boundaries.

### D5, mutually exclusive markup-compatibility branches share one flow

`crates/rdocx/src/redaction.rs:656`

`mc:AlternateContent`, `mc:Choice`, and `mc:Fallback` are not boundaries for
Word or DrawingML flow. Run-level Choice text `sec` followed by Fallback text
`ret` is therefore combined and removed as `secret`, although a consumer
selects only one branch. Each selected branch must be redacted independently,
without forming matches against its mutually exclusive sibling.

### D6, malformed XML with multiple document roots passes validation

`crates/rdocx/src/redaction.rs:1083`

The rewritten-XML validator resolves namespaces but does not track whether
there is exactly one root element. quick-xml accepts a balanced sequence such
as two adjacent `w:document` roots, the redaction parser also finishes with an
empty stack, and the facade reopen scans for bodies without enforcing a single
root. A malformed sensitive part can therefore be rewritten and committed,
contrary to the fail-closed XML contract. Validation must enforce document
well-formedness in addition to balanced element names and bound prefixes.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-3 D1, direct hidden text: hidden insertion or deletion text is now
  projection-transparent when it contains no semantic boundary. Visible
  revision content still joins regular text, while an insertion and deletion
  do not form one false projected match. D1 covers the remaining descendant
  boundary case.
- Pass-3 D2: paired Word breaks and nonempty DrawingML breaks are now
  expanded-name semantic boundaries regardless of empty-element spelling.
- Pass-3 D3 within one projection or value: cross-node text, individual text
  values, CDATA, and sensitive attributes repeat removal to a fixed point. D2
  covers the remaining interaction between the two Word projections.
- Raw attribute preservation: zero findings. Matching attributes patch only
  their raw value spans, leaving tag whitespace, quote choice, and unrelated
  attributes byte-identical.
- Sensitive surface allowlists beyond D3: zero findings. Word, DrawingML,
  ChartML, SpreadsheetML, core-property, custom-property, and drawing
  nonvisual names are otherwise namespace-qualified.
- CDATA rewriting beyond D6: zero findings. Changed CDATA becomes escaped text
  and ordinary mismatched, unclosed, or unbound-prefix XML fails closed.
- OPC relationship resolution and bounds: zero findings. Targets resolve from
  their owning relationship source, external workbook targets and missing
  internal parts fail closed, and outer and nested packages use explicit
  limits.
- UTF-8 and UTF-16LE residual scanning: zero findings beyond the semantic XML
  gaps in D1, D2, and D5. Every inflated outer and nested entry is scanned in
  both encodings.
- Atomicity: zero findings. All changes remain on a complete staged clone until
  serialization, scan, bounded reopen, and validation succeed. Failure keeps
  package bytes, typed state, and all four cache or engine identities intact.
- Package preservation: zero findings. The regression compares every untouched
  outer part byte for byte, plus complete relationships and content types.
- Panic and error handling: zero findings. Production positions, slicing,
  indexing, and arithmetic are guarded or saturating.
- Public API isolation: zero findings. The additive method and report remain
  native to `rdocx`, with no Python, WASM, or CLI binding expansion.
- Structure: zero findings. The only new file is explicitly approved, and no
  new trait, generic parameter, crate, feature flag, forwarding wrapper, or
  dependency-family edge appears.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, and no sample or hash baseline file changes.
