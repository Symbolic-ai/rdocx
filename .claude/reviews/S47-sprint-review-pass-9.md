# S47 sprint review, pass 9

**Reviewed**: `sprint/s47` at `71cd6e1` against `d625bda4`, 51 files,
7,286 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 9 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 3 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, a public positioned sidecar can still place raw content after rPrChange

`crates/rdocx-oxml/src/properties.rs:716`
`crates/rdocx-oxml/src/properties.rs:1062`
`crates/rdocx-oxml/src/properties.rs:1394`
`crates/rdocx-oxml/src/properties.rs:1431`

The pass-8 parser now clamps hostile post-change input to the change slot, and
the legacy writer path emits raw content before the change. The aligned
positioned path still trusts the public `revision_xml_positions` values. A
programmatically constructed `CT_RPr` with a typed change, one raw child, and
the aligned position `(RPR_END_SLOT, 0)` takes that path. The generated
`w:rPrChange` is written at slot 40, then the root-end branch writes the slot-41
raw child after it. The public field is hidden from documentation but remains
constructible and mutable.

This leaves the pass-8 requirement false for one supported in-memory state and
can emit schema-invalid run properties. The writer must force every retained
raw child ahead of a present schema-final change regardless of sidecar values,
including aligned, missing, and length-mismatched programmatic sidecars. A
direct-construction regression must exercise each path and assert that nothing
follows `w:rPrChange`.

### B2, owner-local namespace declarations are dropped from retained rPr children

`crates/rdocx-oxml/src/text.rs:300`
`crates/rdocx-oxml/src/properties.rs:1018`
`crates/rdocx-oxml/src/properties.rs:1083`
`crates/rdocx-oxml/src/text.rs:1775`

The run parser captures the complete `w:rPr` only to build a typed projection.
`CT_RPr` retains child bytes and positions but no owner attributes or namespace
scope, and serialization reconstructs a fixed `<w:rPr>` owner. With
`<w:rPr xmlns:x="urn:producer"><x:raw/></w:rPr>`, the retained child is written
as `<x:raw/>` after the only declaration that bound `x` has been discarded.
The same failure affects a locally declared Word alias used by a non-empty
unsupported property. `write_raw_with_word_override` writes bytes unchanged
when no canonical `w` shadow override is active, so it cannot repair either
case.

The pass-8 regression declares `wx` at the document root and the foreign `w`
binding on the enclosing hyperlink, so both bindings survive independently of
the reconstructed run-properties owner. The fix must retain the required owner
scope or promote exact required bindings onto retained child roots. Regressions
must use declarations present only on `w:rPr` for a foreign child, an aliased
non-empty Word property, and a retained change, then prove namespace-well-formed
save and reopen with the same expanded names exactly once.

### B3, duplicate property-change elements silently overwrite earlier revisions

`crates/rdocx-oxml/src/properties.rs:968`
`crates/rdocx-oxml/src/properties.rs:1007`
`crates/rdocx-oxml/src/properties.rs:231`
`crates/rdocx-oxml/src/properties.rs:354`
`crates/rdocx-oxml/src/table.rs:529`
`crates/rdocx-oxml/src/document.rs:277`
`crates/rdocx-oxml/src/revision.rs:238`

Every metadata-valid `w:rPrChange` is parsed into the single `change` field
without checking whether that field is already occupied. A second element
therefore drops the first captured subtree, and revision collection can report
only the survivor. Paragraph, table, section, and numbering property parsers
use the same single-field overwrite pattern. This is different from canonical
collapse of duplicate ordinary formatting properties because each change
element carries independent revision identity and raw source bytes.

This violates the F-149 round-trip gate for hostile but readable producer XML
and lets ordinary load and save delete a revision. Duplicate schema-final
changes must either make the complete owner opaque or retain every noncanonical
occurrence as raw XML while exposing only the defined typed occurrence. Tests
must cover all four property-change owners and the numbering marker, with save,
reopen, reporting, scoped resolution, and exact single retention of every
input subtree.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish the complete gate because the sprint plan assigns the
end-of-milestone gate to S48. For the S47 contribution, all 206 `rdocx-oxml`
tests, all 53 `rdocx` regression tests, all 87 `rdocx` integration tests, and
all 49 hash-harness entries pass at the reviewed SHA. The pass-8 regressions
establish hostile parsed ordering, canonical ordinary-property collapse, and
display insertion around their direct raw boundaries. B1 through B3 identify
programmatic ordering, owner-local namespace, and duplicate revision cases
outside that evidence, so the tracked-change preservation contribution remains
blocked.

## Not found

- `prior review findings`: pass-1 B1, pass-2 B1 and B2, pass-3 B1, pass-4 B1
  through B3, pass-5 B1, pass-6 B1 and B2, and pass-7 B2 through B4 remain
  fixed for their cited cases. Pass-7 B1 and pass-8 B1 through B3 are fixed for
  parsed sidecars and the direct display fixture. B1 through B3 above are the
  remaining uncovered states. Zero additional prior-case regressions.
- `content-control display`: deletion and insertion remapping retains direct
  raw children around a replacement value. The code also leaves deleted text
  and comment references typed, skips nested control boundaries during an
  outer replacement, and later visits selected nested controls through the
  indexed traversal. Zero additional display findings beyond the tests named
  in the milestone evidence.
- `run mutations and comments`: `set_text`, `add_text`, property
  materialization, direct comment removal, and content-control comment removal
  retain live run-local raw boundaries in the focused regressions. Zero
  additional mutation findings.
- `namespace repair`: canonical `w` repair uses parsed qualified names and
  exact declarations for retained run children under a shadowed hyperlink.
  Zero additional canonical-shadow findings beyond B2's owner-local scope.
- `revision reporting and resolution`: modeled revisions report once in tested
  document order, malformed owners stay opaque, selectors retain exact counts,
  and selected nesting resolves inside out with atomic commit. Zero additional
  reporting or resolver findings beyond B3's overwritten duplicates.
- `numbering sole source`: numbering clears the raw run-property projection and
  its position sidecar before overlay serialization. The no-duplication and
  namespace regressions pass. Zero additional numbering findings beyond B3's
  duplicate marker case.
- `surface and docs`: HLD 03, HLD 10, the F-149 plan, and AS_BUILT record the
  intentional low-level 0.8.0 boundary. The native `rdocx::Document` facade is
  additive, and Python, WASM, and CLI surfaces remain unchanged. Zero surface
  or documentation findings.
- `interaction`: outside B1 through B3, no conflict was found between the
  F-149 projection, F-150 resolver, comments, hyperlinks, and content controls.
- `duplication`: no duplicate sprint helper or second public revision model was
  introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `deps`: no dependency was added.
- `harness`: the independent check reports all 49 entries unchanged, matching
  both S47 AS_BUILT entries.
- `oracle`: the normalized-body regression remains pinned to Microsoft Word
  16.104 build 16.104.25121423 and compares normalized typed body structure.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on
  both completed stories, estimates, actuals, HLD files, and unchanged harness
  evidence.
