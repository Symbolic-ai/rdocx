# F-175, all, pass 10

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 2,753 additions and 4 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, entity references split scalar sensitive values into unmatched events

`crates/rdocx/src/redaction.rs:501`

The single-event rewrite treats each `Text`, `CData`, and `GeneralRef` event as
an independent value. The cross-node pass excludes core and custom properties
entirely at `crates/rdocx/src/redaction.rs:417`, and its ChartML and
SpreadsheetML flows cover DrawingML `a:t` and rich-text `t` only, not scalar
`c:v` or direct cell `v`. A sensitive value such as
`<vt:lpwstr>sec&#114;et</vt:lpwstr>` is therefore processed as three values,
none of which matches selector `secret`. The same failure applies to core
property text, ChartML cache `c:v`, and direct SpreadsheetML `v`, including
text and CDATA splits. The raw residual scan does not see the decoded selector,
so the operation can commit a package that still exposes it semantically.
Sensitive scalar element content must be grouped and normalized as one logical
value before matching.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-9 D1: text, CDATA, and cross-node flows now use XML-version-aware
  content decoding, and the focused tests cover raw carriage returns in all
  three paths.
- Pass-9 D2 for namespace bindings and duplicate expanded names: namespace URIs
  are entity-normalized before allowlist and duplicate-name comparison. D1
  covers the remaining GeneralRef grouping gap on scalar sensitive values.
- Pass-9 D3: XML 1.1 declarations now fail closed before mutation or commit,
  including UTF-16 input.
- Word semantic grouping: zero findings. Accepted and rejected projections
  share a fixed point, hidden revision descendants remain transparent, and
  visible run content, fields, ruby branches, and markup compatibility
  alternatives retain the required boundaries.
- Sensitive attributes and preservation: zero findings. Expanded-name
  allowlists use normalized namespace URIs, exact raw value spans are patched,
  and unrelated producer bytes remain unchanged.
- XML structure and declarations: zero findings. Document type declarations
  and XML 1.1 fail closed. Comments, names, namespace bindings, duplicate
  attributes, character references, CDATA, processing instructions,
  declarations, and exactly one root are otherwise validated before commit.
- ChartML and SpreadsheetML: zero additional findings beyond D1. DrawingML
  labels, string and numeric caches, shared strings, inline strings, and direct
  cell values remain within the approved allowlists and package gates.
- OPC resolution and bounds: zero findings. Targets resolve from their owning
  source, duplicate relationship ids and missing internal targets fail closed,
  and outer and nested packages use explicit limits.
- Residual scanning: zero additional findings beyond D1. Every inflated outer
  and nested entry is scanned for the required raw UTF-8 and UTF-16LE forms.
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
  `cargo check -p rdocx --all-targets` pass. D1 lacks entity-split scalar
  fixtures for custom and core properties, ChartML cache values, and direct
  SpreadsheetML cell values.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, with no sample or hash-baseline change.
