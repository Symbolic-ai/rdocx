# F-X036, all, pass 1

**Reviewed**: the complete 23-file working diff, 241 changed lines, against the approved F-X036 plan and its release, release-note, HLD, version-carrier, package, binding, README, CI, and workflow contracts
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the release notes omit the native collaboration tranche

`CHANGELOG.md:17`

The approved release plan requires the `v0.8.0` notes to cover the completed
native Word collaboration work at `.claude/plans/F-X036-design.md:54`, but the
Added section runs from fields through ordered body traversal without naming
the release's comment threads, content-control binding, bookmarks and
cross-references, revision inspection and resolution, tracked revision
rendering, or document-protection APIs. Those are substantial user-visible
additions in the selected `v0.7.0..HEAD` evidence range, including the comment
facade at `docs/sprints/AS_BUILT.md:7053`, content-control binding at
`docs/sprints/AS_BUILT.md:7119`, bookmarks at
`docs/sprints/AS_BUILT.md:7153`, revision resolution at
`docs/sprints/AS_BUILT.md:7239`, tracked rendering at
`docs/sprints/AS_BUILT.md:7286`, and protection metadata at
`docs/sprints/AS_BUILT.md:7330`. The generic phrase "native document
automation" in the highlight does not tell users that these capabilities
shipped. Add meaningful collaboration coverage before publishing the rendered
notes unchanged as the GitHub release body.

## Smells

None.

## Nitpicks

None.

## Not found

- Version carriers and semver: `[workspace.package]`, all nine internal pins,
  all eleven inherited manifest and lockfile packages, both Python project
  versions, both rdocx WASM dependency literals, and the stable CI package
  literal are coherently prepared at 0.8.0. All explicit incubating manifests
  remain 0.4.0.
- Publication scope: metadata reports exactly seven publishable stable crates.
  `rdocx-wasm`, `rdocx-py`, `rpptx-py`, and `oxml-py-support` remain
  unpublished, and the stable predicate still selects the exact dependency
  ordered seven-package allowlist at `.github/workflows/publish.yml:55`.
- Release boundaries: the exact `v0.8.0` tag is absent locally and from
  `origin`, and all seven selected 0.8.0 registry versions are absent. No tag,
  push, publication, package-family expansion, or release mutation is present
  in the working diff.
- Changelog fidelity outside D1: the field, chart, template, mail merge,
  comparison, watermark, complete-layout, provenance, relayout-cache,
  ordered-body, compatibility, and migration claims are supported by the
  reviewed range. Pedro Assumpcao and `@emptinessform` receive the plan-required
  contributor and issue-reporter credit at `CHANGELOG.md:144`.
- HLD prepared-state truth: exactly the four plan-listed HLD files change.
  They distinguish prepared 0.8.0 metadata from the immutable published 0.7.0
  boundary and retain the separate final approval at
  `docs/hld/03-architecture.md:413`,
  `docs/hld/10-bindings-spec.md:337`, and
  `docs/hld/15-build-and-toolchain.md:264`.
- README and workflow assertions: all seven stable README requirements and
  their central gate literals use 0.8.0. The stable metadata regression checks
  the exact carrier, pin, publication, Python, WASM, CI, README, and incubating
  state at `scripts/test_sprint_workflow.py:3902`. Existing mutation tests
  continue to protect publication predicates, package inventory, failure
  propagation, preflight ordering, and reviewed-note publication.
- Focused checks: `cargo metadata --no-deps`, release-note check and render,
  all 63 sprint-workflow tests, prose, generated-skill drift, diff checking,
  and the focused stable plus incubating publication preflights pass. The
  working diff contains no source logic, dependency addition, panic path,
  OOXML-order change, unmodelled-XML mutation, or structural-rule regression.
