# F-X023, correctness, pass 1

**Reviewed**: the uncommitted working tree. One `[workspace.package]` version,
9 root pins, 2 Python project versions, the `rdocx-wasm` contract literals, the
`ci.yml` WASM literal, the stable release regression, `publish.yml`, and
`Cargo.lock`. No product logic changed.
**Verdict**: 0 defects, 0 smells, 1 nice-to-have

## Defects

None.

F-X022's defect was that it stopped at the `crates/` boundary and missed the
release regressions that `publish.yml` runs as its gate. This story took that
lesson: the Python suite and both workflow files were updated in the same pass,
and all 46 release regressions pass.

## Smells

None.

## Nice-to-have

### N1, the stable train bumps in one line and the incubating train in fifteen
`Cargo.toml`

The eleven stable packages inherit `version.workspace = true`, so the version
moves once. The fifteen incubating packages each carry a literal. This story
touched one manifest line where F-X022 touched fifteen, for the same outcome.

Carried from F-X022's own N1 and recorded again here because the contrast is
the argument: giving the incubating train a shared version key would make the
next bump symmetrical. Out of scope for a release story.

## Not found

Checked and produced nothing:

- **completeness**. No `0.6.0` remains anywhere in the workspace: manifests,
  Python projects, Rust sources, workflows, or the test suite. The lockfile was
  regenerated rather than hand-edited.
- **scope**. The incubating train stays at 0.3.0. The stable pins on incubating
  crates were already moved by F-X022 and are untouched here.
- **publication set**. The four unpublished packages, `oxml-py-support`,
  `rdocx-py`, `rdocx-wasm` and `rpptx-py`, inherit 0.7.0 and gain no
  publication authority. The published stable set stays exactly seven.
- **gates**. `publish.yml` now invokes
  `test_stable_release_family_is_prepared_at_0_7_0` and
  `test_incubating_release_family_is_prepared_at_0_3_0`, both of which exist and
  pass.
- **surface**, **layering**, **ooxml**, **panics**. No source logic changed.

## Hash harness

**Unchanged, 28 of 28.** Expected: a version string reaches no rendered byte.
