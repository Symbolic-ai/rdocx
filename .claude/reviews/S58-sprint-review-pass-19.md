# S58 sprint review, pass 19

**Reviewed**: `sprint/s58` at
`f5f0827874dd551e507c262c3b83bf4c327ce4b4` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 204 files, 23,999 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This nineteenth pass reviews the integrated F-X068 shared 0.8.0 release
preparation. The release exception correctly leaves F-X068 reviewed and in
progress until the separately approved `/release rpptx-v0.8.0` publishes and
verifies the complete family at `docs/sprints/CURRENT_SPRINT.md:47` through
`docs/sprints/CURRENT_SPRINT.md:49`.

## Blocking

None. 0 blocking findings.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this dependency-prefix checkpoint. F-X069, F-X070,
and F-X031 remain pending at `docs/sprints/CURRENT_SPRINT.md:48` through
`docs/sprints/CURRENT_SPRINT.md:50`. The sprint definition also requires the
0.8.0 and 0.11.1 publications, the separately approved yanks, and the final
operational branch-protection step at `docs/sprints/CURRENT_SPRINT.md:83`
through `docs/sprints/CURRENT_SPRINT.md:91`.

The prepared prefix has exact-head evidence. Full verification at
`f5f0827874dd551e507c262c3b83bf4c327ce4b4` is recorded as passed with an
unchanged harness at `.claude/scratch/S58-run.json:568` through
`.claude/scratch/S58-run.json:572`. The observed run passed the complete
workspace, 49 of 49 hashes, no-default layout, both WASM targets, warning-free
docs, 27 README inventories, all 22 package dry runs under 10 MiB, and all
cargo-deny policy groups. This supports the release checkpoint but does not
claim publication or final sprint completion.

## Not found

- **Carrier and publication-family mismatch, 0 findings**: all shared and
  PowerPoint dependency pins are 0.8.0 while the stable workspace remains
  0.11.0 at `Cargo.toml:34` and `Cargo.toml:55` through `Cargo.toml:78`. The
  carrier regression enumerates the exact 15 publishable packages and separate
  unpublished WASM preparation member at
  `scripts/test_sprint_workflow.py:5011` through
  `scripts/test_sprint_workflow.py:5062`.
- **Tag-time dependency deadlock, 0 findings**: the published-shared registry
  proof runs only for stable `v` tags with explicit authority at
  `.github/workflows/publish.yml:26` through
  `.github/workflows/publish.yml:30`. The incubating tag retains the complete
  local-patch archive verification and its own allowlist at
  `.github/workflows/publish.yml:35` through
  `.github/workflows/publish.yml:59` and
  `.github/workflows/publish.yml:78` through
  `.github/workflows/publish.yml:90`. Mutation tests reject a missing
  condition, missing authority, or false authority at
  `scripts/test_sprint_workflow.py:6489` through
  `scripts/test_sprint_workflow.py:6556`.
- **Release-note and contribution drift, 0 findings**: the reviewed notes
  describe the shared direction contract, exact 15-package family, stable
  recovery dependency, unpublished WASM exclusion, and empty selected
  contribution inventory at `CHANGELOG.md:7` through `CHANGELOG.md:52`. The
  notes regression rejects issue or pull-request attribution for this carrier
  release at `scripts/test_sprint_workflow.py:5141` through
  `scripts/test_sprint_workflow.py:5156`.
- **Premature external mutation, 0 findings**: repository metadata keeps
  publish, tag, and push disabled at `Cargo.toml:44` through `Cargo.toml:51`.
  Read-only checks found no local or remote `rpptx-v0.8.0` tag, no GitHub
  release, and crates.io search still reports 0.7.0 as the latest
  `oxml-layout` version. The plan explicitly stops for separate approval at
  `.claude/plans/F-X068-design.md:114` through
  `.claude/plans/F-X068-design.md:121`.
- **HLD, interaction, duplication, layering, dependencies, surface, and
  structure, 0 findings**: exactly the five approved HLD files describe the
  prepared 0.8.0 boundary and defer the registry-only stable consumer until
  publication at `docs/hld/12-testing-strategy.md:1159` through
  `docs/hld/12-testing-strategy.md:1176`. F-X068 changes release carriers and
  gates without adding runtime code, a public API, a dependency, a module, a
  feature flag, or a test binary. Its feature microscope independently reports
  zero defects, smells, and nitpicks at
  `.claude/reviews/F-X068-working-pass-1.md:3` through
  `.claude/reviews/F-X068-working-pass-1.md:17`.
