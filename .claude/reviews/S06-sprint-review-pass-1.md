# S06 sprint review, pass 1

**Reviewed**: `sprint/s06` against
`22198fb28388c242343215c670050b31912d5299`, 46 files, 3,695 changed text
lines plus 20 TTF binaries, crates: `oxml-layout`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, the no-default-features HLD contract contradicts the staged crate
`docs/hld/15-build-and-toolchain.md:53`

The toolchain specification says deterministic construction fails without
bundled fonts, and `docs/hld/12-testing-strategy.md:146` calls the same build
the bundled-fonts-off path. S06 instead keeps all bundled fonts available and
uses `--no-default-features` to disable only system font discovery, as defined
by `crates/oxml-layout/Cargo.toml:19`. This is the intended and tested S06
boundary, but the HLD now gives future WASM and CI work the opposite
instruction. Update both passages to say that bundled deterministic fonts
remain available while host font discovery is disabled.

## Nice-to-have

None.

## Milestone gate

The M4 gate is "hash harness unchanged. This is the milestone where that
matters most" at `docs/hld/14-development-backlog.md:257`. It holds for the S06
delta. An independent `python3 scripts/hash_harness.py --check` run at the
reviewed head regenerated the deterministic samples and reported all 28 entries
matching.

The story gates also hold. `cargo test -p oxml-layout --all-features` passed 32
tests, and `cargo test -p oxml-layout --no-default-features` passed 33 tests.
The latter includes the system-discovery isolation regression. The affine
composition test at `crates/oxml-layout/src/transform.rs:161` uses fully nonzero
hand-computed matrices and proves self-first PDF `cm` order. The package list
contains all 20 TTF files, three licence files, and `NOTICE-Caladea`. The
reviewed archive is 3,591,409 bytes, below 10 MiB.

The sprint is not the end of M4 because F-032 through F-036 remain pending.
The end-of-milestone hash condition nevertheless holds at this checkpoint.

## Not found

- Interaction: the F-029 font and output foundation, F-030 owned line boundary,
  and F-031 transform module compose without conflicting types, features, or
  exports.
- Duplication: no same-purpose helper was added by separate S06 stories. The
  staged copies of released layout modules are the migration strategy required
  by F-029 and F-030.
- Layering: `cargo tree -p oxml-layout --edges normal` contains no `rdocx-*` or
  `rpptx*` dependency. No released crate source or manifest changed.
- Harness: every S06 AS_BUILT entry declares the same unchanged 28-entry result,
  which matches the independent integrated run.
- Docs: apart from S1, the sprint delta matches the architecture, rendering,
  migration, backlog, risk, and publication boundaries.
- Deps: the lockfile adds only the `oxml-layout` package stanza. Font loading,
  shaping, parsing, errors, and line breaking each have a named consumer for
  the manifest dependencies.
- Surface: the crate exports only the copied output and font surface, the owned
  F-030 line types and functions, and the exact F-031 `Transform` contract.
- Publication: `oxml-layout` remains version 0.0.0 with `publish = false`, and
  the seven released rdocx packages remain the explicit publication allowlist.
