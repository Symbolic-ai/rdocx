# F-X187, all, pass 1

**Reviewed**: Working diff on `work/f-x187-codex` against exact Base `97c8b599887306d3fd0b4edbe45260e36767ece1`, 13 tracked files, 1534 added lines and 8 removed lines. Full binary diff SHA-256 `a992c14f5eb4ddbb0363f37faf8778b28b1d1f992af66af977be83f70e31fda0`. The worker released all writes before this formal pass. The untracked built Python extension is runtime evidence, not a tracked source change.
**Verdict**: 1 defect, 0 smells, 0 nitpicks. Completion is blocked.

## Defects

### D1, Recursive paragraph handles can replace a later direct sibling

`crates/rdocx/src/document.rs:26651`

The new scoped operation indexes the direct-only result of `scan_story_control_paragraphs` with the second segment returned by `paragraph_story_location`. Those ordinals use different traversals. `paragraph_story_location` counts recursively through a control at `crates/rdocx/src/document.rs:19974` and stores the remaining recursive ordinal at `crates/rdocx/src/document.rs:19978`. The scanner instead accepts only paragraphs whose nearest owning control is the selected outer control at `crates/rdocx/src/document.rs:9267`. Nested-control paragraphs are absent from that list.

For an outer block control containing a nested control with paragraph N, followed by direct paragraphs A and B, the public paragraph order is N, A, B. Recursive collection is explicit at `crates/rdocx-oxml/src/content_control.rs:1212`. The handle for A has recursive ordinal one, but the scoped resolver indexes direct list [A, B] with one and selects B. `Paragraph.replace_text` connects this projection directly to mutation at `crates/rdocx-py/src/paragraph.rs:261` and `crates/rdocx-py/src/paragraph.rs:265`. If A and B contain the same literal once, `expect=1` succeeds and publishes a change to B while leaving A unchanged. The physical-position probe cannot detect this error because both probe spans have already been selected as B. A nested first paragraph can similarly resolve to A, while later handles can fail out of bounds despite addressing existing supported paragraphs.

Resolve the scoped paragraph from the same namespace-qualified recursive grammar used by the originating handle, or use a checked physical projection that proves the actual paragraph identity. Add source-built native and actual Python controls containing nested controls before multiple direct siblings, with identical literals and distinct surrounding text. Check both the nested paragraph and the later direct paragraph. Existing flat-control coverage at `crates/rdocx/tests/regression_test.rs:59948` does not exercise this ordinal disagreement. This is a source-confirmed defect, not an executed runtime reproduction in this audit.

## Smells

None found.

## Nitpicks

None found.

## Evidence and limits

The approved plan and cited architecture, package, binding and testing sections were read. The source/evidence binder at `/private/tmp/fx187-source-binder/source-and-evidence.json` was independently authenticated against all 13 current source hashes, all 20 recorded log hashes and the full diff. No source, test or metadata file was modified during this pass.

The exact-Base discriminator log records one compiled, executed failure through the old global guarded API. The container-attribute before log records one executed loss failure, and its after log records one passing paragraph/cell/table control. These are separate historical observations. Added preservation controls are after-only evidence.

The current focused receipt records actual passing selections of 3 XML units, 2 preparation/reopen units, 1 physical sibling probe, 1 named issue gate, 1 cell/table test, 6 scoped preservation tests, 13 anchor tests and 6 movement tests. The actual extension was rebuilt from held source. The first full Python core run had 95 passes and one pinned-Poppler version failure. The corrected pinned rerun records 96 passes. Strict mypy records seven source files, stubtest records six modules, and affected Clippy denies warnings successfully. None of those controls covers D1.

Affected complete native suites, package archive measurements, actual patched publish dry runs and the remaining scoped policy gates are still due. This review does not claim complete `/verify --scoped`, final archive acceptance, the integrated full sprint gate or release readiness. No Cargo or UI operation was run by this reviewer.

## Not found

- Correctness and contract: no additional finding beyond D1 after examining owned staging, scoped counts, source correspondence, related-story equality and publication.
- Panics: no new finding in checked paths, ranges, pointer comparison or fallible source handling.
- OOXML: no additional finding in qualified owner selection, strict replay, topology-checked scoped table/cell attribute restoration, field and wrapper boundaries or opaque retention.
- Tests: no additional finding beyond the missing nested-control discriminator described in D1. Actual fail-before and after-only claims remain distinguished.
- Structure: no new trait, dependency, module or production file. The shared binding generic has current item and physical-cell consumers. No additional finding.
