# S58 sprint review, pass 7

**Reviewed**: `sprint/s58` at
`f3f56c8688a2f4ee7e831cd3e48e5e10de9b742c` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 123 files, 9,353 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This seventh pass is the explicitly authorized review after remediation of
pass-6 B1. It audits the five-file HLD correction and its dependency-prefix
interactions rather than repeating pass 6 over an unchanged tree. Recording
the reason here satisfies the later-pass exception required by
`.claude/commands/sprint-review.md:45` and
`.claude/commands/sprint-review.md:86`.

## Blocking

None. 0 blocking findings. Pass-6 B1 is remediated.

## Should-fix

### S1, the carrier regression is credited with external registry proof

`docs/hld/12-testing-strategy.md:1095`

The testing strategy says the local stable carrier regression proves both that
the current incubating workspace and the published registry family are 0.7.0
and that `rpptx-wasm` remains unpublished. The named regression only reads
workspace manifests, pins, lock records, README requirements, source literals,
publication flags, and workflow text at
`scripts/test_sprint_workflow.py:4442`. It does not query crates.io, so it can
prove 0.7.0 carrier coherence and `rpptx-wasm` publication ineligibility, but
not the external publication state.

The actual registry proof is correctly recorded in the separate 0.7.0 release
gate at `docs/hld/12-testing-strategy.md:1121` and in the release evidence at
`docs/sprints/AS_BUILT.md:9611`. Reword the carrier-regression sentence to name
only repository metadata and eligibility. Leave the external 15-package,
owner, tag, body, and unpublished-WASM facts with the separate release
evidence. This does not reopen pass-6 B1 or change the verified release
boundary.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this dependency-prefix checkpoint. F-198 is in
progress, F-199 and F-200 remain pending, and stable release F-X060 remains
pending at `docs/sprints/CURRENT_SPRINT.md:41` through
`docs/sprints/CURRENT_SPRINT.md:45`. The later language, complex-script,
bidirectional, stable publication, and registry gates in the sprint definition
remain open at `docs/sprints/CURRENT_SPRINT.md:66` through
`docs/sprints/CURRENT_SPRINT.md:75`. The completed shared 0.7.0 publication is
a prerequisite and does not establish those Word acceptance outcomes.

The applicable publication boundary holds. The complete 15-package
incubating family is recorded at 0.7.0 from the annotated
`rpptx-v0.7.0` tag and reviewed SHA
`1b076c16fb494fe47b054d761e061181a1ea0b15` at
`docs/hld/03-architecture.md:523`. The exact-HEAD full verification is recorded
with all 49 hashes unchanged at `.claude/scratch/S58-run.json:260`. The
should-fix finding concerns which check owns one assertion, not the verified
external result.

## Not found

- **Pass-6 B1 remediation, 0 findings**: exactly the five HLD files listed by
  `.claude/plans/F-X059-design.md:88` changed after pass 6. Architecture names
  the complete published family, tag, reviewed SHA, unpublished `rpptx-wasm`,
  and separate stable family at `docs/hld/03-architecture.md:523` through
  `docs/hld/03-architecture.md:537`.
- **Bindings and stable isolation, 0 findings**: the binding contract records
  the incubating 0.7.0 and stable 0.10.1 trains separately and leaves all WASM,
  npm, and Python publication unauthorized at
  `docs/hld/10-bindings-spec.md:685` through
  `docs/hld/10-bindings-spec.md:698`.
- **Historical registry proof, 0 findings**: the isolated exact
  `rdocx-layout@0.10.1` consumer still requires registry
  `oxml-layout@0.6.0` and rejects the current workspace 0.7.0 family at
  `docs/hld/12-testing-strategy.md:1106`. The F-X059 story repeats that
  boundary at `docs/hld/14-development-backlog.md:3275`.
- **Release gate, 0 additional findings**: the testing strategy records the
  exact 15 entries, sole owner, annotated tag and reviewed SHA, stable-family
  exclusion, byte-identical body, and empty notification inventory at
  `docs/hld/12-testing-strategy.md:1121`. S1 only corrects attribution of the
  preceding local regression.
- **Build and publication mechanism, 0 findings**: the toolchain HLD lists the
  exact 15 packages and their published 0.7.0 tag boundary at
  `docs/hld/15-build-and-toolchain.md:252`, keeps `rpptx-wasm` outside
  crates.io at `docs/hld/15-build-and-toolchain.md:262`, and retains the stable
  consumer's immutable 0.6.0 dependency at
  `docs/hld/15-build-and-toolchain.md:361`.
- **Current-state prose, 0 findings**: the five HLD updates state what is true
  now. They contain no remediation narrative, F-ID change log, or claim that
  the release moved an existing version or tag. Historical versions remain
  only where they explain immutable registry boundaries.
- **Interaction, 0 findings**: F-X058's shared multilingual substrate now has
  the published registry boundary required by F-198 through F-200 at
  `docs/hld/14-development-backlog.md:3270`. F-X060 consumes that verified
  0.7.0 family while leaving stable publication pending at
  `docs/hld/14-development-backlog.md:3287`.
- **Duplication, 0 findings**: the HLD correction adds no second version owner,
  release path, registry proof, or delivery ledger. The carrier regression and
  external release gate remain distinct checks with distinct purposes after
  S1's wording correction.
- **Layering, 0 findings**: the remediation changes documentation only and
  introduces no dependency edge. The format-neutral shared ownership boundary
  remains unchanged at `docs/hld/03-architecture.md:122`.
- **Harness, 0 findings**: full verification at the exact reviewed HEAD records
  49 of 49 unchanged at `.claude/scratch/S58-run.json:260`. The remediation
  changes no source, fixture, hash harness, or baseline.
- **Gate, 0 additional findings**: the three focused carrier and historical
  registry regressions pass at this checkpoint, and the full gate is recorded
  at the exact HEAD. M20 remains open rather than being inferred from the
  dependency publication.
- **Docs, 0 additional findings**: all five approved HLD files agree on the
  published 0.7.0 tag SHA, stable 0.10.1 isolation, historical 0.6.0 proof, and
  unpublished `rpptx-wasm`. The only remaining documentation issue is S1's
  test-proof attribution.
- **Deps and surface, 0 findings**: the remediation adds no production
  dependency, public API, binding method, feature flag, module, or package.
  Stable Word opt-in and final oracle acceptance remain with later stories.
