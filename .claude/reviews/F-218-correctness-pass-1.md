# F-218, correctness, pass 1

**Reviewed**: claim base `424969e8be199f8618d3d7558299b71633cf5582`
through the complete working tree, including the untracked implementation. The
delta is 6 files and 1,845 changed lines, with 1,840 additions and 5 deletions.
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, Agile VBA project signatures are not part of the signature graph
`crates/rpptx/src/embedded.rs:684`

VBA signature classification accepts only the legacy
`vbaProjectSignature` relationship. A VBA project whose attached signature uses
the standardized `vbaProjectSignatureAgile` relationship is therefore reported
as unsigned. Preserve policy does not associate that evidence with the project,
and remove policy leaves it behind. This violates the contract to follow any
project-signature relationship and can report `Absent` for a signed executable
payload.

### D2, transactional ordinary edits can leave an invalid signature reported as present
`crates/rpptx/src/lib.rs:1791`

The ordinary mutation commit path copies the staged invalidation booleans but
never sets them when a retained package signature exists and the staged package
changed. For example, remove a slide from a signed deck while an embedded object
remains on another slide. The commit serializes and reopens the changed package,
so the later byte comparison sees the changed bytes as its new baseline, while
`package_signatures_invalidated` is still false. Inventory then reaches the
`Present` branch at `crates/rpptx/src/embedded.rs:517` even though the retained
signature evidence no longer authenticates the package. A fresh
`Presentation::from_bytes` also resets both state fields at
`crates/rpptx/src/lib.rs:870`, producing the same false validity after save and
reopen.

### D3, package signature presence does not validate the internal signature graph
`crates/rpptx/src/embedded.rs:717`

`has_package_signature` treats the existence of a relationship type as complete
signature evidence. It does not reject an external origin, an unsafe target, a
missing origin part, a missing origin relationship set, or a missing signature
part. Any such package causes every embedded item to be reported as `Present`.
The remove policy can then discard the origin relationship without diagnosing
the malformed or external graph. Signature classification and mutation must
follow the same normalized internal graph and fail-closed rules as the embedded
payload itself.

### D4, duplicate relationship ids are resolved by file order
`crates/rpptx/src/embedded.rs:571`

`required_relationship` delegates to `get_by_id`, which returns the first match
and does not reject duplicate ids. A relationship set can therefore contain two
entries with the XML-owned id, one safe and one external, wrong-type, or aimed
at another part. Inventory and extraction use whichever appears first. Removal
then removes every entry with that id at
`crates/rpptx/src/embedded.rs:739`, so its effect is not even limited to the
relationship that inventory selected. The source-part and relationship-id pair
is the public identity, so ambiguous identities must reject before inspection
or mutation.

### D5, duplicate semantic relationship attributes are resolved by attribute order
`crates/rpptx/src/embedded.rs:1088`

The attribute reader returns immediately on the first `id` in either accepted
relationships namespace. An `ax:ocx`, `p:control`, or `p:oleObj` carrying two
accepted aliases, or one Transitional and one Strict relationship attribute,
is therefore interpreted according to lexical attribute order. This permits an
ambiguous XML owner to select a safe relationship while retaining a second
foreign identity in the opaque bytes. Duplicate semantic relationship
attributes must fail closed rather than select the first value.

### D6, nested same-namespace lookalikes can acquire executable ownership
`crates/rpptx/src/embedded.rs:982`

OLE ownership finds the last matching `a:graphicData` anywhere below a
`p:graphicFrame` and validates only the descendants between that node and
`p:oleObj`. It does not require the complete schema-owned
`p:graphicFrame/a:graphic/a:graphicData` path. A foreign extension inside an
otherwise unrelated graphic frame can therefore nest its own same-namespace
`a:graphicData` and `p:oleObj`, causing inventory to expose it and removal to
delete the enclosing real graphic frame. The ActiveX path has the adjacent
problem at `crates/rpptx/src/embedded.rs:1021`: any immediate `p:control` below
any descendant `p:controls` is accepted without proving the schema-owned slide
or presentation position. Both scanners must keep extension and opaque
lookalikes outside executable ownership.

## Smells

None.

## Nitpicks

None.

## Not found

- Panics: no untrusted-input panic was found. The one `expect` follows a
  successful lookup of the same slide owner, and the byte-range removal checks
  every boundary before slicing.
- Arithmetic and depth safety: no overflow or unchecked depth decrement was
  found in the iterative XML scanners.
- Atomicity: apart from the ambiguity cases above, replacement and removal are
  staged on a cloned presentation, serialized, reparsed, validated, and only
  then committed.
- Public API: the exported enums, owned info value, identity arguments, and
  four facade methods match the approved additive pre-1.0 Rust contract. No
  Python, WASM, or CLI surface was added.
- Structure and dependencies: the approved private module isolates the graph
  logic without adding a trait, generic, wrapper, feature, crate, or integration
  binary. `cargo tree -p rpptx -e normal` showed no reversed format dependency,
  and `no_shared_crate_depends_on_a_format_crate` passed.
- Focused validation: `cargo check -p rpptx --all-targets`, the six named F-218
  tests, `cargo test -p oxml-opc relationship`, the dependency-direction test,
  and `git diff --check` passed. The focused embedded filter also passed its
  adjacent media tests. These green paths do not cover D1 through D6.
- Differential testing: the approved test plan does not assign a Python,
  LibreOffice, or Microsoft Office oracle to this opaque package-graph API, so
  no missing differential oracle was counted as a finding.
