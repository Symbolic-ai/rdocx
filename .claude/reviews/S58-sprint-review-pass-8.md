# S58 sprint review, pass 8

**Reviewed**: `sprint/s58` at
`5baced8535beaa08b3c269b6777cd396e451d27e` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 124 files, 9,491 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This eighth pass is the explicitly authorized review after remediation of
pass-7 S1. It audits the release-evidence ownership correction and reconfirms
the post-publication dependency prefix rather than repeating pass 7 over an
unchanged tree. Recording the reason here satisfies the later-pass exception
required by `.claude/commands/sprint-review.md:45` and
`.claude/commands/sprint-review.md:86`.

## Blocking

None. 0 blocking findings. Pass-6 B1 remains remediated.

## Should-fix

### S1, external unpublished-WASM proof still has no correct owner

`docs/hld/12-testing-strategy.md:1100`

The stable carrier paragraph now correctly limits its claims to repository
carrier coherence and `rpptx-wasm` publication ineligibility at
`docs/hld/12-testing-strategy.md:1095`. The adjacent paired incubating
regression still says it proves that `rpptx-wasm` remains unpublished. That
test reads manifests, pins, lock records, publication flags, README and source
literals, and workflow text at `scripts/test_sprint_workflow.py:4834`. It does
not query crates.io, so it proves ineligibility rather than external absence.

The separate 0.7.0 release-gate paragraph records registry entries, owner,
annotated tag, reviewed SHA, stable exclusion, byte-identical notes, and the
empty notification inventory at `docs/hld/12-testing-strategy.md:1121`, but it
does not record the independently verified absence of
`rpptx-wasm@0.7.0` from crates.io. Complete S1 by changing the paired local
regression to publication ineligibility and assigning the external absence to
the release gate. No runtime, package, tag, release, or ledger change is
needed.

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

The applicable dependency-publication boundary still holds. The complete
15-package incubating family, annotated `rpptx-v0.7.0` tag, and reviewed SHA
`1b076c16fb494fe47b054d761e061181a1ea0b15` remain recorded at
`docs/hld/03-architecture.md:523`. Full verification at the exact reviewed
HEAD records all 49 hashes unchanged at
`.claude/scratch/S58-run.json:273`. S1 concerns attribution of one external
fact, not the verified release result.

## Not found

- **Pass-7 S1 remediation, 0 additional findings**: the stable carrier
  paragraph now says only that repository carriers are 0.7.0 and
  `rpptx-wasm` is publication-ineligible at
  `docs/hld/12-testing-strategy.md:1095`. The remaining ownership mismatch is
  fully captured by S1.
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
  boundary at `docs/hld/14-development-backlog.md:3275`.
- **Release evidence, 0 additional findings**: the 0.7.0 release gate owns the
  exact 15 entries, sole owner, tag, reviewed SHA, stable exclusion,
  byte-identical body, and empty notification inventory at
  `docs/hld/12-testing-strategy.md:1121`. S1 only adds the already verified
  external unpublished-WASM fact to that owner.
- **Build and publication mechanism, 0 findings**: the toolchain HLD lists the
  exact published 0.7.0 family and tag boundary at
  `docs/hld/15-build-and-toolchain.md:252`, keeps `rpptx-wasm` outside
  crates.io at `docs/hld/15-build-and-toolchain.md:262`, and retains the stable
  consumer's immutable 0.6.0 dependency at
  `docs/hld/15-build-and-toolchain.md:361`.
- **Current-state prose, 0 findings**: the five HLD files describe present
  behavior and immutable registry boundaries without adding remediation or
  change-history prose.
- **Interaction, 0 findings**: F-X058's shared substrate retains the published
  boundary required by F-198 through F-200 at
  `docs/hld/14-development-backlog.md:3270`. F-X060 still consumes 0.7.0 while
  leaving stable publication pending at
  `docs/hld/14-development-backlog.md:3287`.
- **Duplication, 0 findings**: the carrier regressions and external release gate
  remain distinct evidence sources. Completing S1 requires only placing each
  assertion with its existing owner, not adding another test or release path.
- **Layering, 0 findings**: the remediation changes one HLD file and adds the
  pass-7 review record. It introduces no dependency edge or ownership change.
- **Harness, 0 findings**: exact-HEAD full verification records 49 of 49
  unchanged at `.claude/scratch/S58-run.json:273`. The remediation changes no
  source, fixture, harness, or baseline.
- **Gate, 0 additional findings**: the local stable carrier now claims only
  facts it tests, and the recorded full gate passes at the exact HEAD. M20
  remains open rather than being inferred from dependency publication.
- **Docs, 0 additional findings**: B1 remains fixed across all five HLD files.
  The only documentation issue is S1's remaining external-evidence ownership.
- **Deps and surface, 0 findings**: the remediation adds no production
  dependency, public API, binding method, feature flag, module, or package.
