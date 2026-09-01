# F-219, correctness, pass 5

**Reviewed**: the complete updated working-tree implementation and plan diff, nine files with 3,321 added lines and 34 removed lines, plus the pass-1 through pass-4 reviews
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, placeholder preflight misses SmartArt graphic frames and preserved choices
`crates/rpptx/src/lib.rs:3393`

The placeholder walk rejects `p:ph` only when it was projected onto a typed
`Shape` or `Picture`. A SmartArt `GraphicFrame` retains its `p:nvPr` as raw
non-visual children at `crates/rpptx-oxml/src/graphic_frame.rs:87`, so a
SmartArt frame carrying `<p:nvPr><p:ph idx="7"/></p:nvPr>` passes this check.
The same gap applies to a placeholder in a non-selected `mc:Choice`, because
the walk visits only the fallback at `crates/rpptx/src/lib.rs:3396`. Transfer
then retargets the slide to the selected destination layout without reconciling
that retained index, which silently severs placeholder inheritance. The
regression inserts only a directly typed ordinary shape at
`crates/rpptx/tests/integration.rs:546`, so it does not prove the public
placeholder-free contract for the SmartArt frame itself or preserved active
compatibility branches.

### D2, slide-owned images are not checked for unsupported dependencies
`crates/rpptx/src/lib.rs:3382`

The top-level preflight requires an image target to exist, but it recurses only
when the slide relationship is a diagram relationship. The relationship-free
image check at `crates/rpptx/src/lib.rs:3426` therefore applies only to images
reached from a diagram part. A slide image that owns an internal chart, OLE,
media, or custom relationship passes preflight. `duplicate_relationship_scope`
then sends only its bytes to the media store at `crates/rpptx/src/lib.rs:3481`,
silently dropping the owned graph or reusing a destination image whose owned
relationships differ. This contradicts the bounded fail-closed transfer
contract and its collision-safety guarantee.

### D3, a foreign `dgm` root binding can change typed text into foreign XML
`crates/rpptx-oxml/src/diagram.rs:1262`

Diagram text validation rejects writer conflicts only for `a` and `r`. An
aliased root such as `<q:t xmlns:q=".../diagram" xmlns:dgm="urn:producer">`
therefore passes as typed diagram text while retaining the foreign `dgm`
declaration. Dirty writing always creates `dgm:t` and then pushes every retained
root attribute at `crates/rpptx-oxml/src/diagram.rs:1325`, producing a root in
`urn:producer` rather than the diagram namespace. The saved part changes the
meaning of the edited node and may no longer reopen as editable SmartArt. The
new unsafe-shadow regression covers a nested foreign `a` binding at
`crates/rpptx-oxml/tests/integration.rs:239`, but not a safe root alias combined
with a foreign binding for the fixed output prefix.

### D4, an external slide-layout relationship bypasses the selected layout
`crates/rpptx/src/lib.rs:3466`

All external relationships preserve their original target before the
relationship-type mapping runs. A source slide with a `slideLayout`
relationship marked `TargetMode="External"` passes preflight because external
relationships are skipped at `crates/rpptx/src/lib.rs:3365`, then this branch
copies it unchanged instead of using `destination_layout_index`. The transfer
can consequently succeed with no destination-owned layout relationship, even
though the API requires the caller to select one. The existing load and commit
path accepts this package shape, so validation does not turn the silent bypass
into an atomic error.

### D5, an acyclic diagram chain can exhaust the process stack
`crates/rpptx/src/lib.rs:3439`

The visited set terminates cycles, but the preflight still recurses once per
distinct diagram part. A source package can supply an arbitrarily deep acyclic
chain using the five allowed diagram relationship types, and package input is
externally controlled. That chain can overflow the Rust stack before the
preflight returns. The copy phase repeats the same unbounded recursion at
`crates/rpptx/src/lib.rs:3579`, so even a depth that survives preflight gets a
second opportunity to abort the process. Cycle protection is correct, but an
iterative walk or an explicit checked graph bound is required for the panic
contract.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-4 diagram closure: diagram-owned relationships are now traversed recursively with source-part cycle protection. Nested unsupported relationship types and diagram-owned images with relationships reject before staging, while accepted diagram cycles copy to fresh destination parts without collision aliasing.
- Pass-4 direct placeholder remediation: ordinary typed shape and picture placeholders, including those in nested groups and the selected fallback, reject before staging and leave destination bytes unchanged. The uncovered placeholder forms are D1 above.
- Pass-4 text-root preservation: accepted unrelated attributes and safe namespace declarations on `dgm:t` survive dirty node text writing. Nested foreign DrawingML content and conflicting `a` or `r` bindings remain opaque and byte-exact. The remaining fixed `dgm` output-prefix conflict is D3 above.
- Prior correctness: inherited relationship namespace remap, complete data-model sequence enforcement, full XML root validation, producing-scope resolution, exact raw relationship remap, cached drawing-id remap, and slide, layout, and master inspection remain corrected.
- OOXML preservation: point and connection attributes, raw direct events, background, whole, extension-list, unsupported algorithm, style, colour, drawing, and safe text-root content remain retained in schema order.
- Contract: no additional unsupported mutation surface, dependency, feature, or binding API was introduced. The remaining bounded-transfer contract failures are D1, D2, and D4 above.
- Panics: no new unchecked index, arithmetic overflow, or malformed canonical-text slice was found beyond the graph-depth abort in D5.
- Structure: no unjustified trait, dynamic dispatch, dependency, feature, crate, wrapper, or integration test binary was added.
- Tests: `cargo test -p rpptx-oxml` passed 163 tests. `cargo test -p rpptx` passed 182 tests with 8 ignored. Both changed-crate checks, `cargo fmt --all --check`, and `git diff --check` passed. The worker progress record reports the full verification gate and pinned 50-deck corpus gate passed after the pass-4 remediation.
