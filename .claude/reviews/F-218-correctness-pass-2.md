# F-218, correctness, pass 2

**Reviewed**: claim base `424969e8be199f8618d3d7558299b71633cf5582`
through the complete working tree, including the untracked implementation and
pass-1 review. The implementation delta is 6 files and 2,523 changed lines,
with 2,518 additions and 5 deletions.
**Verdict**: 5 defects, 0 smells, 0 nitpicks. Not ready.

## Defects

### D1, OLE inventory excludes layout and master graphic frames
`crates/rpptx/src/embedded.rs:170`

OLE discovery iterates only `self.slides`. It never scans the already resolved
slide layouts or the presentation's slide masters, even though both own
`p:spTree` values and can carry the same schema-owned OLE graphic frame. This
is not hypothetical corpus breadth. The tracked `alterman_security.pptx` has
two OLE relationships and matching frames in `slideLayout1.xml`, plus another
in `slideMaster1.xml`. `embedded_content` omits all three, and the generic
source-part identity therefore cannot extract, replace, or remove them. The
source-built inventory test at `crates/rpptx/tests/integration.rs:9` exercises
only a slide and does not consume the approved tracked OLE corpus case.

### D2, preserved VBA signature invalidation is lost on fresh reopen
`crates/rpptx/src/lib.rs:872`

`from_package` always resets `embedded_invalidated_signatures` to an empty set.
For a package with an attached VBA project signature and no package signature,
replacing the VBA payload with `PreserveInvalidatedSignatures` reports
`Invalidated` only on the live owner. Saving and reopening retains the original
VBA signature bytes, resets the identity set, observes the attached
relationship again, and reports `Present`. The policy contract requires that
preserved evidence remain invalidated after mutation. The policy regression at
`crates/rpptx/tests/integration.rs:372` checks the live result and retained
parts but never reopens this VBA-only path.

### D3, default-feature reopen still reports content-invalid package signatures as present
`crates/rpptx/src/embedded.rs:526`

Without `digital-signatures`, reopen classifies a retained package signature as
invalid only when a manifest reference or selected relationship disappeared.
A normal shape or text edit changes a signed part while keeping every part and
relationship reference present, so `signature_manifest_has_missing_reference`
returns false and inventory reports `Present`. The default-feature regression
at `crates/rpptx/tests/integration.rs:281` proves only the removed-reference
case. The real digest-invalid case at
`crates/rpptx/tests/integration.rs:247` is compiled only with the optional
signature verifier. The default public inventory API can therefore still
report stale preserved signature evidence as present after save and reopen.

### D4, package signature validation accepts illegal origin topology
`crates/rpptx/src/embedded.rs:949`

`package_signature_graph` follows every package-level origin and each origin's
signature children, but it does not require the OPC singleton origin or reject
signature-typed relationships outside those positions. A package with two
different origin targets is accepted. A package with one accepted origin plus
a digital-signature relationship from an ordinary part is also accepted, and
remove policy deletes only the graph it happened to traverse. The package
contract rejects duplicate and misplaced signature infrastructure before
mutation. The malformed-graph regression at
`crates/rpptx/tests/integration.rs:630` covers external, traversal, and missing
nodes but not origin cardinality or misplaced signature relationships.

### D5, a VBA signature id duplicated by a non-signature relationship is accepted
`crates/rpptx/src/embedded.rs:914`

Attached VBA signature discovery counts only the two signature relationship
types and never validates unique ids across the complete VBA project
relationship set. One signature relationship and one unrelated relationship
with the same id therefore appear to be one valid attached signature.
Inspection succeeds, and remove policy strips the signature relationship while
leaving the colliding relationship behind. This is the same ambiguous OPC
identity that pass 1 required to fail before inspection or mutation. The new
duplicate-id regression at `crates/rpptx/tests/integration.rs:707` challenges
only the slide payload relationship set.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-1 D1, Agile VBA signatures**: both legacy and Agile relationship types
  enter the same attached-signature classification and both explicit mutation
  policies. No remaining Agile-only branch was found.
- **Pass-1 D2, live transactional package invalidation**: the ordinary
  `commit_candidate` path now carries the staged invalidation state, and the
  all-feature real-signature regression proves the live and verifier-backed
  reopen path. D2 and D3 are the two remaining durability gaps outside that
  covered path.
- **Pass-1 D3, basic package signature graph safety**: external, traversal,
  missing-origin, missing-origin-relationship, missing-signature, and duplicate
  traversed relationship identities fail before embedded mutation. D4 is the
  remaining graph-topology gap.
- **Pass-1 D4, selected payload identity**: OLE, ActiveX, VBA, and package
  relationships selected through a public source-part and id now require one
  exact relationship. D5 is confined to the adjacent VBA signature relset.
- **Pass-1 D5, semantic XML attributes**: Transitional and Strict aliases of
  the relationship `id` expanded name are counted together. A second semantic
  attribute fails instead of winning by lexical order.
- **Pass-1 D6, slide schema ownership**: OLE and control discovery now requires
  complete namespace-aware schema positions. Foreign extensions and nested
  same-namespace lookalikes do not acquire ownership. D1 is the omitted
  producing-root scope, not a recurrence of descendant matching.
- **Atomicity and ownership-aware deletion**: supported replacement and removal
  stage on a consolidated clone, serialize, reopen, validate, and publish only
  the candidate. Shared payloads and unrelated producer orphans remain intact
  in the covered slide, ActiveX, and VBA paths. No additional partial-commit
  path was found.
- **OOXML preservation and schema order**: replacement retains the owning XML
  and relationship identity. Removal edits only the captured owner range and
  reparses through the typed root. Namespace aliases, Strict relationship
  namespaces, fixed-prefix shadows, and retained producer children remain
  covered without a new source-byte rewrite finding.
- **Panics and bounds**: no reachable untrusted-input panic, unchecked slice,
  depth underflow, or arithmetic overflow was found. The one `expect` follows
  resolution of the same slide owner before mutable lookup.
- **Public API**: the three enums, owned information value, four facade methods,
  normalized identity arguments, and SHA-256 value match the approved native
  Rust contract. The impact remains additive for the published pre-1.0
  `rpptx` and `oxml-opc` crates. No Python, WASM, or CLI surface was added.
- **Structure and dependency direction**: the approved private module adds no
  trait, generic, wrapper, feature, crate, dynamic dispatch, binary fixture, or
  integration binary. `cargo tree -p rpptx -e normal` retained the required
  layering, and `no_shared_crate_depends_on_a_format_crate` passed.
- **Focused validation**: `cargo test -p oxml-opc relationship` passed 6 tests.
  The all-feature embedded filter passed 7 tests, the signature filter passed
  10 tests, and the duplicate-identity and nested-lookalike regressions each
  passed independently. `cargo check -p rpptx --all-targets` passed with three
  pre-existing F-221 test-constant warnings. `git diff --check` passed. These
  green paths do not exercise D1 through D5.
