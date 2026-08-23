# F-175, all, pass 9

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 2,654 additions and 4 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, XML end-of-line normalization can leave a semantic selector behind

`crates/rdocx/src/redaction.rs:473`

Sensitive text and CDATA are decoded without applying XML end-of-line
normalization, and the cross-node pass does the same at
`crates/rdocx/src/redaction.rs:600`. In XML 1.0 a raw carriage return is
semantically a line feed. A compact sensitive element containing raw
`a\rb` therefore does not match selector `a\nb`, and the byte scan does not
find that selector either. The operation can commit with a zero count even
though an XML consumer exposes the exact selector. Matching must use the
document version's normalized XML content for text and CDATA in both passes.

### D2, entity-normalized namespace bindings bypass expanded-name matching

`crates/rdocx/src/redaction.rs:1573`

`resolved_namespace` copies quick-xml's raw, non-normalized namespace bytes.
An XML namespace declaration may legally spell part of its URI with a numeric
character reference. For example, binding `w` to the Word URI with the `i` in
`main` written as `&#105;` makes valid `w:t` and `w:author` producers invisible
to every sensitive-surface allowlist. If the value spells `secret` as
`sec&#114;et`, the residual byte scan also misses it and the operation commits
the recoverable value. The same raw comparison lets two namespace spellings of
one URI evade the duplicate expanded-name attribute check.

### D3, raw XML 1.1 restricted characters are accepted as well formed

`crates/rdocx/src/redaction.rs:1339`

The XML 1.1 character arm accepts every scalar from U+0001 through U+D7FF.
XML 1.1 permits restricted controls such as U+0001 only through character
references, not as literal characters. A sensitive part can contain a raw
restricted control in an unmodelled attribute, lose the selector elsewhere,
and commit malformed XML. Validation must distinguish literal characters from
character references or reject XML 1.1 declarations for OOXML parts.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-8 D1: all document type declarations now fail closed before a candidate
  can commit. The root-mismatch fixture directly covers the prior example.
- Pass-8 D2 for ordinary namespace bindings: start and empty elements share one
  validator that rejects duplicate attribute expanded names under different
  literal prefixes. D2 covers the remaining entity-normalized namespace case.
- Pass-8 D3: every package and part relationship scope gets an independent id
  set, so duplicate ids fail before target validation or commit.
- Pass-8 D4: every public report field now accumulates through the shared
  saturating helper, and the boundary test starts at `usize::MAX`.
- Word semantic grouping: zero additional findings. Accepted and rejected
  projections share a fixed point, hidden revision descendants remain
  transparent, and visible run content, fields, ruby branches, and markup
  compatibility alternatives retain the required boundaries.
- Sensitive-surface allowlists and preservation: zero additional findings
  beyond D1 and D2. Ordinary prefix aliases use expanded names, sensitive
  attributes patch exact raw value spans, and unrelated producer bytes remain
  unchanged.
- XML structure and declarations: zero additional findings beyond D3. Comments,
  element and attribute names, namespace bindings, predefined entities, CDATA,
  processing instructions, declarations, and exactly one root are otherwise
  validated before commit.
- ChartML and SpreadsheetML: zero findings. DrawingML labels, string and numeric
  caches, shared strings, inline strings, and direct cell values use the
  approved semantic flows and internal relationship-resolved workbook boundary.
- OPC resolution and bounds: zero findings. Targets resolve from their owning
  source, duplicate relationship ids and missing internal targets fail closed,
  and outer and nested packages use explicit limits.
- Residual scanning: zero additional findings beyond D1 and D2. Every inflated
  outer and nested entry is scanned for the required raw UTF-8 and UTF-16LE
  forms.
- Atomicity and cache preservation: zero findings. Mutation remains staged
  through serialization, scan, bounded reopen, and validation. Failure keeps
  package bytes, typed state, and all four cache or engine identities.
- Package preservation: zero findings. Tests compare every untouched part byte
  for byte and preserve complete relationship and content-type collections.
- Panic and error handling: zero findings. Production positions, slices,
  indexes, counts, and depth arithmetic are guarded or saturating.
- Public API isolation and structure: zero findings. The additive native method
  and report do not expand Python, WASM, or CLI bindings. The sole new file is
  approved, and no new trait, generic parameter, crate, feature flag, wrapper,
  or dependency-family edge appears.
- Tests: the 6 focused library tests, 3 focused regression tests, and
  `cargo check -p rdocx --all-targets` pass. D1 through D3 lack semantic XML
  normalization fixtures that would enforce these contract edges.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, with no sample or hash-baseline change.
