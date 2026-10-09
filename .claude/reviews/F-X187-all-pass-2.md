# F-X187, all, pass 2

**Reviewed**: Working diff on `work/f-x187-codex` against exact Base and HEAD `97c8b599887306d3fd0b4edbe45260e36767ece1`, 17 tracked changed files, 1690 added lines and 36 removed lines, plus the unchanged pass 1 review. Tracked binary diff SHA-256 `465491ef0c180730430e2b864dae8799037dbf4a85632fd647198a8ed282b187`. The complete captured diff including pass 1 is identified by SHA-256 `c697783edb4f8ea10258285aa12439351b6797b2ff1e7637264f6e334b67ab5d`. All worker source, test, documentation, measurement and progress writes were released before this pass. No active worker Cargo process was reported.
**Verdict**: 0 defects, 0 smells, 0 nitpicks. Pass 1 D1 is resolved.

## Defects

None found.

## Smells

None found.

## Nitpicks

None found.

## Previous finding

D1 is closed. `paragraph_story_location` at `crates/rdocx/src/document.rs:19970` now first resolves the actual facade paragraph and compares its concrete identity against direct body paragraphs and directly owned outer-control paragraphs. Its second segment is a direct-control paragraph ordinal. Nested control and table descendants return no two-segment location, as explicitly approved in the remediation contract. `story_paragraph_mut` at `crates/rdocx/src/document.rs:16064` uses the same direct paragraph iteration. The source scanner and scoped selection therefore interpret that ordinal consistently.

Actual pre-remediation native and built-Python controls each recorded the reported misdirection: local count one succeeded, A remained unchanged, and B was replaced. The native log is `/private/tmp/fx187-ordinal-before-native.log`, SHA-256 `56640485e453b3f843bf2690625e06beebd6fe7264078f69bfb8489a12aa2f05`. The Python log is `/private/tmp/fx187-ordinal-before-python.log`, SHA-256 `5c3136042d29ac77f25e7d1b717dfa013eaa29c0eec679158fb1788ffcfc6741`. These observations are separate from the exact claim-Base duplicate-clause failure.

The repaired native discriminator selects path `[0, 0]`, changes A, preserves B and refuses the unsupported deeper location. Two native controls pass, including empty direct paragraphs, nested controls, a table descendant, a later direct paragraph, and comment-anchor identity after reopen and replacement. Current Python coverage verifies the same replacement and deeper refusal with exact unchanged bytes and a still-valid held handle. The approved plan attributes the adapted direct-control correction to `hadim`, PR 287 commit `64854a40c9e44505dfac98aa975b4ad3505a2ec8`, and retains separate PR 286 contribution provenance.

## Evidence authentication

The final binder `/private/tmp/fx187-source-binder-final/source-and-evidence.json`, SHA-256 `f312b62dea2ef46830ff2f84263449dce49d62f9433590bcd3551547837689e9`, was independently authenticated against all 18 listed files, all 47 logs, all three retained artifacts, current HEAD, status, tracked binary diff and progress-note hash. The reviewed plan remains approved and the story remains in-progress. Its completion and sprint-close checklist items are subsequent workflow actions, not claims that those actions have already happened.

The approved contract, current HLD changes and source/test implementation were examined. The original named issue discriminator actually compiled and failed at the exact claim Base through the old global guarded API. The table/cell attribute loss was independently exposed and repaired before this review. Other added preservation cases remain qualified as after-only controls. Earlier zero-selection, fixture, cache and toolchain failures do not count as positive acceptance.

The current affected native all-feature run records 2422 passes and 22 existing ignored tests: 508 facade unit, 364 integration, 913 regression, 634 XML unit and three documentation tests. Focused runs include physical duplicate-sibling proof, initial preparation refusal, actual OPC final-reopen entry-limit refusal, namespace and container conflict refusal, selected and nested root attributes, exact opaque XML, wrapper and field boundaries, related stories, and existing anchor and movement controls. Eleven CLI global replacement controls pass.

The actual remediated extension was rebuilt with Rust 1.97.1. The retained extension at `/private/tmp/fx187-final-extension/_rdocx.abi3.so` independently matches SHA-256 `71adef3ca0ba1615f1e2468c050d9276d651800eb62eaf2c86d0fbef348870ab` and 86735016 bytes. Current runtime import provenance identifies that extension. The final Python core run passes 97 tests. The immediately preceding run's 96 passes and one incorrect refusal-exception expectation are retained as historical evidence, not reported as a passing run. Strict mypy 2.3.0 validates seven source files and actual runtime stubtest validates six modules.

Affected all-target checks, denied-warning Clippy and rustdoc, both WASM graphs, all 49 unchanged deterministic hashes, 27 README inventories and 22 publishable package inventories, five affected compiled Rust README examples, prose, adapter drift and formatting pass. Literal helper rustc and rustdoc probes identify version 1.97.1. Workflow tests record 140 tests with two existing skips in 114.718 seconds. This is feature scoped evidence and does not replace final integrated workspace verification or sprint review.

## Archive rider

Both measured archives and both actual publish-verification archives were independently opened. Every archived Rust source, test and example member matches the current worktree byte for byte, 27 members for `rdocx-oxml` and 31 for `rdocx` in each archive. Normalized VCS member accounting and member counts were independently recomputed. Measured and actual archives differ by one compressed byte per crate, within the existing 64-byte tolerance, with identical normalized member measurements.

| Package | Measured compressed bytes | Actual publish compressed bytes | Normalized member bytes | Members |
|---|---|---|---|---|
| rdocx-oxml | 450129 | 450128 | 2798010 | 32 |
| rdocx | 1564667 | 1564666 | 8680657 | 36 |

All four archives are below 10 MiB. Current README rows and measurement tuples match the measured archives. Actual publish-verification archives have SHA-256 `51c523ba4d162e98740fda03ef0d5a65fa2651039adde860d128cbe191f8e38e` for XML and `17e587ea02aad16e3fea5de9855a349e420d2a95ab54935932489b3339846ddc` for the facade. The actual publication check invokes `cargo publish --dry-run --allow-dirty` with all 22 local patches and retains package verification. Logs show both crates being verified and upload being aborted because of dry run. No registry upload or release publication occurred.

## Not found

- Correctness: no remaining defect in local counts, direct paragraph projection, physical span correspondence, staging or once-only publication.
- Contract: no remaining defect in the approved concrete native and Python APIs, detached StoryItem behavior, related owner support, zero and mismatch semantics or approved direct-control correction. Issues 264 and 291 remain outside scope.
- Panics: no finding in checked owner paths, range selection, pointer equality, ordinal lookup or fallible parsing and reopen handling.
- OOXML: no finding in qualified namespace handling, strict X186 replay, schema-order reuse, topology-checked scoped table/cell attribute restoration, wrapper and field boundaries or exact opaque and unselected-source retention. Global serializers and replacement behavior remain unchanged.
- Tests: no remaining finding. D1 has actual native and Python failure observations and current successful controls. Runtime, typing, CLI, archive and deterministic output evidence remain distinct.
- Structure: no finding. No new production file, module, dependency or trait. The shared binding generic has current item and physical-cell consumers. The one-shot reopen injection is test-only.

No code, test, plan, HLD, measurement or progress file was changed by this review. No Cargo or UI command was run. This pass writes only this review record and returns to the sprint orchestrator. Prepare, integration, final integrated verification, sprint close and release approval remain subsequent workflow actions.
