# S47 sprint review, pass 6

**Reviewed**: `sprint/s47` at `9039c08` against `d625bda4`, 46 files,
5,849 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 6 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, shadowed Word prefixes still promote or discard foreign run children

`crates/rdocx-oxml/src/text.rs:224`
`crates/rdocx-oxml/src/text.rs:257`
`crates/rdocx-oxml/src/text.rs:286`
`crates/rdocx-oxml/src/text.rs:322`
`crates/rdocx-oxml/src/properties.rs:707`
`crates/rdocx-oxml/src/properties.rs:816`

The pass-5 writer keeps the reconstructed run and its typed descendants in the
Word namespace, but the run parser still recognizes text, drawings, tabs,
breaks, and note references by local name alone. Under the reviewed aliased
Word hyperlink with `xmlns:w="urn:foreign"`, a foreign `w:t` inside a valid
`wx:r` is therefore projected as ordinary Word text and written back under the
Word namespace. A foreign empty `w:rPr` is dropped by the local-name exclusion.
Within a valid aliased `wx:rPr`, foreign same-local properties are skipped
instead of retained. The pass-5 regression uses a foreign `w:inside`, whose
local name is not modeled, and a Word `wx:b`, so it does not exercise these
collisions.

This changes expanded names, can manufacture public text or formatting from
foreign XML, and can lose producer XML on an ordinary load and save. It
contradicts the F-149 namespace-collision gate and HLD 03's raw preservation
contract. The fix must namespace-check every modeled run child and attribute,
retain foreign and unmodeled run and run-property children, and prove after
save and reopen that foreign `w:t`, `w:drawing`, empty `w:rPr`, and a foreign
same-local property remain foreign, unreported, and present exactly once.

### B2, raw children inside a run still move after typed content

`crates/rdocx-oxml/src/text.rs:164`
`crates/rdocx-oxml/src/text.rs:278`
`crates/rdocx-oxml/src/text.rs:442`
`crates/rdocx-oxml/src/revision.rs:879`

`CT_R` stores unmodeled children in a flat `extra_xml` list without their
typed-content boundary. Except for the special comment-reference slot, the
writer emits that complete list only after all typed run content. A raw child
that originally precedes text or sits between two typed children therefore
moves to the end of the run. The pass-5 regression places its one raw run
child after the run's only text node, so the asserted case cannot reveal the
reordering.

This violates ordered raw XML preservation and can change producer semantics
even though the bytes of the moved subtree remain present. The fix must retain
raw run children at ordered typed-content boundaries, including raw children
before, between, and after typed text, properties, fields, or drawings as the
grammar permits. A regression must cover the shadowed-prefix hyperlink, save,
reopen, expanded names, exact child order, revision reporting, and no duplicate
emission.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish that gate because the sprint plan assigns the complete
end-of-milestone gate to S48. For the S47 contribution, all 206 `rdocx-oxml`
tests, all 50 `rdocx` regression tests, and all 87 `rdocx` integration tests
pass. The pass-5 regression proves that aliased typed runs, typed run
properties, one trailing raw run child, an aliased revision, and one raw
hyperlink-boundary child retain their tested order and survive reopening under
a locally foreign `w` binding. The hash harness independently reports all 49
entries unchanged. B1 and B2 show that foreign same-local children and other
raw run-child positions remain outside that evidence, so the tracked-change
contribution remains blocked.

## Not found

- `prior review findings`: pass-1 B1, pass-2 B1 and B2, pass-3 B1, pass-4 B1
  through B3, and the direct pass-5 case remain fixed for their cited inputs.
  Valid nested revisions report and resolve inside out. Targetless hyperlinks
  retain sibling order. Malformed owners remain opaque. Comment mutations
  remap retained hyperlink content. Relationship ids use expanded names.
- `hyperlink boundaries`: aliased revision subtrees and raw boundary children
  retain expanded names, order, reporting, and single emission in the focused
  pass-5 case. No separate boundary defect was found beyond the run-local B1
  and B2 cases.
- `fields and drawings`: the shadowed-prefix writer places a Word namespace
  declaration on modeled simple fields, and modeled drawings inherit the Word
  binding from their reconstructed run. No writer-side prefix defect was found
  beyond the namespace-unsafe parsing in B1 and the ordering model in B2.
- `interaction`: outside B1 and B2, no additional conflict was found between
  typed revision ownership, hyperlink serialization, comment mutations,
  inspection, and scoped resolution.
- `duplication`: no duplicate sprint helper or second public revision model was
  introduced, and focused save and reopen checks report each tested revision
  once.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: the independent check reports all 49 entries unchanged, matching
  both S47 AS_BUILT entries.
- `docs`: HLD 03, HLD 04, and HLD 10 describe revision ownership, atomic
  resolution, and the native-only surface. No separate documentation gap was
  found beyond the implementation contradictions in B1 and B2.
- `deps`: no dependency was added.
- `surface`: `RevisionKind`, `RevisionRef`, `Document::revisions`, and the
  eight resolution methods are required by F-149 and F-150. Python, WASM, and
  CLI surfaces remain unchanged.
- `oracle`: the normalized-body regression records Microsoft Word 16.104
  build 16.104.25121423 and compares normalized typed body structure rather
  than producer bytes.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on
  the two completed stories, their estimates and actuals, HLD files, and
  unchanged harness evidence.
