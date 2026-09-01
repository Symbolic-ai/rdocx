# F-226, correctness, pass 3

**Reviewed**: pass-2 remediation working diff, 5 tracked paths plus the design plan, 1,604 diff lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, notes-slide relationships are resolved in the notes-master scope

`crates/rpptx/src/lib.rs:6511`

The compositor copies notes-slide placeholder content and ordinary children
into the master-derived transient tree, then calls `render_export_surface` with
only `notes_master_part` as the source. That function scans all merged shapes
against the same notes-master relationship set at
`crates/rpptx/src/lib.rs:6961`. A direct notes-slide image, hyperlink, chart, or
SmartArt relationship is therefore missing or can alias an unrelated master
relationship with the same `rId`. The current media regression proves only a
notes-master image. It does not put relationship-owned content in the notes
slide. The transient composition must preserve distinct owner scopes or remap
notes-slide relationship identifiers into an unambiguous combined scope.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-2 D1 now follows index-first and type-fallback placeholder matching,
rejects ambiguity in both directions, and has omitted-index success coverage.
The pass-1 remediations remain correct. No new public API, dependency,
allocation, namespace, schema-order, source mutation, deterministic-font,
panic, hash, or ordinary rendering issue was found. Focused tests, `rpptx`,
Clippy, prose, diff hygiene, and 49 of 49 hash entries passed.
