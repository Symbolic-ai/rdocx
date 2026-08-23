# F-175, all, pass 3

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 1,902 additions and 4 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, hidden revisions incorrectly split a visible projection

`crates/rdocx/src/redaction.rs:629`

The accepted and rejected passes now join regular text to a visible revision,
but they flush the flow on both sides of every revision hidden in that
projection. For example, accepted Word text containing regular `sec`, a
deleted `x`, and regular `ret` displays `secret`. The accepted pass flushes at
the hidden deletion, while the rejected pass sees `secxret`, so neither pass
removes the selector. The raw scan sees XML markup between the surviving text
fragments and the candidate commits with sensitive text in the accepted view.
The inverse failure occurs in the rejected view around a hidden insertion.
Hidden revision content must disappear from its projection without becoming a
semantic text boundary.

### D2, semantic line breaks are boundaries only in self-closing syntax

`crates/rdocx/src/redaction.rs:632`

`crates/rdocx/src/redaction.rs:649`

Word and DrawingML breaks flush a flow only when quick-xml reports an
`Event::Empty`. The equivalent Word spelling `<w:br></w:br>` therefore does
not separate adjacent text. More importantly, the schema-valid DrawingML form
`<a:br><a:rPr/></a:br>` can never be an empty event. Text `sec` before either
form and `ret` after it is incorrectly joined and removed even though the
rendered content has a line break. Boundary behavior must depend on the
expanded-name semantics rather than the producer's empty-element spelling.

### D3, one cross-node removal can expose a selector that is never rescanned

`crates/rdocx/src/redaction.rs:707`

Each projected flow computes matches once, applies those removals, and returns.
Removing a match can join the surviving prefix and suffix into another exact
match across XML text nodes. For example, three DrawingML text nodes containing
`aa`, `bc`, and `bc` form `aabcbc`. Redacting `abc` removes the original match
and leaves `a` plus `bc`, which displays as `abc`. The later per-node pass does
not see a complete selector, and the raw byte scan sees markup between the
fragments, so the candidate commits with sensitive chart text still present.
Cross-node flows need a fixed-point removal or an equivalent final semantic
rescan.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-2 D1, visible revision joins: regular text now joins visible inserted
  content in the accepted projection and visible deleted content in the
  rejected projection. Mutually exclusive insertion and deletion branches do
  not form one false match. D1 describes the remaining hidden-revision case.
- Pass-2 D2: the modeled placeholder replacement is removed from the public
  path. Redaction now flushes the staged clone, rewrites the package, validates,
  scans, reopens, and commits without a second modeled text algorithm.
- Pass-2 D3, cache atomicity: the injected package serialization failure is
  reached after all four layout cache or retained-engine identities are primed.
  The unit assertion compares all four identities, package bytes, and typed
  paragraph state after failure.
- Pass-2 D3, byte preservation: the round-trip regression compares the exact
  bytes of every outer package part not listed as an edited sensitive part. It
  also compares the complete relationship sets and content types.
- Expanded-name and raw attribute handling: zero additional findings. Text and
  attribute allowlists remain namespace-qualified, and matching attributes
  patch only their raw value spans.
- XML and CDATA validity: zero additional findings. Changed CDATA becomes
  escaped text, and rewritten XML is reparsed before acceptance.
- OPC integrity and bounds: zero findings. Relationship targets are resolved
  relative to the owning source, external targets and missing parts fail
  closed, and outer and nested packages use explicit read limits.
- UTF-8 and UTF-16LE scanning: zero findings beyond the semantic cross-node
  gaps in D1 and D3. Every inflated outer and nested entry is checked in both
  encodings.
- Panic and error handling: zero findings. Production parser positions,
  slicing, indexing, and arithmetic are guarded or saturating.
- Public API isolation: zero findings. The additive method and report remain
  native to `rdocx`, with no Python, WASM, or CLI binding expansion.
- Structure: zero findings. The only new file is explicitly approved, and no
  new trait, generic parameter, crate, feature flag, forwarding wrapper, or
  dependency-family edge appears.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, and no sample or hash baseline file changes.
