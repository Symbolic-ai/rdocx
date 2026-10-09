# F-X189, ALL, pass 1

**Reviewed**: Frozen working diff against claim Base38cf126a23a687f75e5bb7470a67f033f2bd6cb7 on work/f-x189-codex. The tracked diff covers58 files,433 insertions and294 deletions. The freeze additionally retains the worker progress record. All six aspects were inspected independently against the approved design, release workflow, completed S89/S90 delivery range, contribution inventory and current compatibility sources. No implementation, Cargo, test execution, network, native UI or Git mutation occurred in this pass.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None found.

## Smells

None found.

## Nitpicks

None recorded.

## Correctness and family contract

`Cargo.toml:34`
`Cargo.toml:55`
`crates/rdocx-py/pyproject.toml:7`
`crates/rpptx-py/pyproject.toml:7`
`scripts/test_sprint_workflow.py:29`

The stable workspace group, exact seven published Word crates and rdocx Python carrier agree at0.16.0. The exact fifteen published shared OOXML/PowerPoint crates and rpptx Python carrier agree at0.14.0. Unpublished oxml-py-support, rdocx-py and rdocx-wasm inherit the stable group. rpptx-py remains unpublished and explicit0.14.0. rpptx-wasm remains unpublished0.12.1. Publishability, allowlist membership and dependency features are unchanged.

Independent parsed comparison confirms26 local lock records change only their version fields. External lock records, checksums and dependency lists are identical. Root dependency entries differ only in intended internal versions. No dependency direction, runtime API, external tool version or new source consumer is introduced.

`.github/workflows/ci.yml:386`
`scripts/test_sprint_workflow.py:88`

Stable WASM assertions advance to0.16.0, while separate rpptx-wasm0.12.1 and wasm-pack tool0.15.0 remain exact. The distinction is tested rather than relying on broad string substitution.

## Release notes and compatibility

`CHANGELOG.md:6`
`CHANGELOG.md:16`
`CHANGELOG.md:31`
`CHANGELOG.md:39`
`CHANGELOG.md:46`
`docs/hld/10-bindings-spec.md:129`

Both prepared family sections contain the prescribed five meaningful subsections. The Word body retains all10 PR and14 issue links, authenticated hadim, pedroassumpcao and changjoon-park handles, specific contribution outcomes and direct versus hardened-equivalent classifications. Shared/PowerPoint notes include PR269 and its contributor, while excluding Word-only feature outcomes. The accepted work is not described as an unchanged merge of the open upstream stacks.

S89 notes, ranges, fragments and glossary capabilities remain in the range since previous family tags at9d019472f7e6b95ac4ba0770c4dee35dcbc28f0e. PR280's inherited279, PR287's inherited271 and PR290's inherited287/271 are not counted as new independent fixes. Issue282 retains independently implemented reporter credit. PR293's namespace predicate core is credited without importing its stale platform measurement override.

F282 claims remain native Rust only. Seventeen source kinds, sixteen roles and twelve styles describe metadata, while210-of223 dense APA configurations and lean numeric1033 Book consumers describe bounded formatting. Citation admissions remain separate. Atomic unfinished-standard refusal, noncatalogue retention, deferred F-X192 catalogue and carried F-283 navigation are explicit. No dedicated bibliography Python/WASM API, arbitrary Unicode ordering or universal Word pixel parity is claimed.

Compatibility accurately lists all seven added LayoutInput members, including footnote_layout_like_word8. CT_Shape, CT_NoteProperties and NoteLayout literal changes are explicit. Both families document shared note_reference_source fields, four FieldKind variants and nested-preorder FieldSource.index semantics. The formerly stale TextSegment shape statement is corrected. No crate-private paginator fields are presented as public migration obligations.

The entire historical CHANGELOG tail beginning v0.15.0 remains byte-identical to Base. Issue264 is absent from new notes and planned notifications, and Issue281 stays open. Known-record refresh binds all10 unchanged accepted heads, closed269 and nine open PRs plus fourteen open issues. Closed269 is not used as proof of an upstream merge. Planned record-specific notifications are not posted actions or premature release approval.

## Tests and evidence

`scripts/test_sprint_workflow.py:29`
`scripts/test_sprint_workflow.py:3734`
`scripts/test_sprint_workflow.py:5785`
`scripts/test_sprint_workflow.py:6869`

