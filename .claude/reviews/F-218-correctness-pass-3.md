# F-218, correctness, pass 3

**Reviewed**: claim base and source HEAD
`424969e8be199f8618d3d7558299b71633cf5582`, the complete working-tree delta,
the approved plan, cited HLD sections, scratch record, and correctness passes 1
and 2. The implementation delta is 6 files with 3,222 additions and 8
deletions, including the untracked `crates/rpptx/src/embedded.rs`.

**Verdict**: 2 defects, 0 smells, 0 nitpicks. Not ready.

## Defects

### D1, distinct OLE relationships in one compatibility frame acquire overlapping ownership

The scanner appends every schema-positioned `p:oleObj` relationship found under
one `p:graphicFrame`, including separate `mc:Choice` and `mc:Fallback` paths, to
the same frame accumulator at `crates/rpptx/src/embedded.rs:1542` and
`crates/rpptx/src/embedded.rs:1581`. At frame close, it emits a separate
reference for every id but assigns each one the whole identical frame range at
`crates/rpptx/src/embedded.rs:1595`. Inventory consequently inserts distinct
ids as distinct executable objects at `crates/rpptx/src/embedded.rs:785` and
`crates/rpptx/src/embedded.rs:863`, even though both claim the same owning XML
node.

Removal of either reported object removes that shared whole-frame range at
`crates/rpptx/src/embedded.rs:483`, then deletes only the selected relationship
and its newly unreachable target at `crates/rpptx/src/embedded.rs:381`. The
other relationship and payload remain in the package without their owning XML.
This contradicts the approved single-logical-payload and exact-coordinate
contract at `.claude/plans/F-218-design.md:110` and the requirement to leave
unrelated payloads untouched at `.claude/plans/F-218-design.md:118`. A frame
whose compatibility paths resolve to different ids must fail closed before
inventory or mutation. The producing-scope regression covers ordinary
single-id frames at `crates/rpptx/tests/integration.rs:88`, but no test uses
different ids in compatibility paths of one frame.

### D2, a signature origin may own unrelated relationships that removal silently discards

The package signature graph validates relationship-id uniqueness, then filters
the origin relset down to digital-signature relationships at
`crates/rpptx/src/embedded.rs:1224` and
`crates/rpptx/src/embedded.rs:1231`. It does not reject an internal or external
relationship of another type in that relset. The resulting graph records only
signature targets at `crates/rpptx/src/embedded.rs:1248`. Signature removal
then deletes the complete origin relationship set and origin part at
`crates/rpptx/src/embedded.rs:1353`, without deleting or preserving the
unrelated target through a valid owner. Thus a remove-invalidated-signatures
mutation can discard an unrelated ownership edge despite the contract that it
deletes only corresponding signature infrastructure at
`.claude/plans/F-218-design.md:125`.

The malformed-topology table exercises external, traversal, missing,
duplicate-origin, and misplaced signature relationship cases at
`crates/rpptx/tests/integration.rs:885`, but it has no origin relset containing
an unrelated relationship. The origin relset must be constrained to its legal
signature relationships, or the implementation must otherwise prove that
removing it cannot affect unrelated package ownership.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass-2 D1, producing scopes**: inventory now traverses every loaded layout
  and each presentation-listed master, validates the master root, and mutation
  updates the appropriate typed or raw owner. The source-built and tracked
  corpus tests cover layout and master inventory and removal at
  `crates/rpptx/tests/integration.rs:88` and
  `crates/rpptx/tests/integration.rs:174`.
- **Pass-2 D2, persisted VBA invalidation**: the private project-relset marker
  survives serialization and fresh reopen in both default and all-feature
  builds. It adds no public API, feature, crate, or reverse dependency.
- **Pass-2 D3, persisted package invalidation**: the private package marker is
  staged through ordinary and embedded commits, and both feature configurations
  recognize it after reopen. Its topology and target are validated before use.
- **Pass-2 D4, singleton signature topology**: package-root origin cardinality,
  misplaced origin and signature relationships, safe targets, required parts,
  duplicate signature targets, and signature relset id uniqueness are checked.
  D2 above is the remaining adjacent origin-relset ownership gap.
- **Pass-2 D5, complete VBA relset identity**: duplicate ids are rejected over
  the full project relationship set before signature classification, so a
  non-signature decoy cannot alias a signature id.
- **Atomicity**: public mutations operate on a consolidated staged clone and
  publish only after serialization, reopen, and validation. The two defects
  above concern accepted ownership graphs, not a partial commit after an error.
- **OOXML namespace and order**: OLE and ActiveX discovery uses expanded-name
  relationship attributes and complete schema-owner paths. Strict aliases,
  fixed-prefix shadows, foreign descendants, and retained producer XML are
  covered. No additional namespace declaration loss or schema-order defect was
  found.
- **Panics and bounds**: the XML walkers use checked ancestry movement and
  return errors for malformed structure and duplicate semantic attributes. No
  reachable untrusted-input panic, integer overflow, unchecked depth decrement,
  or invalid slice access was found.
- **Public API and structure**: the three enums, information value, and four
  facade methods match the approved additive native Rust contract. No Python,
  WASM, or CLI surface, trait, generic, wrapper, feature, crate, dynamic
  dispatch, binary fixture, or integration binary was added. Dependency
  direction remains valid.
- **Focused validation**: `cargo test -p rpptx --all-features --test integration
  embedded_ -- --nocapture` passed 7 tests. The all-feature signature filter
  passed 11 tests. The producing-scope filter passed 2 tests, including the
  tracked corpus case. The default-feature fresh-reopen package invalidation
  test passed. `cargo test -p oxml-opc relationship` passed 6 tests.
  `cargo check -p rpptx --all-targets` passed with the three pre-existing F-221
  test-constant warnings. `cargo fmt --all --check` and `git diff --check`
  passed. These green paths do not exercise D1 or D2.
- **Differential testing**: the approved plan assigns no external application
  oracle to this opaque package-graph API, so no missing oracle was counted as
  a finding.
