# F-219, correctness, pass 7

**Reviewed**: claim-base `be090606d44d021c1e2ba82da52aff3aff086e10` through final implementation `481d1245ba5a52691b156ff88923d118d5330aee`, 10 files and 3,894 changed lines (3,860 additions, 34 deletions), plus the six prior reviews
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, cached drawing shape counts still accept same-namespace opaque lookalikes
`crates/rpptx-oxml/src/diagram.rs:634`

`CT_DiagramDrawing::from_xml` derives `shape_count` with the unrestricted descendant scan in `count_descendants` at `crates/rpptx-oxml/src/diagram.rs:1151`. That helper counts every expanded-name `dsp:sp` anywhere below the part root. A same-namespace `<dsp:sp>` under a root extension or another opaque wrapper therefore increases the public cached-drawing projection even though it is outside the schema-owned `dsp:spTree` shape path. The pass-6 remediation correctly constrained the cached drawing relationship id, layout, style, and colour projections, but did not constrain this second cached-drawing fact. The focused schema-position regression at `crates/rpptx-oxml/tests/integration.rs:267` omits `CT_DiagramDrawing`, while the existing drawing test uses only a foreign-namespace lookalike. The approved plan requires cached-drawing projections, not only the cached relationship id, to ignore same-namespace lookalikes outside schema positions.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-6 remediation**: All four prior fixes are present. Checked node edits and transfer validate the data, layout, quick-style, colour, and present cached-drawing relationship roles against exact internal types. Transfer requires exactly one internal source slide-layout relationship. Checked data-model edits reject duplicate point and connection model ids before mutation. Layout, style, colour, and cached relationship-id extraction and remapping follow schema-owned paths. The remaining cached drawing shape-count path is D1.
- **Transfer and graph closure**: No additional issue was found in the bounded `transfer_smartart_slide_from` contract, placeholder scanning across typed and preserved compatibility forms, slide-owned or diagram-owned image relationship rejection, nested unsupported dependency rejection, fresh part allocation, image-byte deduplication, diagram cycles, or the shared 128-part preflight and copy ceiling.
- **Duplicate and transfer remapping**: The four `dgm:relIds` roles and the schema-owned unqualified cached drawing id remap in their producing scopes. Copied diagram cycles reuse allocated targets, destination-visible collisions allocate fresh parts, and unrelated raw relationship-like content remains unchanged.
- **Correctness and public API**: No additional contract issue was found in slide, layout, and master scope resolution, optional cached drawings, supported node text editing, point ordering, layout families, style labels, colour labels, or concrete `DiagramPart<T>` resource states.
- **Panics and atomicity**: No reachable unchecked index, arithmetic overflow, unbounded recursion, malformed canonical-text slice, or partial publication was found. Checked failures occur before publication, and staged operations reopen before committing.
- **OOXML and preservation**: Apart from D1, namespace aliases, fixed-prefix shadows, complete-root validation, schema child order, safe text-root attributes, ordered raw events, relationship namespace remapping, and byte-exact untouched diagram parts showed no issue.
- **Tests**: The focused SmartArt selections passed 9 `rpptx-oxml` tests and 13 `rpptx` tests. Full `cargo test -p rpptx-oxml --quiet` passed 15 unit and 151 integration tests. Full `cargo test -p rpptx --quiet` passed 26 unit and 161 integration tests with 8 ignored. `git diff --check` passed. The isolated worktree has no corpus directory, so the external 50-deck cases reported their designed skip. The only sensitivity gap found is described in D1.
- **Structure**: The approved diagram module and five concrete `DiagramPart<T>` instantiations remain justified. No unapproved trait, dynamic dispatch, dependency, feature, crate, wrapper, or integration binary was added.
