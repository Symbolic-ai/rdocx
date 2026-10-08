# F-X186, all, pass 2

**Reviewed**: Working diff on `work/f-x186-codex`, Base and HEAD `c303d34b76f762e626df7f46a607069b89661b87`. All 16 tracked paths, 1784 insertions and 25 deletions. Includes separate remediation of pass1.
**Verdict**: 1 defect, 0 smells, 0 nitpicks. Remediation and another independent review are required.

## Identity and scope

Reviewed against the approved plan, its cited HLD architecture, packaging, binding and testing contracts, CLAUDE, WORKFLOW and microscope. Revisited all six aspects and the complete current diff, with particular attention to the changed raw-reference ownership and canonical alias discriminator.

Independently authenticated `/private/tmp/fx186-pass2-freeze.json`, SHA-256 `b3d040b7a59e55ee652e873b7d8a294756ad4eb3535d270399de8effeb64de4c`. All 16 tracked files, 46 logs, six receipts and three evidence files match their bindings. Exact status and ordinary binary diff match. Diff SHA-256 is `c1c2f81a0b96d011e26a0a7fb7c8523cca41069d2e88da7b234e1bdb4dcc8ef0`. Pass1 record SHA is `04bc8acd2a78f7927316a6f71265d656f89bdb2714d47500f4a0ff2a82eda9df`.

## Defects

### D1, the shared literal finder now rejects previously supported exact matches

`crates/rdocx/src/comments.rs:1394`

The shared helper finds and splits a literal-text match, but replaces the prior exact-range guard with `accepted_comment_projection(id, false)`. The rich accepted-display reader has a different text and namespace context from the existing literal anchor operation. This changes both existing add-comment behavior and the new move-to-text operation that promises the same finder semantics.

Two unchanged existing tests reproduce distinct manifestations of this one guard replacement:

- `crates/rdocx/tests/regression_test.rs:2653`: in `removing_a_comment_clears_fixed_prefix_markers_in_a_document_of_another_prefix`, a document uses `ns0` for Word elements, while new markers use `w`. Adding a comment on literal `B` inside its content control now returns an error claiming the range shows an empty string. The original fixture is at `regression_test.rs:2422`.
- `crates/rdocx/tests/regression_test.rs:2969`: in `comment_on_text_refuses_a_range_that_would_show_a_field_result`, literal `AB` across a tab is now refused because the rich reader reports `A\tB`. The public add-comment contract at `comments.rs:1294` explicitly assigns tabs and breaks zero width for this literal operation. The unchanged test also requires refusing ranges that accidentally include simple or complex field results, so merely removing the exact guard is insufficient.

The authenticated current `/private/tmp/fx186-20261008T174314Z/rdocx.log`, SHA-256 `05d0433b248e072a2f686f6eb5ecc1f8ac65138ebb618af543edf819b2014980`, contains both actual errors and the regression result of 898 passed, two failed and seven ignored. These are compatibility failures, not optional fixture changes.

Keep the rich X185 anchor reader semantics intact for its existing callers. The literal add/move finder needs its documented zero-width and exact field-display safeguards under the actual paragraph namespace context. Preserve qualified marker identity, opaque XML and atomic refusal. Reconcile both unchanged tests and add direct coverage for the corresponding move-to-text route rather than redefining their expected behavior.

## Pass1 remediation assessment

- Prior D1 is fixed in the reviewed source. Unordered fallback skips only entries with actual private carrier flags, not raw elements inferred from numeric identity. The compiled aggregate test demonstrates all three independent raw constructions, with absent typed reference, same typed id and different typed id. It failed before and passes after. Erased private positions leave raw references independent. Normal flagged carriers still emit once and do not survive typed removal, replacement or id changes.
- Prior D2 is fixed. The element's own Word alias declaration is structural, while other declarations, payload attributes and paired content retain source. The unchanged canonical alias test passes. Full current oxml evidence is 630 unit tests and one doctest passing.
- Prior D3 is fixed by one localized `type_complexity` allowance and concrete tuple rationale on `comment_move_source_edits`. Current four-crate all-target, all-feature denied-warning Clippy passes. No speculative abstraction was added.

## Smells

Zero.

## Nitpicks

Zero.

## Aspect results

- Correctness: D1. No additional finding in reference transport, strict root selection, staged source edits, physical paragraph rebasing, destination namespace closure or conservative Google wrapper pruning.
- Contract: D1. No additional finding in identity, replies, companion bytes, supported story placement, stale-path checks, refusal atomicity or Python revision behavior.
- Panics: zero findings in the changed production paths.
- OOXML: zero additional findings beyond the namespace-context manifestation of D1. Pass1 raw preservation and canonical serializer defects are resolved.
- Tests: zero additional findings. The full facade suite exposed the compatibility defect missed by focused controls. The original positive fail-before evidence and current movement controls remain valid within their scope.
- Structure: zero findings. Concrete helpers have actual callers. No new production file, crate, module, trait, generic or dependency was introduced, and the earned lint failure is resolved.

## Verification qualifications

Current focused evidence establishes native movement six, CLI one, the independent raw-reference aggregate one, unchanged canonical alias one and carrier lifecycle one passing. Actual current Python rebuild, its movement control and all 94 core tests pass. Current scoped fmt and Clippy pass. Full oxml passes, and facade unit 501 and integration 364 pass. Facade regression fails as stated above.

Scoped verification is **INCOMPLETE**. Later scoped CLI, documentation, WASM, typing, current hash harness, package measurement, publish dry-run, README inventory and workflow-policy acceptance were not run. Historical pass1 lint and canonical alias failures are superseded evidence, not current unresolved failures. Historical Base hash49 is not a current hash claim. No baselines or archive measurements were updated, and no native Word oracle, full feature acceptance or sprint completion is claimed.

Only this review record was written. No source, test, plan, HLD, progress, baseline or other artifact was changed, and no Cargo command was run by the reviewer. Return control for separate remediation.
