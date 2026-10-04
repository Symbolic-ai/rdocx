# S86 sprint review, pass 1

**Reviewed**: `sprint/s86` at `a30715fe` against merge base `080d06a7`, 114 files, 8,374 changed lines. Crates touched: `oxml-chart`, `oxml-cli-support`, `oxml-core`, `oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`, `rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`, `rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-py`, `rpptx-render`, `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

M24 requires an approved capability matrix with no unexplained partial row and a broad source-built document and pinned Word corpus passing validation, authoring, mutation, save-reopen, deterministic rendering, accessibility, preservation, and binding parity without Word repair. S86 advances this milestone and does not close it. F-274 and later M24 stories remain scheduled in S87 through S91. The S86 authoring gates passed their focused Word comparisons, the integrated workspace test suite, and the 49-entry hash harness. F-273 records the known endnote page and label policy difference with Word, which F-274 owns.

The S86 release work prepares the two family versions and unified pipeline. Both family contracts, release-note checks, clean 22-crate package dry run, locally installed Python wheel suites, and the integrated full gate passed. The hosted six-platform build-only rehearsal is scheduled at the final pushed S86 SHA before sprint closure. Tag creation, registry publication, release assets, attestations, and Issue 266 notification follow `/close-sprint` under separate `/release` approvals. This review does not claim those later gates have passed.

## Not found

Interaction: the integrated note and namespace tests pass with both release version families in the same workspace.

Duplication: no new duplicate helper was found across the seven F-ID diffs.

Layering: no `oxml-*` dependency on an `rdocx-*` or `rpptx-*` crate was added.

Harness: every plan expected unchanged output and all 49 integrated entries match.

Gate: the named feature contracts, integrated workspace tests, Python wheel checks, policy suite, README checks, package dry run and supply-chain check pass. The hosted and publication stages remain explicit later gates.

Docs: the sprint definition of done now names the post-close publication boundary, and each feature updated its listed HLD files.

Deps: the release version and pin changes use the existing package graph. No new external dependency was added.

Surface: the public authoring additions follow F-271 through F-273. The release stories change version carriers and workflow, not extra public APIs.
