# F-X188, all, pass 2

**Reviewed**: Frozen working diff against claim Base `ce6d46baec4af822c802d55f20cb9e83c1a6528e`, 18 tracked files, 2494 insertions and 79 deletions, plus immutable pass1. F-X188 remains in progress. The approved plan, cited HLD, progress, complete implementation and prior review were read. All pass1 production bindings remain unchanged except the bounded document and field traversal repair.

**Verdict**: 1 defect, 0 smells, 0 nitpicks.

## Defects

### D2, Forward omission still examines retained root payload outside the selected body

`crates/rdocx/src/field.rs:3197`

The repaired forward path selects note owners recursively, but still passes
the entire prepared main document part to `omit_comment_markers`. That part
can retain a producer background outside the selected `w:body`.

A concrete admitted source is a foreign `producer:background` carrying a
qualified, valid-id `w:commentReference` with opaque child content, followed by
a plain body paragraph. The document parser captures a foreign background's
complete payload at `crates/rdocx-oxml/src/document.rs:2959`, preserves it
through serialization at `crates/rdocx-oxml/src/document.rs:3079`, and explicitly
tests that preservation at `crates/rdocx-oxml/src/document.rs:3276`. This foreign
container is opaque producer content, so its nested reference need not be a
rendered or transferred body annotation.

`DocumentFragment::from_range` prepares the source and checks the selected
story and item boundaries without invoking whole-source comment ownership
validation (`crates/rdocx/src/document.rs:1144`). Its direct body capture
replaces only the body selection and writes the retained package
(`crates/rdocx/src/document.rs:1206` and
`crates/rdocx/src/document.rs:1224`). The background therefore survives in the
private fragment package. The new whole-part omission reaches its opaque
carrier and returns an error at `crates/rdocx/src/comments.rs:2889`, refusing
plain-body glossary creation or update because of unselected root payload.

The existing import identity scan collects only within the body
(`crates/rdocx/src/field.rs:9680`), and the actual content extraction selects
the physical body owner (`crates/rdocx/src/document.rs:6897`). A background
with no note references is therefore not already an admitted dependency that
this transfer must process. The approved selected-body clarification is at
`.claude/plans/F-X188-design.md:320`, and its transferred-content contract is
at `.claude/plans/F-X188-design.md:128`.

Bound both marker omission and initial note dependency selection to the
prepared selected body, retaining its inherited namespaces. Preserve the
fragment's unselected root payload. Add source-built forward create and update
controls with admitted foreign background payload, unchanged source bytes,
successful selected body output and atomic refusal when the same unsafe
carrier occurs inside the selected body. This is a source-proved finding,
not an executed reproduction from this review.

## Smells

None found.

## Nitpicks

None recorded.

## Prior finding

Pass1 D1's unselected-note defect is resolved. The forward path now derives
recursive note owners from prepared source, before identity and comment
dependency capture (`crates/rdocx/src/field.rs:3196`). The shared helper checks
normalized physical main identity before refusing glossary-local numeric
projection (`crates/rdocx/src/document.rs:19058`) and modifies only selected
note spans. Its visited set bounds cycles and repeated references.

The genuine current-before log reports eight valid selection failures and
four selected-bad refusal controls. The expanded after matrix reports16
successes across actual footnotes and endnotes, eight atomic refusals and
recursive Footnote to TextBox to Endnote creation, update and public insertion.
Both recursive fixture-construction failures remain explicitly unqualified.
These controls resolve note selection, while D2 concerns retained main-root
payload rather than note owners.

## Not found

- Correctness: No additional finding beyond D2. The ten complete installer transactions, imported physical-target retirement and glossary cleanup retain their reviewed ownership boundaries.
- Contract: D2 is the remaining selected-content boundary defect. Full Issue288 and292 scope, qualified local owner isolation, legacy undefined-marker policy, source preservation and separate Issue291 scope remain explicit.
- Panics: No unexpected new panic path. Infallible compatibility wrappers intentionally panic on refused candidates, while fallible and Python entrances propagate errors before publication. Existing live identity assertions and actual final-reopen refusal remain intact.
- OOXML: No additional finding. Marker omission retains carriers and opaque siblings. Local companions without definitions and shared marked physical notes refuse. Shared unannotated notes remain supported, and glossary raw and typed state remain coherent.
- Tests: The note selection distinction is now proved. D2's retained root payload distinction is absent. Installer, shared-target, companion, glossary-local, native note, reverse producer, Python revision and CLI controls remain intact.
- Structure: No new production file, module, crate, trait, generic parameter, dependency or feature. Main and glossary are concrete consumers of the shared traversal and ownership code.

## Evidence checked

Authenticated `/private/tmp/fx188-d1-final-source-binder/binder.json`, SHA256
`70012ae801ea1587f395ae7dce2f1641d40815b15a65f599b4a65c6871951405`,
all18 live and captured source bindings,82 retained logs, seven records and
progress. Pass1 authenticates unchanged to SHA256
`d2af24de7b18f23b78e2afde2987d8043e03a3e3cccd8167152c7139a17cb7d2`.

Final5 scoped verification has all13 steps successful. Affected native suites
report1877 passed and22 existing ignored. Fresh rebuilt Python runtime
reports99 passed, strict mypy checks seven files and stubtest checks six
modules. Both imported and retained extension bytes authenticate to final5
provenance. Both actual verified publication dry-run archives were opened,
and every archived source and test member matches the current tree,25 facade
and three CLI members. Archives remain below10MiB and current measurement
records match the README tuples. All49 hash entries match.

Only the three declared original logs are exact-Base failure evidence. D1 and
the other later repairs have separate current-source failure logs. Shared
marked-note evidence demonstrates validation ambiguity, not destructive
deletion. Historical compilation and fixture failures are not accepted
behavior proof. These gates remain scoped feature evidence, not final
integrated sprint verification or publication.

Only this review file was written. No source, tests, metadata, Git or Cargo
mutation occurred. Return to the orchestrator for separate remediation.
