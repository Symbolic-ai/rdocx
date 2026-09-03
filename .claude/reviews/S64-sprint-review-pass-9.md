# S64 sprint review, pass 9

**Reviewed**: clean `sprint/s64` at
`ff66b63f054b766669e933ca0ff209ceda37aeba` against
`0582da0a38886f5ceeb65ab9afcd0797f6fa14b0`, 107 files, 20,239 changed
lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

Pass 9 is the post-publication review for the `scheduled dependency-prefix
boundary`. It is an explicit release-boundary extension beyond the default
three-pass loop.

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

None. Count: 0.

## Milestone closure

The M21 gate at `docs/hld/14-development-backlog.md:1896` remains closed. The
portable source-built gate and the captured signed-source PowerPoint gate still
apply one complete semantic contract before and after save and reopen. The
post-release exact-HEAD run retained Chrome SSIM 0.984628993, Poppler SSIM
0.999557935 for preserved import and 0.998561398 for editable import, and the
one-pixel sensitivity score 0.767265539. The corrected PowerPoint artifact gate
retained static geometry errors of 4, 3, and 2 pixels, movie geometry errors of
4, 4, and 6 pixels, notes normalized size errors of 0.045874, 0.046222, and
0.058054, and handout normalized geometry error 0.045256. All mutation
sensitivities passed.

The Issue 67 correction remains bounded and exact. All 48 authenticated
performance invocations passed at the record HEAD. Their middle-two medians
produced current-to-v0.11.1 ratios of 0.422, 0.382, 0.364, and 0.326, and
current-to-`0582da0` ratios of 0.219, 0.199, 0.179, and 0.180 for the 175 and
700 paragraph native and bundled-fallback paths. These remain below the
declared ceilings in `docs/hld/14-development-backlog.md:3648`.

## Stable release closure

F-X076 is complete at `.claude/plans/F-X076-design.md:3`, and both release
checklist items are complete at `.claude/plans/F-X076-design.md:134`. The
GitHub publication workflow recorded at `docs/sprints/AS_BUILT.md:11181`
completed both crates.io publication and GitHub release jobs successfully.
Independent registry checks downloaded exact version 0.12.0 for `rdocx-opc`,
`rdocx-oxml`, `rdocx-layout`, `rdocx-html`, `rdocx-pdf`, `rdocx`, and
`rdocx-cli`. Every package retains owner `mantissaman (Atul Sharma)`, while
`rdocx-wasm@0.12.0` remains absent as required by
`docs/sprints/AS_BUILT.md:11187`.

The annotated `v0.12.0` tag dereferences to the reviewed SHA recorded at
`docs/sprints/AS_BUILT.md:11179`. The public release body matched the reviewed
changelog render byte for byte at 3,525 bytes with the SHA-256 recorded at
`docs/sprints/AS_BUILT.md:11207`. All seven release-bound notification URLs are
recorded at `docs/sprints/AS_BUILT.md:11222`. PRs 61 through 64 remain closed
and unmerged, Issues 65 and 66 remain closed, and Issue 67 remains open as
required by `docs/sprints/AS_BUILT.md:11219`.

The delivery records now agree. F-X076 is done and ownerless at
`docs/sprints/CURRENT_SPRINT.md:44`, its exact stable-family outcome is part of
the sprint definition of done at `docs/sprints/CURRENT_SPRINT.md:100`, and its
durable evidence is recorded in one AS_BUILT entry. The deterministic hash
harness remains 49 of 49 unchanged.

## Not found

- No interaction defect was found between HTML import, PDF import, the M21
  representative deck, or the Word pagination correction.
- No dependency or layering drift was found. No `oxml-*` crate gained a
  forbidden `rdocx-*` or `rpptx-*` dependency.
- No release-family crossover was found. The exact seven stable packages were
  published at 0.12.0, while shared and PowerPoint packages remained at 0.9.0
  and binding packages remained unpublished.
- No contribution-attribution or record-state drift was found. Every reviewed
  hardened equivalent has specific authenticated credit and exactly one
  release-bound notification.
- No tracker, AS_BUILT, design-plan, HLD, release-note, registry-owner, tag,
  release-body, or package-version inconsistency was found.
- No unexplained deterministic output delta, package-size failure, WASM
  regression, documentation failure, supply-chain failure, duplicate
  implementation, second rendering engine, unplanned public API, GUI action,
  or printer side effect was found.
