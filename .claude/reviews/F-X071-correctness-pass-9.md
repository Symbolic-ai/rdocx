# F-X071, correctness, pass 9

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final source `5b68cbf8f191031d0231a8d3e08358025c912f79`, 22 implementation and test files plus the approved-plan contract and 4,223 changed lines (4,035 additions, 188 deletions), together with untracked correctness passes 1 through 8
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, mixed inherited and redundant-local uses change marker cardinality
`crates/rdocx/src/document.rs:943`
`crates/rdocx/src/document.rs:1017`
`crates/rdocx/src/document.rs:1044`
`crates/rdocx/src/document.rs:1183`

Owner capture calls `namespace_owner_markers` without redundant local bindings,
while candidate replay calls it with those bindings enabled. This correctly
normalizes one inherited raw marker after the OXML model makes it self-contained,
but it also promotes every pre-existing redundant-local use into a new owner
marker. For example, a paragraph with `xmlns:x="urn:target"` can contain both
`<x:a/>`, which depends on the owner, and
`<x:b xmlns:x="urn:target"/>`, which does not. Capture records only `x:a`.
After parsing makes the first raw child self-contained, candidate scanning
records both `x:a` and `x:b`, strips both matching declarations, and then fails
the marker multiset's exact cardinality check. The valid owner is rejected even
though its dependent raw child and namespace are unchanged. The same problem
applies to nested owner scopes and attributes with redundant local bindings.
The decoy regression at `crates/rdocx/tests/regression_test.rs:2925` proves that
a separate owner with a redundant binding is treated as ambiguous. It does not
cover dependent and independent uses mixed within one legitimate owner.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-8 D1 remediation**: Existing nested owner order is valid again. The
  legacy horizontal-rule round trip and the insert, remove, and reorder owner
  replay regression pass. Word and XML identities remain exact while pending
  foreign declarations no longer invalidate an outer structural snapshot.
- **Pass-8 D2 remediation**: Different-URI and same-URI self-contained decoys
  both fail closed. The semantic-structural fallback was removed, redundant
  owner declarations are stripped only from a marker's root start tag, and
  namespace facts still distinguish foreign URI shadows. The remaining mixed
  cardinality failure is D1.
- **Lexical marker normalization**: Outside D1, the parser accepts single and
  double quotes plus XML whitespace around attribute assignments, removes only
  exact owner declaration name and decoded-value pairs, and retains every other
  attribute, quote, spacing, raw subtree, marker count, and multiset distinction.
  Expanded markers preserve nested declarations and fixed-prefix shadows.
- **Numbering namespace validation**: The complete retained subtree is checked
  with nested scope push and pop. Undeclared root and descendant element or
  attribute prefixes reject before mutation. Declared aliases, default
  namespaces, local shadows, and fresh-prefix collisions remain safe.
- **Numbering preservation and OOXML**: Unchanged raw leaves replay
  byte-for-byte. Duplicate expanded-name Word values remain untyped and
  canonicalize safely on mutation. Metadata duplicates and other raw XML retain
  their schema boundaries and ordering. No new drawing, document-background,
  row-property, table, or repeated-save regression was found.
- **Public API contract**: The approved plan and HLD 10 still distinguish the
  additive native `rdocx` facade from intentional low-level pre-1.0
  `rdocx-oxml` exhaustive-literal breaks. Commit `5b68cbf` adds no public
  signature or binding change.
- **Panics and bounds**: No new panic, unchecked arithmetic, excessive revision
  recursion, malformed revision acceptance, or depth-bound regression was
  found.
- **Tests**: `cargo test -p rdocx-oxml --quiet` passed 334 unit tests and one
  doctest. `cargo test -p rdocx --lib --quiet` passed 326 tests with three
  ignored. The `rdocx` regression binary passed 178 tests with one ignored.
  Workspace all-target, all-feature Clippy with warnings denied, prose, and both
  diff checks passed. The missing mixed-binding sensitivity is D1.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, dynamic dispatch, or unnecessary indirection was found.
