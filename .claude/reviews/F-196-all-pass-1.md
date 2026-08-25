# F-196, all aspects, pass 1

**Reviewed**: uncommitted `work/f-196-codex` diff, 8 files with 343 additions
and 16 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the checksum gate changes only the last entry

`scripts/test_sprint_workflow.py:1310`

The test named as verifying every checksum changes only `entries[-1]`. An
implementation that ignores the first four digests and verifies only the last
would keep this gate green. The approved regression gate says every checksum is
verified, so the test must corrupt each entry in turn and require every
corruption to fail.

### D2, the packaging text gives the corpus rule to a crate asset

`docs/hld/15-build-and-toolchain.md:162`

The immediately preceding paragraph says external corpora remain outside every
published crate, then says the same treatment applies to
`crates/rpptx/assets/default.pptx` and requires that asset to live inside its
crate. Inserting the corpus paragraph broke the original referent for "same
treatment" and now makes two opposite packaging rules sound equivalent. The
crate-local asset rule needs an explicit referent or the corpus paragraph must
move after it.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic-path, OOXML, test, or structure
finding was found. The manifest has an exact five-entry inventory with one of
every required category, unique safe leaf paths and source URLs, immutable
HTTPS provenance, lowercase SHA-256 values, nonempty producers, and exact
reviewed licence and licence-URL pairs. The fetch path verifies before atomic
replacement and removes its temporary sibling on failure. Read-only checking,
directory membership, CI ordering, HLD impact, the unchanged hash harness, and
the repository structural rules are otherwise satisfied. No crate, module,
trait, generic, dependency, feature flag, public API, or binary fixture was
added.
