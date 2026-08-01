# F-067, correctness, pass 1

**Reviewed**: working tree against the claimed base, 13 files, 641 insertions
and 34 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the new rpptx crate version contradicts the architecture contract

`crates/rpptx-oxml/Cargo.toml:3`

The crate declares version `0.0.0`, matching the design plan but contradicting
the cited versioning contract at `docs/hld/03-architecture.md:106`, which
requires every `rpptx-*` crate to opt out at version `0.1.0`. The design plan
does not list the architecture document under HLD impact, so completion cannot
silently rewrite that contract. Revise the plan to reconcile the version, then
align the manifest, lockfile, and package-policy test with the resolved value.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic-path, OOXML ordering or preservation,
test-strength, or structural findings.
