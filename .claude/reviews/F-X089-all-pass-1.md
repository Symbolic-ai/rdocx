# F-X089, all, pass 1

**Reviewed**: full working diff against base `509940cb`, 32 files and 1,171
changed lines, comprising 753 additions and 418 deletions
**Verdict**: 4 defects, 0 smells, 2 nitpicks

## Defects

### D1, the required README runner rejects the reviewed tree
`scripts/readme_doctests.py:233`
`crates/rpptx-py/README.md:20`

The required-text contract searches for `The binding does not expose PDF or
raster rendering.` as one uninterrupted string. The README wraps between
`raster` and `rendering`, so `python3 scripts/readme_doctests.py` exits 1 before
archive or Rust-example validation. The focused mutation tests pass because
none asserts that the unmodified `validate_inventory()` result is true.

### D2, unsupported comparison rows and uniqueness mutations pass validation
`scripts/readme_doctests.py:608`
`scripts/readme_doctests.py:616`
`README.md:164`

The evidence check compares URL sets, then finds each expected product row, but
it never requires the table to contain only those rows and never binds the
scoped uniqueness statement. An arbitrary extra comparison row, changing
`rdocx alone` to `every project`, and duplicating an approved URL all return
true from `validate_comparison_evidence`. This contradicts the design and HLD
contract that every comparison row and the scoped uniqueness statement are
mutation-sensitive and backed by the exact evidence set.

### D3, both WASM build instructions create a different package name from the import
`crates/rdocx-wasm/README.md:27`
`crates/rdocx-wasm/README.md:37`
`crates/rpptx-wasm/README.md:28`
`crates/rpptx-wasm/README.md:38`
`scripts/readme_doctests.py:218`
`scripts/readme_doctests.py:247`

Both examples import an `@tensorbee/...` package, but both documented
`wasm-pack build` commands omit `--scope tensorbee`. The copied commands
therefore generate unscoped package identities and do not produce the package
named by the import. The validator makes the incorrect commands required text
instead of checking the scoped package command used by CI.

### D4, the facade security claims hide required default-off features
`README.md:23`
`crates/rpptx/README.md:10`
`crates/rdocx/Cargo.toml:20`
`crates/rpptx/Cargo.toml:21`
`scripts/readme_doctests.py:391`

The root and `rpptx` capability lists present encryption and signing as ordinary
facade operations, while `agile-encryption` and `digital-signatures` are absent
from both default feature sets. Neither README tells a user to enable them, so
the documented plain dependency declarations do not expose those APIs. The
root matrix gate also checks only DOCX-001, DOCX-080, DOCX-082, and DOCX-083.
It still passes if DOCX-002, the encrypted package row, is changed from
`complete` to `unsupported`, and there is no matrix row bound to the signing
claim.

## Smells

None.

## Nitpicks

- `crates/rdocx-py/README.md:16`, the unpublished Cargo-package explanation is
  repeated immediately in the relationship section.
- `crates/rpptx-py/README.md:16`, the unpublished Cargo-package explanation is
  repeated immediately in the relationship section.

## Not found

- **API and crate boundaries**: no additional false facade, renderer, OXML,
  Python, CLI, compatibility-shim, or publication claims were found across the
  27 README sources.
- **Official evidence accuracy**: the five comparison rows agree with the
  reviewed official sources and keep `ND` distinct from unsupported. The defect
  is the validator's incomplete table and uniqueness contract.
- **Local links**: link resolution uses each declaring README as its base,
  rejects repository escapes, checks Markdown anchors, and inventories all 27
  package README locations.
- **Recursive page traversal**: the `oxml-layout` guidance correctly requires
  recursion through marked-content children or use of `oxml_layout::walk`.
- **Packages**: metadata drives distinct README discovery, publishability, and
  source paths. The local patch set is checked against the publishable package
  set, and archive comparison is byte exact.
- **HLD scope**: only `docs/hld/12-testing-strategy.md`,
  `docs/hld/14-development-backlog.md`, and
  `docs/hld/15-build-and-toolchain.md` changed, exactly matching the design
  plan's HLD impact list.
- **Structure and OOXML**: no production module, abstraction, dependency,
  serializer, schema ordering, or package-preservation behavior changed.
- **Mechanical checks**: `git diff --check`, `scripts/prose_check.py`, and the
  eight focused README mutation tests passed. The default README runner failed
  as described in D1. The official-link mode could not run in the review
  sandbox because DNS access is unavailable.
