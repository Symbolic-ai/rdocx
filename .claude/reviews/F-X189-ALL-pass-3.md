# F-X189, ALL, pass 3

**Reviewed**: Distinct test-only official identity remediation after ALL2 ended and an actual second size-pipeline failure. Working branch work/f-x189-codex, claim Base38cf126a23a687f75e5bb7470a67f033f2bd6cb7. Increment against ALL2 is five added plan lines, eight added/two removed existing test lines and fourteen added ignored progress lines. Other56 bound records remain exact ALL2 bytes. The entire corrected size pipeline and all six aspects were independently inspected. No Cargo, test execution, source edits, formatting, network, UI or Git mutation occurred.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None found.

## Smells

None found.

## Nitpicks

None recorded.

## Official version contract and negative control

`.github/workflows/ci.yml:320`
`docs/hld/15-build-and-toolchain.md:828`
`.claude/plans/F-X189-design.md:120`
`crates/rpptx-wasm/src/lib.rs:272`
`crates/rpptx-wasm/src/lib.rs:339`

The validator and positive fixture now compare the exact official `wasm-opt version 125 (version_125)` identity already required by CI and HLD. This is not prefix acceptance or a tool-version upgrade. Binaryen125, wasm-pack0.15.0, manifests, dependency pins and optimization flags remain unchanged.

`crates/rpptx-wasm/src/lib.rs:369`
`crates/rpptx-wasm/src/lib.rs:374`

The new case sets the abbreviated identity and requires the exact version diagnostic. Version checking precedes arguments and gzip binding, so the guard is specifically about identity. It restores the official identity before the existing wrong-gzip assertion. Earlier unrelated-gzip, wrong wasm-pack/default-profile, omitted nontrapping and wrong gzip-level controls remain active. No assertion is deleted or replaced by a looser expectation.

## Entire size pipeline

`crates/rpptx-wasm/src/lib.rs:127`
`crates/rpptx-wasm/src/lib.rs:137`
`crates/rpptx-wasm/src/lib.rs:178`
`crates/rpptx-wasm/src/lib.rs:199`
`crates/rpptx-wasm/src/lib.rs:239`

The actual builder uses the concrete default wasm-pack nodejs release profile with --no-opt, then Binaryen125 with -Oz, bulk-memory and nontrapping. Command success is checked before reading optimized bytes and gzip. Normalization remains indices3/5 in the six-element optimizer vector, preserving both flags and -o. Wasm-pack indices1/7 and gzip index3 remain valid for their concrete arrays. No disabled default feature, alternative artifact or relaxed optimizer contract enters this correction.

`crates/rpptx-wasm/src/lib.rs:268`
`crates/rpptx-wasm/src/lib.rs:280`
`crates/rpptx-wasm/src/lib.rs:283`
`crates/rpptx-wasm/src/lib.rs:287`
`crates/rpptx-wasm/src/lib.rs:302`
`crates/rpptx-wasm/src/lib.rs:305`

Validation keeps exact versions, all argument arrays, WASM magic and deterministic gzip -n -9 header. It decompresses the actual gzip and compares all bytes with the freshly optimized WASM. It refuses lengths at or above1,000,000. No stale smaller unrelated artifact passes only by satisfying the threshold.

`crates/rpptx-wasm/src/lib.rs:313`
`crates/rpptx-wasm/src/lib.rs:322`
`crates/rpptx-wasm/src/lib.rs:325`

The actual ignored gate retains its strict ceiling, nontrivial deck full-package roundtrip and new-presentation checks. Native and Node controls remain separate evidence, and their passes do not substitute for the optimized-size execution.

## Actual second failure qualification

The retained log414ac976d6358fd266b1baceeedfd82f12fef688428aa8c83db5383369ea0427 records an actual release build and executed ignored test, then exact default size pipeline refusal on the abbreviated version-validator expectation. Source control flow reaches that diagnostic only after corrected optimizer and gzip commands return successfully. That progress is not a size or roundtrip pass.

Pre-identity sourcecc3360609e3a9ab107db0911aa91adad43dc4930b9b8ec7b5c4f0712ab40ef81 is exact ALL2 source. The second-failure receiptf3d39990e536713cffa71db55f1bd40c71f7eee1525ab4f05481e54c8a5bf5cf reports exit101. This is distinct from the first Binaryen input validation failure and the executed omission-before assertion failure. No missing-API, import or compilation failure is substituted.

Separately retained scoped receiptc23fa5e028361daaf76f853c228a276c6ced9306799d98c7ea7c528548b108c2 and native-controls log4082d93c837840f47c3681ffc619551fba656f78c8c91d9ee88a1dd363e9b798 record three native controls passed and the actual size gate ignored. They retain their checkpoint qualification. They are not relabelled execution of the newly frozen official-identity correction or an actual size pass.

## Not found

- Correctness: zero findings. Official identity, fixture and reset agree, while full command, path, gzip and threshold behavior remains intact.
- Contract: zero findings. Existing CI/HLD identity is enforced by a bounded approved test correction. ALL1/ALL2 family carriers, notes, compatibility, archive policy and approval barriers remain unchanged.
- Panics: zero findings. No production input operation changes. Private test argument indexing is within concrete array lengths. String assignments add no slicing or arithmetic risk.
- OOXML: zero findings. No schema, namespace, parser, writer, unmodeled XML or relationship mutation changes. Full-package roundtrip assertions remain intact.
- Tests: zero findings. The abbreviated identity guard detects the intended boundary and resets before the next control. Actual failures remain separately preserved, not called passes. No skip obligation or size threshold changes.
- Structure: zero findings. Executable changes stay inside existing cfg(test). Production prefix is byte-identical to ALL2. No API, module, dependency, trait, generic, wrapper or flag.

## Authentication and evidence correction

Corrected freeze /private/tmp/S90-X189-official-version-ALL3-evidence-corrected-qznrzidk/manifest.json SHA256412110677f4e2b06391337047013a835cc302d85ecb7dfd05d4e1fc1a5e76e0d binds59 current/retained records and six original/retained evidence pairs. All hashes matched independently before final inspection and immediately before writing. Incremental exact-version-test.diff SHA256807ee78cc88d36cfe42d2c7addcbb5631e744a4bb76da3932523aa698421246b matches the manifest and inspected source increment. Current test SHA256a711d69c47345523a1c61f59fc7b87d8f80d9828160211fdecca06216b531553.

Initial binder SHA256efdc70fca993a20351cb4d9d4837b6d1def6593db2c5b24263c04a6f0517ffed collided the two receipt basenames. Authentication caught the temporary retention error before verdict. Original evidence remained available and exact. Replacement paths are unique and preserve all59 source hashes, all six original evidence hashes and exact incremental diff. The bad binder remains retained and explicitly superseded. No source changed to correct this evidence process error.

## Pending execution and handback

Current corrected focused/size executions and actual bundler/npm artifacts remain worker/root obligations until observed and source-bound. This clean static verdict does not claim the gzip threshold passed, a hosted build succeeded, scoped completion or publication approval. Other Python/package evidence retains its own actual receipt/source qualifications. Final integrated full verification, sprint review, hosted exact-SHA build-only rehearsal and separate final family approvals remain mandatory.

Only this designated review file was written. ALL review write ownership is released on return. The pass ends without remediation or a confirmation pass.
