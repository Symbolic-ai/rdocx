# F-219, correctness, pass 8

**Reviewed**: claim-base `be090606d44d021c1e2ba82da52aff3aff086e10` through final implementation `44c5325ae388c3b2a1ad6d269d288194abda01f1`, 10 files and 3,868 changed lines (3,834 additions, 34 deletions), plus the seven prior reviews
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-7 remediation**: `CT_DiagramDrawing::from_xml` now counts only direct expanded-name `dsp:sp` children of direct expanded-name `dsp:spTree` children at `crates/rpptx-oxml/src/diagram.rs:632`. Alias prefixes resolve correctly, namespace shadows fail the URI checks, and same-namespace shapes at the root, below opaque children, or inside extension content remain unprojected. The parser retains the complete original drawing bytes at `crates/rpptx-oxml/src/diagram.rs:644`.
- **Relationship roles and scopes**: Checked node edits and transfer validate all four frame relationship ids against their exact internal diagram types. A present schema-owned cached drawing id resolves in the same slide, layout, or master producing scope to the Microsoft diagram-drawing type. Missing, external, wrong-type, and absent-target roles fail without mutation.
- **Schema-position projection**: Layout title, category, algorithm, and constraint facts, style and colour labels, cached drawing ids, and cached drawing shapes ignore foreign and same-namespace lookalikes outside their owned paths. Cached drawing remapping changes only the schema-owned unqualified `relId`.
- **Transfer and graph closure**: The bounded `transfer_smartart_slide_from` path requires exactly one internal source layout, rejects placeholders in typed and preserved compatibility forms, rejects relationship-owning slide and diagram images, and rejects unsupported nested dependencies. Cycles terminate, the shared 128-part ceiling bounds preflight and copy, fresh allocation avoids destination part collisions, and image bytes deduplicate without diagram-part aliasing.
- **Duplicates and remapping**: Checked text and point-order edits reject duplicate point or connection model ids before dirtying the model. Duplication and transfer remap all four `dgm:relIds` roles and the schema-owned cached drawing id while retaining unrelated relationship-like bytes.
- **Atomicity and panics**: Public checked operations stage package changes and reopen before publication. No reachable unchecked index, arithmetic overflow, unbounded recursion, malformed canonical-text slice, or partial-publication path was found.
- **OOXML and preservation**: Root validation reaches EOF, data-model known children retain schema order, namespace aliases and fixed-prefix shadows fail closed, and ordered raw events, safe root attributes, opaque text, unsupported algorithms, and untouched diagram parts remain byte-preserved.
- **Public API and structure**: The native pre-1.0 API remains additive and concrete. The approved diagram module and five concrete `DiagramPart<T>` instantiations are justified. No unapproved trait, dynamic dispatch, dependency, feature, crate, wrapper, or integration binary was added.
- **Tests**: The focused cached-drawing schema-position regression passed. Full `cargo test -p rpptx-oxml --quiet` passed 15 unit and 151 integration tests. Full `cargo test -p rpptx --quiet` passed 26 unit and 161 integration tests with 8 ignored. `git diff --check` passed. The isolated worktree has no corpus directory, while the progress record documents the pinned 50-deck corpus and full verification gates from the implementation run.
