# F-039, all aspects, pass 1

**Reviewed**: the eight-file working diff against claim base `173cd894`, with
266 insertions and 62 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the backlog still forbids the approved shipped-backend edit

`docs/hld/14-development-backlog.md:326`

The F-037 contract still says to leave `rdocx-pdf` unchanged until F-046, while
the approved F-039 plan requires the mirrored shipped-backend change and this
diff implements it at `crates/rdocx-pdf/src/writer.rs:402`. The milestone goal
at `docs/hld/14-development-backlog.md:317` also says released rdocx remains
unchanged. Because the backlog is part of F-039's exact HLD impact, it now gives
future work two conflicting instructions about whether this source change is
allowed. The milestone and F-037 wording must distinguish the approved F-039
CTM mirror from the dependency cutover and publication that remain deferred.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness found balanced page graphics state, the approved text and image
matrices, top-left primitive coordinates, and unchanged annotation dictionary
coordinates in both backends. Panic safety found no new production panic
surface. OOXML ordering, namespaces, whitespace, and subtree preservation are
not applicable because the diff does not parse or serialise OOXML.

Tests found the focused operator cases green, all `rdocx-pdf` tests green, all
seven golden buffers exact against the reviewed Poppler 26.01.0 manifest, the
injected `proposal` pixel rejected precisely, and all 28 hash entries unchanged.
Structure found no new trait, generic parameter, feature flag, production
wrapper, crate, module, or unapproved file.
