# F-175, all, pass 8

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 2,645 additions and 4 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, malformed document type declarations still pass XML validation

`crates/rdocx/src/redaction.rs:1437`

The `DocType` arm checks placement and decoded characters but never validates
the document type grammar or compares its declared root name with the actual
root. Quick-xml deliberately skips DTD contents rather than validating them.
For example, a sensitive part with `<!DOCTYPE wrong>` followed by a
`w:document` root can lose the selector and commit even though the XML
well-formedness constraint requires those names to match. The fail-closed
validator must either reject document type declarations or validate their
complete grammar and root constraint.

### D2, duplicate expanded-name attributes still pass XML validation

`crates/rdocx/src/redaction.rs:1457`

The attribute loop validates each qualified name and resolves each namespace
independently, but it never rejects two different prefixes that resolve to the
same attribute expanded name. An element can therefore bind `w` and `z` to the
Word namespace, carry both `w:author` and `z:author`, lose the selector from one
value, and commit namespace-invalid XML. Quick-xml's attribute iterator rejects
duplicate raw qualified names only. The expanded-name contract requires a
per-element set of resolved attribute names for both start and empty elements.

### D3, package validation does not reject duplicate relationship ids

`crates/rdocx/src/redaction.rs:1630`

Relationship validation checks external mode and internal target existence,
but it does not require ids to be unique within a relationship scope. The OPC
parser retains duplicate ids, so a document containing two `rId1` entries can
be redacted, serialized, reopened, and committed as a relationship-invalid
candidate. This contradicts the plan and HLD requirement to commit only a
relationship-valid package.

### D4, embedded-workbook replacement counts can overflow on 32-bit targets

`crates/rdocx/src/redaction.rs:201`

Each bounded nested workbook returns a safe individual count, but the public
report combines workbook counts with unchecked `+=`. On a 32-bit target, 65
relationship-reachable, highly compressible workbooks whose worksheet XML is
near the 64 MiB per-part limit can collectively exceed `usize::MAX` while the
outer archive remains within its own inflated-byte and entry limits. Debug
builds panic and release builds wrap the public count. The accumulator must use
checked or saturating arithmetic consistently with `RedactionReport::total`.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-7 D1: BOM-marked UTF-16 declarations are now parsed separately and
  checked for version, attribute order, standalone value, and endian-consistent
  encoding before their unchanged bytes are restored. Valid UTF-16LE and
  UTF-16BE fixtures pass, while the malformed and inconsistent fixtures fail.
- Pass-7 D2 checks: comment and end-name checks are enabled, and rewritten XML
  now validates qualified names, characters, predefined entities, CDATA
  placement, processing instructions, declaration placement, and exactly one
  root. D1 and D2 identify the remaining well-formedness gaps.
- Pass-7 D3: formatting text directly inside `cp:coreProperties` is excluded
  from the sensitive metadata surface, while actual `cp`, `dc`, and `dcterms`
  property values remain included.
- Word semantic grouping: zero additional findings. Accepted and rejected
  projections share a fixed point, hidden revision descendants remain
  transparent, and visible run content, fields, ruby branches, and markup
  compatibility alternatives retain their required boundaries.
- Sensitive-surface matching and preservation: zero additional findings beyond
  D2. Element and attribute allowlists use expanded names, sensitive attribute
  edits patch exact raw value spans, and unrelated producer bytes remain
  unchanged.
- ChartML and SpreadsheetML: zero findings. DrawingML labels, string and numeric
  caches, shared strings, inline strings, and direct cell values use the
  approved flows and relationship-resolved internal workbook boundary.
- OPC traversal and bounds: zero additional findings beyond D3 and D4. Targets
  resolve from their owning source, external workbooks and missing internal
  parts fail closed, and both outer and nested packages have explicit limits.
- Residual scanning: zero findings. Every inflated outer and nested entry is
  scanned for the required UTF-8 and UTF-16LE byte forms.
- Atomicity and cache preservation: zero findings. Mutation remains staged
  through serialization, scan, bounded reopen, and validation. Failure keeps
  package bytes, typed state, and all four cache or engine identities.
- Package preservation: zero findings. Tests compare every untouched part byte
  for byte and preserve complete relationship and content-type collections.
- Panic and error handling: zero additional findings beyond D4. Production
  positions, slices, indexes, depth arithmetic, and XML edits otherwise return
  errors or use guarded arithmetic.
- Public API isolation and structure: zero findings. The additive native method
  and report do not expand Python, WASM, or CLI bindings. The sole new file is
  approved, and no new trait, generic parameter, crate, feature flag, wrapper,
  or dependency-family edge appears.
- Tests: the 6 focused library tests, 3 focused regression tests, and
  `cargo check -p rdocx --all-targets` pass. D1 through D4 lack adversarial
  coverage that would enforce their corresponding contract edges.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, with no sample or hash-baseline change.
