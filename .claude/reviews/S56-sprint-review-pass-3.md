# S56 sprint review, pass 3

**Reviewed**: `sprint/s56` at
`33ba6d62d96995c3c0c5e05dee7a4ae86d7bf668` against merge base
`92659e7ba3742aab888a8d5603e42560ff3398fc`, 145 files, 27,340 additions,
and 2,700 deletions, including the F-X056 incubating release recovery
preparation
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Dispositions**: 0 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Release recovery audit

The immutable v0.10.0 failure is preserved rather than rewritten. Repository
guidance records that only `rdocx-opc` and `rdocx-oxml` published, that
`rdocx-layout` failed against the old shared registry API, and that no GitHub
release was created (`CLAUDE.md:14`,
`docs/hld/15-build-and-toolchain.md:316`). The existing tag is not moved or
deleted.

F-X056 resolves the registry boundary with one complete incubating-family
preparation. All 15 workspace pins are 0.6.0 (`Cargo.toml:55`), the 15
publishable manifests and `rpptx-wasm` preparation manifest agree, and the
stable workspace remains at 0.10.0. The exact metadata regression checks every
manifest, pin, lock record, README carrier, source assertion, CI literal,
publication flag, family preflight, and selected release-note inventory
(`scripts/test_sprint_workflow.py:4316`).

The publish workflow still has disjoint stable and incubating predicates. Its
incubating path publishes the exact 15 packages in dependency order with bare
verified commands and registry waits (`.github/workflows/publish.yml:72`). The
selected notes contain only the incubating outcomes and the three reviewed
external records. Issue 44, PR 45, and Issue 46 each appear twice and credit
authenticated contributor `@emptinessform` through hardened equivalents
(`CHANGELOG.md:27`, `CHANGELOG.md:51`). Stable-only PRs 47 through 52 are not in
the rendered body.

The clean-tree patched dry run staged exactly 22 archives. All are under 10
MiB. `oxml-layout` retains 20 TTFs and four legal files, `rdocx-layout` has no
font copy, and `rpptx` retains `assets/default.pptx`. Normal dependency trees
contain no new reverse family edge. Both Python crates compile, both WASM
targets compile, the no-default-font path passes, cargo-deny passes, and all 49
deterministic hashes remain unchanged.

## Human action

### H1, publish `rpptx-v0.6.0` only after separate final approval

`.claude/commands/release.md:87`
`.claude/plans/F-X056-design.md:72`

F-X056 is reviewed and remains in progress in both delivery trackers. The next
external mutation must be `/release rpptx-v0.6.0` at the exact reviewed SHA.
That boundary must present the selected package set, rendered notes,
authenticated record inventory, and exact unposted comments, then obtain a new
explicit go or no-go. F-X057 and the stable 0.10.1 recovery remain pending
until the incubating registry family and GitHub release verify.

**Disposition**: human-action after the current-HEAD full verification is
recorded.

## Milestone gate

The M18 technical gate remains satisfied. ODT round-trip and loss diagnostics,
EPUB structure plus the checksum-pinned EPUBCheck oracle, SVG searchable text
plus the calibrated 150 dpi SSIM gate, and ordered-reader preservation all
passed in the integrated workspace. The shared failure-atomic staging helper
remains the sole implementation. F-X056 changes only release metadata and
version carriers, and its hash harness result is unchanged across all 49
entries.

## Not found

- `interaction`: the incubating bump publishes the shared APIs required by the
  already integrated stable features without altering runtime behavior.
- `duplication`: no new implementation path or release allowlist is added.
- `layering`: normal dependency trees retain the documented one-way family
  direction.
- `harness`: no baseline changed and all 49 entries match.
- `gate`: the complete local verify and every release risk rider pass.
- `docs`: all five F-X056 HLD impact files describe current prepared and
  partially published reality.
- `deps`: every internal incubating requirement is 0.6.0 and stable remains
  isolated at 0.10.0.
- `surface`: no runtime API, binding API, crate, module, feature, parser, or
  serializer is introduced by the release preparation.
