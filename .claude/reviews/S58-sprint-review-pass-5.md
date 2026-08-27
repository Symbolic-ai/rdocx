# S58 sprint review, pass 5

**Reviewed**: `sprint/s58` at
`9da5414c8bd3a3959dc2d7eedd21da7e5fcb0000` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 121 files, 8,999 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This fifth pass is the explicitly authorized review after F-X059 integration.
It audits a new release-carrier, registry-proof, release-note, HLD, and package
delta rather than repeating pass 4 over an unchanged tree. Recording the
reason here satisfies the later-pass exception required by
`.claude/commands/sprint-review.md:45` and
`.claude/commands/sprint-review.md:86`.

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

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It does not
yet hold at this scheduled dependency-prefix checkpoint. F-198 is in progress,
F-199 and F-200 remain pending, and the stable release F-X060 remains pending
at `docs/sprints/CURRENT_SPRINT.md:41` through
`docs/sprints/CURRENT_SPRINT.md:45`. The sprint definition also requires both
the incubating 0.7.0 and stable 0.11.0 families to be published from their
separately approved reviewed SHAs at `docs/sprints/CURRENT_SPRINT.md:66`
through `docs/sprints/CURRENT_SPRINT.md:75`. F-X059 is only reviewed and still
in progress at `docs/sprints/CURRENT_SPRINT.md:37`, so this prefix makes no
publication, registry-owner, tag, notification, M20 completion, or sprint
closure claim.

The applicable dependency-prefix gate holds at the reviewed HEAD. Sprint state
records F-X059's reviewed implementation Head and integration commit at
`.claude/scratch/S58-run.json:64`, followed by a passing full verification at
the exact canonical SHA with all 49 hashes unchanged at
`.claude/scratch/S58-run.json:228`. F-X059's microscope found zero defects,
smells, and nitpicks at `.claude/reviews/F-X059-working-pass-1.md:1`. The
focused stable carrier, incubating carrier, immutable registry, release-note,
and publication-routing regressions also pass in this review.

## Not found

- **Interaction, 0 findings**: F-X058 owns the complete shared multilingual
  runtime substrate and explicitly leaves stable Word integration to later
  stories at `CHANGELOG.md:11` and `CHANGELOG.md:50`. F-X059 changes its exact
  incubating carriers and release evidence without changing runtime behavior.
  The prepared family therefore exposes F-X058 coherently while the separate
  stable Word family remains on its reviewed 0.10.1 boundary.
- **Duplication, 0 findings**: the release contract has one canonical exact
  15-package family regression at `scripts/test_sprint_workflow.py:4834` and
  one matching workflow allowlist at `.github/workflows/publish.yml:72`.
  Release notes, carrier assertions, and the workflow consume that contract
  without adding a second publication path, release ledger, or version owner.
- **Layering, 0 findings**: the F-X059 delta changes release carriers and
  documentation only. It adds no runtime dependency edge, and the complete
  sprint delta preserves the architecture rule that the shared 0.7.0 family
  owns format-neutral behavior while the stable family stays separate, as
  documented at `docs/hld/03-architecture.md:523`.
- **Harness, 0 findings**: the exact-HEAD full verification records
  `unchanged, 49 of 49` at `.claude/scratch/S58-run.json:228`. The F-X059
  microscope independently records the unchanged 49-entry deterministic
  harness at `.claude/reviews/F-X059-working-pass-1.md:58`. No harness script
  or baseline is changed by the F-X059 integration.
- **Gate, 0 findings**: the carrier tests require all manifests, workspace
  pins, lock records, publication flags, README literals, Rust assertions, the
  CI literal, and the exact dependency-ordered preflight at
  `scripts/test_sprint_workflow.py:4834`. The immutable registry test uses an
  isolated exact `rdocx-layout@0.10.1` consumer at
  `scripts/test_sprint_workflow.py:4616` and requires published
  `oxml-layout@0.6.0` rather than current workspace 0.7.0 at
  `scripts/test_sprint_workflow.py:4662`. Both checks pass.
