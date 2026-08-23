# F-175, all, pass 11

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 2,809 additions and 4 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-10 D1: zero findings. Core and custom metadata, ChartML cache `c:v`,
  and direct SpreadsheetML `v` now enter the cross-event sensitive-value flow
  at `crates/rdocx/src/redaction.rs:752`. Their expanded-name start and end
  boundaries at `crates/rdocx/src/redaction.rs:833` keep adjacent scalar values
  isolated while Text, CDATA, and GeneralRef events within one value are
  matched together.
- Entity-split remediation tests: zero findings. Focused cases cover core and
  custom metadata at `crates/rdocx/src/redaction.rs:2110`, ChartML cache values
  at `crates/rdocx/src/redaction.rs:2182`, and direct SpreadsheetML values at
  `crates/rdocx/src/redaction.rs:2195`.
- Word revision projections: zero findings. The accepted and rejected passes
  still share the fixed-point loop at `crates/rdocx/src/redaction.rs:388`.
  Hidden revision descendants remain projection-transparent, mutually
  exclusive content cannot join, and visible run content, fields, ruby
  branches, and markup compatibility alternatives retain their boundaries.
- Sensitive-surface allowlists and preservation: zero findings. Expanded-name
  comparisons use normalized namespace URIs, exact raw attribute value spans
  are patched, and unaffected XML byte ranges remain unchanged.
- XML structure and declarations: zero findings. XML 1.1 and document type
  declarations fail closed. Names, namespace bindings, duplicate attributes,
  entities, line endings, CDATA, comments, processing instructions,
  declarations, and exactly one root are validated before commit.
- ChartML and SpreadsheetML: zero findings. DrawingML labels, string and
  numeric caches, shared strings, inline strings, and direct cell values remain
  within the approved semantic and relationship-resolved package boundaries.
- OPC resolution and bounds: zero findings. Targets resolve from their owning
  relationship scope, duplicate relationship ids and missing internal targets
  fail closed, and outer and nested packages use explicit limits.
- Residual scanning: zero findings. Every inflated outer and nested entry is
  scanned for the required raw UTF-8 and UTF-16LE forms.
- Atomicity and cache preservation: zero findings. Mutation remains staged
  through serialization, scan, bounded reopen, and validation. Failure keeps
  package bytes, typed state, and all four cache or engine identities.
- Package preservation: zero findings. Tests compare every untouched package
  part byte for byte and preserve complete relationship and content-type
  collections.
- Panic and error handling: zero findings. Production positions, slices,
  indexes, counts, and depth arithmetic are guarded or saturating within their
  package bounds.
- Public API isolation and structure: zero findings. The additive native method
  and report do not expand Python, WASM, or CLI bindings. The sole new file is
  approved, and no new trait, generic parameter, crate, feature flag, wrapper,
  or dependency-family edge appears.
- Tests and checks: zero findings. The 6 focused library tests, 3 focused
  regression tests, `cargo check -p rdocx --all-targets`,
  `cargo fmt --all -- --check`, and `git diff --check` pass.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, with no sample or hash-baseline change.
