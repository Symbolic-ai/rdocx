# S06 sprint review, pass 2

**Reviewed**: `sprint/s06` against
`22198fb28388c242343215c670050b31912d5299`, 49 files, 3,777 changed text
lines plus 20 TTF binaries, crates: `oxml-layout`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M4 gate is "hash harness unchanged. This is the milestone where that
matters most" at `docs/hld/14-development-backlog.md:257`. It holds for the S06
delta. An independent `python3 scripts/hash_harness.py --check` run at
`b8e9aaf0fadf0f1c799e5824c9772918035111bd` regenerated the deterministic
samples and reported all 28 entries matching.

The integrated workspace gate also passed. An independent
`cargo test --workspace --all-features --exclude rdocx-py --exclude rpptx-py`
run completed successfully. The excluded binding packages are not yet workspace
members, so Cargo reported that fact as a warning.

The story gates hold. `cargo test -p oxml-layout --all-features` passed 32 tests,
and `cargo test -p oxml-layout --no-default-features` passed 33 tests. The
no-default run includes the host-discovery isolation assertion backed by the
feature guard at `crates/oxml-layout/src/font.rs:150`. The owned line contract
and explicit no-wrap regression are exercised at
`crates/oxml-layout/src/line.rs:906` and
`crates/oxml-layout/src/line.rs:942`. The hand-computed PDF composition gate is
at `crates/oxml-layout/src/transform.rs:161`.

The package list contains all 20 TTF files, the three licence files, and
`NOTICE-Caladea`. The reviewed archive is 3,591,409 bytes, below 10 MiB.

The sprint is not the end of M4 because F-032 through F-036 remain pending. The
end-of-milestone hash condition nevertheless holds at this checkpoint.

## Not found

- Interaction: the copied font and output foundation, owned line boundary, and
  affine transform share only the intended crate-local types. Their combined
  default and no-default test paths pass.
- Duplication: no same-purpose helper was introduced by separate S06 stories.
  The staged copies are the migration step required by F-029 and F-030.
- Layering: `cargo tree -p oxml-layout --edges normal` contains no `rdocx-*` or
  `rpptx*` dependency. The released rdocx source and manifests have no S06
  delta.
- Harness: all three AS_BUILT entries declare an unchanged 28-entry result,
  consistent with the independent integrated run.
- Gate: the M4 hash condition and the three story gates have direct passing
  test evidence.
- Docs: pass-1 S1 is resolved. `docs/hld/12-testing-strategy.md:145` and
  `docs/hld/15-build-and-toolchain.md:53` now agree that no-default mode disables
  host discovery while bundled deterministic fonts remain available.
- Deps: the new manifest dependencies each serve the copied error, font,
  shaping, parsing, or line-breaking implementation. The lockfile adds only the
  `oxml-layout` package stanza.
- Surface: the public exports match the copied F-029 surface, the concrete
  F-030 types and line breaker, and the exact six-operation F-031 transform
  contract.
- Publication: `crates/oxml-layout/Cargo.toml:4` keeps version 0.0.0 and
  `crates/oxml-layout/Cargo.toml:11` disables publication. The release workflow
  still names only the seven released rdocx packages.
