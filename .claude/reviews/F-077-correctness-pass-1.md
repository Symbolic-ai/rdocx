# F-077, correctness, pass 1

**Reviewed**: uncommitted F-077 worker diff, 8 files, 713 additions and 15 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, notesStyle is incorrectly required

`crates/rpptx-oxml/src/notes_parts.rs:124`

The PresentationML `CT_NotesMaster` sequence defines `p:notesStyle` with
`minOccurs="0"`, but the parser rejects a notes master that omits it and the
public model cannot represent that valid document. Make `notes_style` optional,
write it only when present, add a no-notesStyle round-trip test, and correct the
approved plan and HLD sequence at
`docs/hld/06-presentationml-model.md:60` before completion.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no other notes-root, text-order, body-placeholder filtering,
  recursive traversal, or relationship-cardinality defect found.
- Contract: no work outside the approved notes parts, typed shape text body,
  tests, and HLD impact found apart from the schema correction above.
- Panics: no production panic path, unchecked indexing, or unsafe arithmetic on
  untrusted XML found.
- OOXML: no other child-order, namespace-resolution, fixed-prefix, or raw-slot
  preservation defect found.
- Tests: no vacuous story gate found. The corpus gate proves notes extraction
  and structural round-trip, while the relationship gate checks the required
  package links.
- Structure: no unjustified trait, generic, dynamic dispatch, forwarding-only
  wrapper, feature flag, crate, or dependency edge found. The new module was
  explicitly approved and owns one cohesive notes-part family.
