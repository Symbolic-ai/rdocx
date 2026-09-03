# S64 sprint review, pass 8

**Reviewed**: clean `sprint/s64` at
`65c707df29cd9ecdfb8a5f1c4e2e090aabb246d8` against
`0582da0a38886f5ceeb65ab9afcd0797f6fa14b0`, 106 files, 20,061 changed
lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

Pass 8 is the explicitly scheduled stable-release boundary after the pass-7
clean dependency-prefix boundary. It does not reopen the default three-pass
loop.

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

None. Count: 0.

## Milestone closure

The M21 gate at `docs/hld/14-development-backlog.md:1896` remains closed. One
shared assertion applies the complete package, collaboration, section, media,
playback, timing, signature, slide, and SmartArt contract to original and
save-reopened bytes. The portable source-built regression is at
`crates/rpptx/tests/integration.rs:9428`, and the exact captured signed-source
PowerPoint gate is at `crates/rpptx/tests/integration.rs:9981`. The latter pins
PowerPoint 16.104, the no-repair source, static PDF, movie, notes PDF, and
handout PDF. It covers all three static pages, three aligned movie samples,
three notes pages, and the three-up handout with real geometry, text, and paint
mutation sensitivity.

The final exact-HEAD run retained the reviewed oracle values. Chrome 152 scored
0.984628993. Poppler 26.01.0 scored 0.999557935 for preserved import and
0.998561398 for editable import, while the isolated one-pixel mutation scored
0.767265539 and failed. The PowerPoint gate retained static geometry errors of
4, 3, and 2 pixels, movie geometry errors of 4, 4, and 6 pixels, notes
normalized size errors of 0.045874, 0.046222, and 0.058054, and handout
normalized geometry error of 0.045256. The deterministic hash harness remains
49 of 49 unchanged.

The Issue 67 correction also remains closed. The exact current F-X075
production and harness manifests are pinned at
`crates/rdocx/tests/regression_test.rs:365` and
`crates/rdocx/tests/regression_test.rs:216`. The authenticated 48-run rider at
this reviewed HEAD remained below both approved ratio ceilings for 175 and 700
paragraphs in native and bundled-font modes.

## Stable release boundary

F-X076 prepares exactly seven stable packages at 0.12.0. The selected package
set is `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`, `rdocx-pdf`,
`rdocx`, and `rdocx-cli`, as specified at
`.claude/plans/F-X076-design.md:54`. Shared and PowerPoint packages remain at
their independently published 0.9.0 boundary. Python, WASM, npm, and PyPI
publication remain excluded.

The changelog identifies the same stable family at `CHANGELOG.md:46`, and the
publish workflow retains dependency order from `rdocx-opc` through
`rdocx-cli` at `.github/workflows/publish.yml:64`. Release regressions cover
the stable carriers, registry-only shared dependency proof, deterministic
notes, seven-record contribution inventory, owner verification, exclusions,
and leave-state notifications. Package preparation verified all 22 workspace
archives and kept the largest below 10 MiB.

The release gate is deliberately incomplete. Its two remaining checklist items
at `.claude/plans/F-X076-design.md:134` require a new exact-SHA approval, real
publication, independent registry and owner verification, byte-identical
release notes, and seven reviewed contribution comments. F-X076 therefore
correctly remains in progress at `docs/sprints/CURRENT_SPRINT.md:44`. This clean
review permits the separate release approval request. It does not authorize
publication or sprint closure by itself.

## Not found

- No interaction defect was found between HTML import, PDF import, the combined
  M21 representative deck, or the independent Word pagination correction.
- No dependency or layering drift was found. No `oxml-*` crate gained a
  forbidden `rdocx-*` or `rpptx-*` dependency.
- No unplanned public API, crate, feature, trait, generic, binding publication,
  or package-family crossover was found.
- No HLD scope drift was found. F-X076 changes exactly the five HLD files listed
  at `.claude/plans/F-X076-design.md:93`, and the documents describe current
  prepared intent rather than publication history.
- No release-carrier mismatch was found. Workspace pins, lock records, READMEs,
  CI assertions, binding metadata, the changelog, release-notes renderer, and
  workflow tests agree on stable 0.12.0 and shared 0.9.0.
- No contribution-attribution drift was found. PRs 61 through 64 remain
  attributed to `@pedroassumpcao`, Issues 65 through 67 remain attributed to
  `@emptinessform`, and every outcome is classified as a hardened equivalent
  with record state preserved.
- No deterministic output, package-size, WASM, documentation, supply-chain, or
  registry-proof regression was found in the final exact-HEAD verification.
- No duplicate implementation, second rendering engine, fixed normal-test
  oracle path, GUI or printer side effect, panic-based acceptance path, or
  cleanup-based false positive was found in the integrated sprint delta.
