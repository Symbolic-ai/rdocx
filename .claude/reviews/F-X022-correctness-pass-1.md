# F-X022, correctness, pass 1

**Reviewed**: the uncommitted working tree. 15 crate manifests, 14 root pins,
11 READMEs, 7 Rust sources, one line in `scripts/readme_doctests.py`, and
`Cargo.lock`. No product logic changed.
**Verdict**: 0 defects, 0 smells, 1 nice-to-have

## Defects

None outstanding.

### D1 found and fixed during the pass, the release regressions were missed
`scripts/test_sprint_workflow.py` and `.github/workflows/ci.yml`

The first pass moved every carrier under `crates/` and stopped there. It missed
the release-family preflight in `scripts/test_sprint_workflow.py`, which
`.github/workflows/publish.yml` runs by name as the publication gate, and the
`verify_package` literal for `@tensorbee/rpptx-wasm` in `ci.yml`.

`cargo test` does not run the Python suite and neither does `/verify`, so this
would have passed every local gate and failed in CI at publication time. Found
by running the release regressions directly rather than trusting the local
gate.

Fixed: `test_incubating_release_family_is_prepared_at_0_2_0` renamed to
`..._0_3_0` with its expectations moved, `publish.yml` updated to invoke the new
name, and the `ci.yml` WASM literal moved to 0.3.0. All 46 tests in the suite
pass.

Every carrier moved together, which is the only real risk in a version bump. The
counts match the plan's inventory exactly: 15, 14, 11, 7. No `0.2.0` remains in
any manifest, README, Rust source or test under `crates/`.

## Smells

None.

## Nice-to-have

### N1, the version lives in 48 places rather than one
`Cargo.toml` and 47 other files

The incubating train has no single source of truth. The stable train uses
`version.workspace = true`, so its eleven packages move by editing one line. The
incubating packages each carry a literal, and the pins, READMEs and test
assertions repeat it.

That is why this story's inventory had to be taken by grep rather than known.
A second `[workspace.package]`-style shared version for the incubating train
would collapse 15 manifest edits to one, though the pins and prose would remain.
Out of scope here, and worth its own story if the train bumps often.

## Not found

Checked and produced nothing:

- **correctness**. `cargo update --workspace` regenerated the lockfile rather
  than a hand edit, so the lock agrees with the manifests by construction.
- **completeness**. The seven Rust sources that assert a version string or a
  pin string all pass, and they are what would catch a partial bump. The README
  doctest runner asserted the old install string and now asserts the new one.
- **scope**. The stable train is untouched at 0.6.0. Its pins on the incubating
  crates moved to 0.3.0, which is required rather than incidental: `rdocx-layout`
  uses the new `oxml-layout` API and could not resolve against 0.2.0.
- **publication set**. `rpptx-wasm` moved to 0.3.0 and remains `publish = false`,
  so the published set stays exactly fourteen.
- **surface**, **layering**, **ooxml**, **panics**. No source logic changed.

## Hash harness

**Unchanged, 28 of 28.** Expected: a version string reaches no rendered byte.
