# S47 sprint review, pass 5

**Reviewed**: `sprint/s47` at `99d3211` against `d625bda4`, 45 files,
5,618 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 5 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, a shadowed Word prefix moves reconstructed hyperlink runs into a foreign namespace

`crates/rdocx-oxml/src/text.rs:354`
`crates/rdocx-oxml/src/text.rs:1220`
`crates/rdocx-oxml/src/text.rs:1508`
`crates/rdocx-oxml/src/text.rs:1529`

The pass-4 writer selects a safe fallback prefix for a modeled hyperlink when
its preserved attributes locally rebind `w`, then writes that local foreign
`xmlns:w` declaration back on the owner. The typed runs inside the hyperlink
still serialize through `CT_R::to_xml`, which unconditionally emits `w:r` and
its typed children with the `w` prefix. A namespace-correct input such as an
aliased Word `wx:hyperlink` with `xmlns:w="urn:foreign"` and aliased Word runs
therefore loads as modeled content, but saves its reconstructed runs under the
foreign namespace inherited from the retained owner declaration. Reopening
the result no longer sees those runs as WordprocessingML.

This breaks F-149's prefix-tolerant round-trip gate and the raw-preservation
contract under the same local namespace shadowing that pass 4 addressed for
the relationship id. The fix must make the safe Word prefix apply to every
reconstructed typed descendant of the hyperlink, while retaining foreign
owner declarations and raw children without duplication. Add a regression
with an aliased Word hyperlink, a locally foreign `w` binding, ordinary runs
before and after revision and raw children, then prove save and reopen retain
the text, revision reporting, child order, and expanded Word names.

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
end-of-milestone gate to S48. For the S47 contribution, all 205 `rdocx-oxml`
tests, all 50 `rdocx` regression tests, and all 87 `rdocx` integration tests
pass. The focused pass-4 regressions prove opaque malformed wrappers across
all selectors, retained zero-run and multi-run hyperlink content across
comment removal and insertion, and relationship ids resolved by expanded
name with a locally shadowed `r` prefix. B1 shows that the equivalent shadow
case for the Word prefix still changes modeled content into foreign XML, so
the tracked-change contribution remains blocked.

## Not found

- `prior review findings`: pass-1 B1, pass-2 B1 and B2, pass-3 B1, and pass-4
  B1 through B3 remain fixed for their cited cases. Malformed revision owners
  are opaque to reporting and every resolution scope. Comment insertion and
  removal retain and remap hyperlink revisions and raw children, including a
  span whose only ordinary run is removed. Relationship ids use expanded-name
  lookup, aliases are projected, and a locally foreign `r:id` stays raw.
- `interaction`: outside B1, no additional conflict was found between typed
  revision ownership, comment mutations, hyperlink inspection, and scoped
  resolution.
- `duplication`: no duplicate sprint helper or second public revision model
  was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: the independent check reports all 49 entries unchanged, matching
  both S47 AS_BUILT entries.
- `docs`: HLD 03, HLD 04, and HLD 10 describe revision ownership, atomic
  resolution, and the native-only surface. No separate documentation gap was
  found beyond B1's implementation contradiction.
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
