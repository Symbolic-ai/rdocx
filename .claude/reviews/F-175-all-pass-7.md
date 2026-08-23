# F-175, all, pass 7

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 2,334 additions and 4 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the UTF-16 path hides malformed XML declarations from validation

`crates/rdocx/src/redaction.rs:339`

The BOM-aware path removes any leading text that starts with `<?xml` through
the first `?>`, validates and rewrites only the remaining body, then prepends
the unchecked text. A UTF-16 story beginning with a declaration that omits the
required version, uses an invalid version or standalone value, or declares an
encoding inconsistent with its BOM can therefore be redacted and committed
with its malformed prolog intact. The declaration must be parsed and validated
before it is preserved outside the UTF-8 rewrite pass.

### D2, malformed lexical XML still passes the rewritten-part validator

`crates/rdocx/src/redaction.rs:1231`

The validator checks roots, depth, declaration and document-type placement,
and namespace bindings, but it does not enable quick-xml's comment checks or
validate character and entity rules in every text and attribute value. A
sensitive part can therefore contain a comment with an internal double
hyphen, a forbidden XML control character, or an undefined entity in an
unmodelled attribute, lose the selector elsewhere, and commit while remaining
malformed. The fail-closed contract requires complete XML well-formedness, not
only balanced expanded names.

### D3, core-properties formatting whitespace is treated as metadata

`crates/rdocx/src/redaction.rs:1113`

Every text event whose current element is in the core-properties namespace is
classified as sensitive, including indentation directly inside the
`cp:coreProperties` root. A non-empty selector equal to a unique newline and
tab indentation sequence removes that producer formatting, increments the
metadata report, and can commit even though no core property value matched.
The core-properties root must be excluded so only actual property elements
contribute sensitive text.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-6 D1: `w:tblPrExChange` now belongs to the namespace-qualified revision
  container set used by the author matcher. The entity-decoded author fixture
  proves that its exact raw value span is patched.
- Pass-6 D2: annotation references, page-number blocks, and all six legacy
  short and long date blocks now form Word text-flow boundaries. The table
  fixture covers each expanded name with paired producer syntax.
- Pass-6 D3 for valid documents: BOM-marked UTF-16LE and UTF-16BE parts are
  decoded, redacted through the shared semantic rewriter, and emitted with the
  original byte order and declaration. D1 covers malformed declarations hidden
  by that adapter.
- Word semantic grouping: zero additional findings. Accepted and rejected
  projections share a fixed point, hidden descendants remain transparent,
  revision branches stay isolated, and visible run content, ruby components,
  field instructions, and markup-compatibility alternatives form appropriate
  boundaries.
- Expanded names and raw preservation: zero findings beyond D3. Sensitive
  attributes remain namespace-qualified and patch only their raw value spans.
  Prefix aliases and foreign same-local-name producer content are covered.
- ChartML and SpreadsheetML: zero findings. DrawingML labels, string and
  numeric caches, shared strings, inline strings, and direct cell values are
  handled through the approved semantic flows and public numeric gate.
- OPC resolution and bounds: zero findings. Relationship targets resolve from
  their owning source, external workbook targets and missing internal parts
  fail closed, and outer and nested package opens use explicit limits.
- UTF-8 and UTF-16LE residual scanning: zero findings. Every inflated outer
  and nested entry is checked in both required encodings.
- Atomicity: zero findings. Mutation stays staged through serialization, scan,
  bounded reopen, and validation. Failure preserves package bytes, typed state,
  and all four cache or engine identities.
- Package preservation: zero findings. Every untouched outer part is compared
  byte for byte, together with complete relationships and content types.
- Panic and error handling: zero findings. Production positions, slicing,
  indexing, and arithmetic are guarded or saturating.
- Public API isolation: zero findings. The additive method and report remain
  native to `rdocx`, with no Python, WASM, or CLI binding expansion.
- Structure: zero findings. The sole new file is explicitly approved, and no
  new trait, generic parameter, crate, feature flag, forwarding wrapper, or
  dependency-family edge appears.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, and no sample or hash baseline file changes.
