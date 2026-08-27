# S58 sprint review, pass 6

**Reviewed**: `sprint/s58` at
`9a02d8371c495ba84607be57ffb9cf6d62edd57e` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 122 files, 9,193 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This sixth pass is the explicitly authorized post-publication review after the
F-X059 release gate and completion ledgers. It audits a new external release
and tracked completion delta rather than repeating pass 5 over an unchanged
state. Recording the reason here satisfies the later-pass exception required
by `.claude/commands/sprint-review.md:45` and
`.claude/commands/sprint-review.md:86`.

## Blocking

### B1, the HLD still describes 0.7.0 as unpublished

`docs/hld/03-architecture.md:523`

All five HLD files in F-X059's approved impact list still describe the 0.7.0
incubating family as prepared and 0.6.0 as the latest published family. The
same stale state appears in the binding contract at
`docs/hld/10-bindings-spec.md:685`, the release-regression description at
`docs/hld/12-testing-strategy.md:1095`, the F-X059 story at
`docs/hld/14-development-backlog.md:3268`, and both publication sections at
`docs/hld/15-build-and-toolchain.md:252` and
`docs/hld/15-build-and-toolchain.md:362`.

That now contradicts the completed release record, which says all 15 packages
were published from reviewed SHA
`1b076c16fb494fe47b054d761e061181a1ea0b15` at
`docs/sprints/AS_BUILT.md:9605`, and it misstates the registry boundary that
F-198 through F-200 are allowed to consume. The fix must update exactly the
five plan-listed HLD files to current post-release reality. It must name 0.7.0
as the latest complete incubating publication from the immutable
`rpptx-v0.7.0` tag at the reviewed SHA, retain stable 0.10.1 isolation, retain
the historical `rdocx-layout@0.10.1` to `oxml-layout@0.6.0` registry proof,
keep `rpptx-wasm` unpublished, and record the verified release gate without
turning the spec set into a change history.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this post-publication dependency-prefix checkpoint.
F-198 is in progress, F-199 and F-200 remain pending, and stable release
F-X060 remains pending at `docs/sprints/CURRENT_SPRINT.md:41` through
`docs/sprints/CURRENT_SPRINT.md:45`. The sprint definition still requires the
later language, complex-script, bidirectional, stable publication, and
registry gates at `docs/sprints/CURRENT_SPRINT.md:66` through
`docs/sprints/CURRENT_SPRINT.md:75`. Publishing the shared 0.7.0 prerequisite
does not establish those Word acceptance outcomes.

The external F-X059 release gate itself holds. The annotated local and remote
`rpptx-v0.7.0` tag dereferences to reviewed SHA
`1b076c16fb494fe47b054d761e061181a1ea0b15`. GitHub Actions run 33049354630
completed the output, metadata, notes, archive, exact 15-package incubating
publication, and GitHub Release jobs while skipping stable publication, which
matches the tracked evidence at `docs/sprints/AS_BUILT.md:9611`. All 15
registry entries resolve at 0.7.0, are unyanked, and have sole owner
`mantissaman`. `rpptx-wasm@0.7.0` is absent. The 2,529-byte GitHub release body
is byte-identical to the committed changelog render. The completed dependency
prefix is nevertheless not clean until B1 reconciles the authoritative HLD
with that verified state.

## Not found

- **Publication evidence, 0 findings**: the release run, exact tag target,
  selected 15-package registry set, owner inventory, release body, and stable
  exclusion independently agree with the release record at
  `docs/sprints/AS_BUILT.md:9611` through
  `docs/sprints/AS_BUILT.md:9619`.
- **Completion ledgers, 0 findings**: F-X059 is done with no owner in the
  active sprint at `docs/sprints/CURRENT_SPRINT.md:37`, done in the backlog at
  `docs/sprints/BACKLOG.md:514`, and recorded once with matching size and
  actuals at `docs/sprints/SPRINT_TRACKER.md:337`. The design plan is completed
  with every release checklist item ticked at
  `.claude/plans/F-X059-design.md:113`.
- **Contribution inventory and notifications, 0 findings**: the release
  inventory contains no selected authenticated external record and therefore
  requires no comment at `docs/sprints/AS_BUILT.md:9621` through
  `docs/sprints/AS_BUILT.md:9627`. PRs 55 through 57 remain open and reserved
  for F-X064 through F-X066, so their exclusion from F-X059 is consistent.
- **Interaction, 0 additional findings**: F-X058's multilingual runtime
  substrate and F-X059's published version family remain coherent. Stable Word
  consumers do not opt into that path yet, as recorded at `CHANGELOG.md:50`.
  The only post-publication interaction defect is the stale HLD state in B1.
- **Duplication, 0 findings**: the exact incubating family still has one
  canonical carrier regression at `scripts/test_sprint_workflow.py:4834` and
  one matching publication allowlist at `.github/workflows/publish.yml:72`.
  No second release path or completion ledger was introduced.
- **Layering, 0 findings**: the publication and completion commits add no
  runtime dependency edge. The full sprint delta retains the format-neutral
  shared ownership boundary described at `docs/hld/03-architecture.md:122`,
  apart from the version-state defect recorded in B1.
- **Harness, 0 findings**: full verification at the release SHA records all 49
  hashes unchanged at `.claude/scratch/S58-run.json:241`, and the completed
  release record agrees at `docs/sprints/AS_BUILT.md:9634`. The publication and
  completion commits change no harness script or baseline.
- **Gate, 0 additional findings**: the seven focused stable carrier,
  incubating carrier, immutable registry, release-note, and workflow-routing
  regressions pass at this checkpoint. Release-note validation also passes.
  The M20 gate remains open rather than being inferred from the successful
  dependency publication.
- **Docs, 0 additional findings**: the HLD mismatch is fully captured by B1.
  The AS_BUILT entry accurately distinguishes the published runtime family,
  unpublished `rpptx-wasm`, empty contribution inventory, and later stable
  consumers at `docs/sprints/AS_BUILT.md:9605` through
  `docs/sprints/AS_BUILT.md:9644`.
- **Deps, 0 findings**: all 15 incubating workspace pins remain 0.7.0 across
  `Cargo.toml:55` through `Cargo.toml:70`, and the separate stable carriers
  remain 0.10.1 at `Cargo.toml:71` through `Cargo.toml:78`. The publication
  finalization adds no production dependency.
- **Surface, 0 findings**: the post-publication delta adds no Rust, binding,
  WASM, CLI, or authoring API. `rpptx-wasm` remains preparation-only and
  unpublished, as the release record confirms at
  `docs/sprints/AS_BUILT.md:9609`.
- **Package, legal, font, and assets, 0 findings**: the successful publication
  workflow repeated archive verification before the real incubating allowlist.
  The tracked package contract still includes all deterministic fonts, licence,
  notice, and subset provenance files at
  `crates/oxml-layout/Cargo.toml:13`, while `rpptx` retains its packaged default
  presentation asset.