- **Docs, 0 findings**: F-X059 changes exactly the five HLD files listed by its
  approved plan at `.claude/plans/F-X059-design.md:88`. The HLD consistently
  distinguishes the prepared 0.7.0 family, the last published 0.6.0 family,
  the published stable 0.10.1 family, and the unpublished `rpptx-wasm`
  preparation at `docs/hld/03-architecture.md:523`,
  `docs/hld/10-bindings-spec.md:685`, and
  `docs/hld/12-testing-strategy.md:1095`. The prose describes current intent
  and does not turn the preparation into release history.
- **Deps, 0 findings**: the exact 15 incubating workspace pins named by the
  carrier regression are 0.7.0 across `Cargo.toml:55` through
  `Cargo.toml:70`, while the separate stable workspace carriers remain 0.10.1
  at `Cargo.toml:71` through `Cargo.toml:78`. The published
  stable dependency proof is independent of those workspace pins, and F-X059
  adds no production package.
- **Surface, 0 findings**: F-X059 adds no public Rust type, function, trait,
  module, feature flag, binding method, or authoring surface. The binding HLD
  keeps both WASM crates unpublished and withholds npm and Python authority at
  `docs/hld/10-bindings-spec.md:692` through
  `docs/hld/10-bindings-spec.md:699`.
- **Release contract, 0 findings**: the real workflow uses separate namespace
  predicates and exact allowlists for the seven stable packages at
  `.github/workflows/publish.yml:55` and the 15 incubating packages in
  dependency order at `.github/workflows/publish.yml:72`. Its preflight runs
  the stable carrier, immutable registry, incubating carrier, reviewed-note,
  hash, and package checks at `.github/workflows/publish.yml:20` through
  `.github/workflows/publish.yml:53`. F-X059 does not create a tag or grant
  publication authority before the separate final release approval.
- **Release notes and inventory, 0 findings**: the prepared
  `rpptx-v0.7.0` notes describe only F-X058's shared multilingual substrate at
  `CHANGELOG.md:7` through `CHANGELOG.md:40`. They state the exact 15-package
  minor boundary, stable 0.10.1 isolation, deferred stable Word acceptance,
  and unpublished `rpptx-wasm` status at `CHANGELOG.md:42` through
  `CHANGELOG.md:54`. The contributor inventory records no new authenticated
  external issue or pull-request record and therefore no notification at
  `CHANGELOG.md:56`. The deterministic release-note check and focused notes
  regression pass.
- **Package, legal, font, and assets, 0 findings**: `oxml-layout` packages its
  source, deterministic TTF files, licences, notices, and subset provenance at
  `crates/oxml-layout/Cargo.toml:13`. The family-to-licence inventory assertion
  covers every bundled family at
  `crates/oxml-layout/src/bundled_fonts.rs:135`. Noto source hashes and the
  subset record are explicit at `crates/oxml-layout/fonts/NOTICE-Noto:7` and
  `crates/oxml-layout/fonts/SUBSET-NotoSansSC.md:3`. The clean F-X059
  microscope records the patched 22-package dry run, 10 MiB ceiling, complete
  24-font legal inventory, default presentation asset, and both WASM checks at
  `.claude/reviews/F-X059-working-pass-1.md:47`.
- **Ledgers, 0 findings**: completed F-X058 is recorded as done and reviewed
  F-X059 remains in progress at `docs/sprints/CURRENT_SPRINT.md:36` through
  `docs/sprints/CURRENT_SPRINT.md:37`. The backlog agrees at
  `docs/sprints/BACKLOG.md:513` through `docs/sprints/BACKLOG.md:515`. F-X059
  correctly has no completed delivery entry before the real release gate, so
  the sprint records do not overstate publication or M20 completion.
