# S58 sprint review, pass 9

**Reviewed**: `sprint/s58` at
`20d0d94c063f646eb688c790cf7a7f489ebc425d` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 125 files, 9,628 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This ninth pass is the explicitly authorized review after complete remediation
of pass-8 S1. It audits the corrected evidence ownership and reconfirms the
post-publication dependency prefix rather than repeating pass 8 over an
unchanged tree. Recording the reason here satisfies the later-pass exception
required by `.claude/commands/sprint-review.md:45` and
`.claude/commands/sprint-review.md:86`.

## Blocking

None. 0 blocking findings. Pass-6 B1 remains remediated.

## Should-fix

None. 0 should-fix findings. Pass-8 S1 is fully remediated.

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
`docs/sprints/CURRENT_SPRINT.md:45`. The language, complex-script,
bidirectional, stable publication, and registry gates in the sprint definition
remain open at `docs/sprints/CURRENT_SPRINT.md:66` through
`docs/sprints/CURRENT_SPRINT.md:75`. The published shared 0.7.0 family is a
prerequisite and does not establish final Word acceptance.

The applicable dependency-publication boundary holds. The complete
15-package incubating family, annotated `rpptx-v0.7.0` tag, and reviewed SHA
`1b076c16fb494fe47b054d761e061181a1ea0b15` remain recorded at
`docs/hld/03-architecture.md:523`. Full verification at the exact reviewed
HEAD records all 49 hashes unchanged at
`.claude/scratch/S58-run.json:286`. Independent review confirms all 15
registry entries are live and unyanked under sole owner `mantissaman`,
`rpptx-wasm@0.7.0` is absent from crates.io, the local and remote tag targets
match the reviewed SHA, and the 2,529-byte GitHub release body is byte-identical
to the committed render.

## Not found

- **Pass-8 S1 remediation, 0 findings**: the stable carrier regression claims
  only repository carrier coherence and `rpptx-wasm` publication ineligibility
  at `docs/hld/12-testing-strategy.md:1095`. The paired incubating regression
  uses the same local ownership boundary at
  `docs/hld/12-testing-strategy.md:1100`.
- **External release evidence, 0 findings**: the separate 0.7.0 release gate
  owns all 15 registry entries, sole owner, annotated tag, reviewed SHA,
  stable-family exclusion, byte-identical notes, external absence of
  `rpptx-wasm@0.7.0`, and the empty contribution inventory at
  `docs/hld/12-testing-strategy.md:1121`.
- **Pass-6 B1 remediation, 0 findings**: all five F-X059 HLD impact files still
  state current post-publication reality. Architecture records the complete
  0.7.0 family, exact tag SHA, unpublished `rpptx-wasm`, and separate stable
  0.10.1 family at `docs/hld/03-architecture.md:523` through
  `docs/hld/03-architecture.md:537`.
- **Stable isolation, 0 findings**: the binding contract keeps the incubating
  and stable trains separate and withholds crates.io, npm, and Python authority
  from WASM and binding packages at `docs/hld/10-bindings-spec.md:685` through
  `docs/hld/10-bindings-spec.md:698`.
- **Historical registry proof, 0 findings**: the exact
  `rdocx-layout@0.10.1` registry consumer still requires
  `oxml-layout@0.6.0` and rejects workspace 0.7.0 at
  `docs/hld/12-testing-strategy.md:1106`. The F-X059 story retains the same
  immutable boundary at `docs/hld/14-development-backlog.md:3275`.
- **Build and publication mechanism, 0 findings**: the toolchain HLD lists the
  exact published 0.7.0 family and tag boundary at
  `docs/hld/15-build-and-toolchain.md:252`, keeps `rpptx-wasm` outside
  crates.io at `docs/hld/15-build-and-toolchain.md:262`, and retains the stable
  consumer's immutable 0.6.0 dependency at
  `docs/hld/15-build-and-toolchain.md:361`.
- **Current-state prose, 0 findings**: the five HLD files describe present
  behavior and immutable registry boundaries without remediation or
  change-history prose.
- **Interaction, 0 findings**: F-X058's shared substrate retains the published
  boundary required by F-198 through F-200 at
  `docs/hld/14-development-backlog.md:3270`. F-X060 consumes the verified 0.7.0
  family while leaving stable publication pending at
  `docs/hld/14-development-backlog.md:3287`.
- **Duplication, 0 findings**: local carrier coherence and external release
  verification have distinct owners. The remediation adds no second test,
  registry proof, release path, or delivery ledger.
- **Layering, 0 findings**: the remediation changes one HLD file and adds the
  pass-8 review record. It introduces no dependency edge or ownership change.
- **Harness, 0 findings**: exact-HEAD full verification records 49 of 49
  unchanged at `.claude/scratch/S58-run.json:286`. The remediation changes no
  source, fixture, harness, or baseline.
- **Gate, 0 findings**: both local carrier regressions and the historical
  stable registry regression pass at this checkpoint. The external release
  evidence independently matches the HLD, and M20 remains open rather than
  being inferred from dependency publication.
- **Docs, 0 findings**: all five approved HLD files agree on the published
  0.7.0 tag SHA, stable 0.10.1 isolation, historical 0.6.0 proof, and
  unpublished `rpptx-wasm`. Evidence ownership now matches what each check can
  prove.
- **Deps and surface, 0 findings**: the remediation adds no production
  dependency, public API, binding method, feature flag, module, or package.
