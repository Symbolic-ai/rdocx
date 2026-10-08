# F-X188, all, pass 1

**Reviewed**: Frozen working diff against claim Base `ce6d46baec4af822c802d55f20cb9e83c1a6528e`, 18 tracked files, 2175 insertions and 79 deletions. F-X188 is in progress. The approved design, progress checkpoint and cited HLD contracts were read before the complete implementation, binding, regression and documentation diff.

**Verdict**: 1 defect, 0 smells, 0 nitpicks.

## Defects

### D1, Forward glossary omission examines unselected notes

`crates/rdocx/src/field.rs:3190`

The forward glossary transfer iterates every related footnote and endnote part
and passes the complete part to `omit_comment_markers` at line3212. It does not
restrict this scan to the recursive note owners referenced by the selected
fragment. Fragment capture replaces only the selected main-body content and
then writes the retained package, so unselected notes remain present in that
package (`crates/rdocx/src/document.rs:1206` and
`crates/rdocx/src/document.rs:1224`).

For example, select a plain paragraph whose range contains no note reference,
while an unselected source note contains a valid-id comment reference with
preserved opaque child content. That carrier is accepted and preserved by the
existing parser (`crates/rdocx-oxml/src/text.rs:2630` and its preservation
control at `crates/rdocx-oxml/src/text.rs:18276`). The new whole-part omission
nevertheless reaches the unselected carrier and refuses it at
`crates/rdocx/src/comments.rs:2889`. Thus
`create_building_block_from_fragment` or
`update_building_block_from_fragment` refuses an otherwise supported selected
transfer because of content outside its dependency closure. A malformed
qualified marker retained inside an opaque unselected wrapper has the same
scope problem.

The approved contract selects transferred note and textbox dependencies
(`.claude/plans/F-X188-design.md:128`). Existing dependency traversal follows
referenced note owners recursively
(`crates/rdocx/src/field.rs:4378`), and reverse glossary omission already uses
selected note owners. Apply that same bounded selection before forward
identity and comment dependency capture. Keep refusal for opaque or malformed
markers in a selected note. Add a control that includes selected and
unselected notes in one physical part, proves successful transfer despite an
unselected opaque carrier, proves selected-carrier refusal, and preserves the
source and failed destination bytes. The current selected dependency control
at `crates/rdocx/tests/regression_test.rs:61285` selects the full body and
does not establish this distinction.

This is a source-proved finding. No execution of this additional construction
was performed during the review.

## Smells

None found.

## Nitpicks

None recorded.

## Not found

- Correctness and contract: No additional findings beyond D1. All ten installer boundaries reconcile the complete candidate and validate a separate output clone before committing the original candidate. Existing authoring identities remain live until save. Glossary replacement, removal, local ownership and retired imported section targets reuse the strict graph.
- Panics: No unexpected new panic path found. Legacy infallible setters intentionally retain their documented panic convention. Fallible setters and Python propagate errors before publication. The real final OPC reopen and preparation refusal controls remain in place.
- OOXML: No additional finding. Qualified marker spans preserve mixed carriers and opaque siblings. Physical owner checks refuse local companions without definitions and shared marked note sources, while retaining shared unannotated notes. Glossary raw writes refresh typed state.
- Tests: D1 lacks the unselected-note distinction. Otherwise the retained controls cover the installer census, six variants, shared physical targets, opaque incoming uses, actual footnote and endnote deletion, local equal-id definitions, reverse producer note omission, Python revision behavior, CLI validation and refusal atomicity.
- Structure: No new production file, module, crate, trait, generic parameter, feature or dependency. Concrete main and glossary consumers justify the explicit owner argument.

## Evidence checked

Authenticated the immutable binder
`/private/tmp/fx188-final-source-binder/binder.json`, SHA256
`a225a3ceef073b83dc8c30cd889b735d5fe8bba5b314e7a30f727ed6f06989ef`,
against all18 live source bindings, 71 retained logs, five records and the
progress checkpoint. The source stayed frozen during review.

The current final4 scoped receipt has all13 steps successful. Its affected
native suites report1874 passed and22 existing ignored. Actual final3 Python
runtime reports99 passed, strict mypy checks seven files, and stubtest checks
six modules. The final4 rebinding records the native-test-only expectation
change and unchanged production and Python build inputs. Both the actual
imported and retained extension bytes authenticate to that provenance.

Opened both actual verified dry-run archives and compared every archived
source and test member with the current tree,25 facade members and three CLI
members. Both archives are below10MiB. The measured README tuples agree with
the authenticated archive records. All49 hash entries match. These are scoped
feature results, not final integrated sprint verification or publication.

The three exact-Base failure logs demonstrate the stated original operation
failures. Their transfer control uses forward creation, reverse capture and
generic cross-document import, not a public insertion before proof. Producer
reverse-note, companion-owner and shared marked-note failures are later
current-source controls. The shared-note failure proves validation ambiguity,
not an executed destructive deletion. Historical compilation failures and the
stopped old-source invocation are not accepted runtime proof.

No source, tests, plans, HLD, progress, Git or Cargo mutation was performed.
Only this review file was written. Return to the orchestrator for a separate
remediation phase.
