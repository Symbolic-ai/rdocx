# F-175, all, pass 5

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 2,108 additions and 4 deletions
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, two Word field-code forms are outside the sensitive allowlists

`crates/rdocx/src/redaction.rs:603`

`crates/rdocx/src/redaction.rs:1097`

The Word text flow recognizes `w:instrText` but not the revision form
`w:delInstrText`, and the sensitive attribute matcher does not include
`w:fldSimple/@w:instr`. A selector split across two deleted field-code nodes
survives the rejected projection because the raw scan sees intervening markup.
A simple-field instruction such as `w:instr="sec&#114;et"` also survives because
its decoded value is never inspected and its raw bytes do not contain
`secret`. Both are recoverable Word field instructions and must follow the
same redaction rules as complex `w:instrText` content.

### D2, misplaced XML declarations still pass malformed-input validation

`crates/rdocx/src/redaction.rs:1205`

The exactly-one-root validation now rejects adjacent roots and non-whitespace
text outside the root, but it ignores every other top-level event. quick-xml
emits an XML declaration after the root as `Event::Decl`, so input such as a
valid `w:document` followed by `<?xml version="1.0"?>` reaches EOF with one root
and depth zero. The rewrite and facade reopen also ignore that event, allowing
the malformed sensitive part to commit. Declaration and document-type
placement must be validated as part of the fail-closed XML contract.

### D3, footnote and endnote reference marks still do not break text flow

`crates/rdocx/src/redaction.rs:673`

The expanded visible-content list includes body anchors
`w:footnoteReference` and `w:endnoteReference`, but omits the distinct
`w:footnoteRef` and `w:endnoteRef` marks rendered inside the note stories.
Text `sec` before either automatically numbered mark and `ret` after it is
still joined and removed as `secret`, although the displayed note contains a
number between the fragments.

### D4, author-bearing range revisions are omitted

`crates/rdocx/src/redaction.rs:726`

The revision allowlist used by the author matcher omits range-start revision
elements such as `w:moveFromRangeStart`, `w:moveToRangeStart`, and the custom
XML insertion, deletion, and move range starts. These schema elements carry a
`w:author` attribute. An entity-spelled author such as
`w:author="sec&#114;et"` bypasses the raw scan and commits with the decoded
sensitive author intact. Revision author matching needs the complete set of
author-bearing expanded names without turning zero-width range markers into
text boundaries.

### D5, ruby guide text and base text are joined as one linear value

`crates/rdocx/src/redaction.rs:693`

`w:ruby` is a boundary only at the outside of the phonetic-guide construct.
Its `w:rt` guide text and `w:rubyBase` base text remain in the same
cross-node flow. Guide text `sec` above base text `ret` is therefore removed as
`secret`, even though the values are distinct parallel renderings rather than
one contiguous string. The two ruby components need independent flows.

### D6, the mandatory namespace-alias regression is still absent

`crates/rdocx/src/redaction.rs:1357`

The expanded-name unit fixture uses the canonical `w`, `wp`, and `pic`
prefixes for every sensitive Word and drawing name. The workbook fixture uses
a default namespace, but no test binds an alternate prefix to a sensitive Word,
DrawingML, or ChartML namespace. A regression that hardcodes the canonical
prefixes can therefore keep the named gate green. The parser risk route and
the approved test plan require an explicit prefix-alias case.

### D7, the chart regression never selects a numeric cache value

`crates/rdocx/tests/regression_test.rs:5540`

The authored chart contains numeric values `12.5` and `19.0`, but the test
redacts `secret`, which occurs only in category and series-name strings. The
private ChartML fixture likewise contains only a `c:strCache` match. Removing
`c:numCache` handling would leave both named gates green, despite the design
and HLD requiring numeric cache values and their workbook cells to redact in
the same atomic operation.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-4 D1: boundary events nested inside a revision hidden by the current
  projection are ignored, so the hidden subtree is transparent to surrounding
  visible text.
- Pass-4 D2: accepted and rejected Word projections now alternate until a
  complete pair of passes makes no replacements. The pass-4 exposure example
  reaches a shared fixed point.
- Pass-4 D3: revision visibility now requires the Word transitional or strict
  namespace. Foreign same-local-name producer elements do not control a
  projection.
- Pass-4 D4 for its cited examples: drawings, objects, body footnote and
  endnote references, comments, pictures, content parts, ruby constructs, and
  controls now form semantic boundaries. D3 and D5 describe remaining
  distinctions within note marks and ruby content.
- Pass-4 D5: expanded-name `mc:AlternateContent`, `mc:Choice`, and
  `mc:Fallback` boundaries isolate mutually exclusive branches for Word,
  ChartML, and workbook flows.
- Pass-4 D6 for multiple roots: rewritten XML now tracks depth and root count,
  rejects zero or multiple roots, and rejects text or CDATA outside the root.
  D2 covers the remaining declaration-placement case.
- Fixed-point removal beyond D1: zero findings. Individual text, CDATA,
  attribute, cross-node, and paired Word projection passes strictly remove a
  non-empty selector until none remains in their modeled value.
- Raw attribute preservation: zero findings. Matching attributes patch only
  their raw value spans, leaving tag whitespace, quote choice, and unrelated
  attributes byte-identical.
- OPC relationship resolution and bounds: zero findings. Targets resolve from
  their owning source, external workbook targets and missing internal parts
  fail closed, and outer and nested packages use explicit read limits.
- UTF-8 and UTF-16LE residual scanning: zero findings beyond the semantic and
  entity-spelling gaps in D1 and D4. Every inflated outer and nested entry is
  scanned in both encodings.
- Atomicity: zero findings. Mutation remains staged through serialization,
  scan, bounded reopen, and validation. Failure preserves package bytes, typed
  state, and all four cache or engine identities.
- Package preservation: zero findings. Every untouched outer part is compared
  byte for byte, together with complete relationships and content types.
- Panic and error handling: zero findings. Production positions, slicing,
  indexing, and arithmetic are guarded or saturating.
- Public API isolation: zero findings. The additive method and report remain
  native to `rdocx`, with no Python, WASM, or CLI binding expansion.
- Structure: zero findings. The only new file is explicitly approved, and no
  new trait, generic parameter, crate, feature flag, forwarding wrapper, or
  dependency-family edge appears.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, and no sample or hash baseline file changes.