The named new release regression validates both exact family inventories, carrier/lock agreement, notes, contributor and record coverage, unpublished inherited carriers and the distinct WASM/tool versions. The genuine before receipt retains32 carriers independently verified byte-identical to claim Base. Its actual execution fails on the old stable group, absent selected shared versions and inherited support carrier, not on missing API or import errors.

Current-version assertions advance only where they inspect live manifests or examples. Historical rendered release bodies and their tag selectors remain unchanged, including the rpptx-v0.13.1 body within the renamed-by-behavior current carrier test and historical recovery assertions. No existing assertion or test is removed. The original placeholder-token refusal is retained as a failed intermediate execution and the actual authored content now uses content-control template wording.

The final focused carrier log502c7b7a reports12 passing tests. Qualified note controls log0fcc5ea8 reports two passing tests. The intermediate failed focused log, genuine Base gate and render bodies remain separately authenticated. Negative README diagnostics in the successful focused run are expected mutation checks, not masked gate failures. No scoped completion is inferred from these focused controls.

## Package policy and structure

`scripts/readme_doctests.py:367`
`scripts/readme_doctests.py:393`
`scripts/readme_doctests.py:420`
`scripts/readme_doctests.py:1291`
`docs/hld/15-build-and-toolchain.md:530`

Current README installation examples and all affected measured rows follow the selected versions. Archive dates/tuples are backed by real measurements. Existing normalized VCS accounting,64-byte compressed tolerance,22 local patches,10MiB ceiling and historical deterministic performance observations are unchanged. Independent read-only inspection authenticated all22 latest wave2 archive hashes, their measured tuples and166 actual src/test payloads against current source. Maximum observed compressed size is4634515 bytes and maximum recorded compression difference is2 bytes. This is archive measurement/source proof, not a verified workspace publication dry run.

The production .rs diffs change only existing test version strings. All runtime algorithms, XML schemas, field/cache operations, namespace preservation and unknown XML handling remain unchanged. No new production module, trait, generic, wrapper, dependency, feature flag or runtime flag appears. Exactly the approved HLD10/14/15 files and current guidance are updated. Preparation/past publication/future release authority are distinguished without changing historical releases or external permissions.

## Not found

- Correctness: zero findings in version carriers, allowlists, pins, lock records and notes.
- Contract: zero findings. Bounded completed bibliography, historical range, contributor classification and separate final approval remain intact.
- Panics: zero findings. No changed runtime indexing, slicing, arithmetic, unwrap or untrusted-input path.
- OOXML: zero findings. No parser, writer, child order, namespace or preservation logic change.
- Tests: zero findings. Genuine version-failure evidence and focused after controls are bound. Historical record assertions and negative carrier checks remain active.
- Structure: zero findings. No speculative construct or new production file. Scope is release metadata, examples, policy assertions, notes and approved HLD guidance.

## Authentication and pending gates

Freeze manifest /private/tmp/S90-X189-ALL-source-freeze-ju1hbdtp/manifest.json SHA2561d227004f074c2bfbbf971347e11b36785d082a61ebaf7c041440fd84df602cf independently authenticates59 live/retained files and21 evidence records. Every bound file matched before inspection and immediately before writing. Exact live binary patch SHA256b5bd1ea8e8929706c9b01f947d604c7090adfe83c8b47368b79357f4cfdcc918 matches the manifest. Accepted contribution inventory1291315355a6a6504ca747cd34c62ce1d295a4bf212a6f7b43f5012bd402bc12 and known-record refresh5427c159403d7576cedf3115a719ee63315067c39949a05b05d9d3b2dca91bfb were authenticated without further network access.

This static clean verdict does not claim actual new-version clean Python3.9/3.12 wheel/sdist checks, typing, both WASM gates, verified22-package dry run, scoped completion or integrated full verification. Those remain worker/root obligations. Final reviewed-SHA sprint verification, sprint review and hosted build-only twelve wheels/two sdists precede close. Manual dispatch does not earn tag-only CLI archives. Registry/tool snapshots do not constitute final publication approval. Each family still requires its own immediate approved reviewed-main SHA, shared family first where actual Word pins require it.

The sole write is this designated review file. Review write ownership is released on return. The pass ends here without remediation or a confirmation pass.
