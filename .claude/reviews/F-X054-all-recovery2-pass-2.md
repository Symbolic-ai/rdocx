# F-X054, all, second recovery pass 2

**Reviewed**: uncommitted working diff, 15 files, 3,194 changed lines with
3,125 additions and 69 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, Unsupported XML facts omit the implicit XML namespace binding

`crates/rdocx/src/document.rs:86`

The raw fact resolver checks only declarations on the retained root and the
explicit declarations inherited from `w:body`. XML permanently binds the
`xml` prefix to `http://www.w3.org/XML/1998/namespace` without requiring an
`xmlns:xml` attribute. A valid unsupported body child such as
`<xml:producer/>` therefore reports `qualified_name() == Some("xml:producer")`
but `namespace_uri() == None`. The logical-owner snapshot code handles this
built-in binding separately, so the public compatibility fact and replay
identity disagree about the same expanded name.

### D2, An unused fixed-prefix declaration rejects a safe edit

`crates/rdocx/src/document.rs:937`

The collision guard examines every declaration stored on an owner instead of
only declarations used by its retained raw markers. For example, a paragraph
can declare `xmlns:x="urn:producer"` for `<x:producer/>` and also carry an
unused `xmlns:wp="urn:unused"`. The `x` child causes the paragraph to become a
namespace owner, then the unrelated `wp` declaration makes every modification
fail with `shadowed wp namespace`. No retained event depends on that `wp`
binding, so canonical `wp` output plus replay of `x` is safe. This violates the
declaration-dependent guard and rejects a valid edit.

### D3, Empty markers discard exact expanded-marker identity

`crates/rdocx/src/document.rs:836`

When an owner has any declaration-dependent empty event, the marker collector
discards all declaration-dependent expanded events. Two same-namespace owners
can therefore share an empty `<x:a/>` marker while retaining lexically distinct
expanded `<x:b ...></x:b>` subtrees that normalize to the same logical
snapshot. After editing the intended owner's typed text, replay ignores the
unique expanded raw bytes and fails closed as ambiguous. The retained expanded
event supplies an exact stable identity, so this is a valid edit that the full
marker record could identify safely.

### D4, Duplicate marker matching ignores event cardinality

`crates/rdocx/src/document.rs:977`

Each original marker is matched with an independent `any` search, so one
candidate event can satisfy multiple identical original events. For example,
an owner with two exact `<x:a/>` raw children is treated as structurally
interchangeable with an owner containing `<x:a/>` and the semantically
equivalent but byte-distinct `<x:a />`. Editing the first owner's typed text
then fails closed because the second owner is incorrectly retained as a
same-namespace structural alternate. Marker identity must compare an exact
event multiset, including multiplicity, so the distinct owner remains
selectable through a valid edit.

## Smells

None.

## Nitpicks

None.

## Not found

All earlier pass triggers were rechecked. Raw run boundaries, parser-derived
names, decoded local and inherited namespace values, root and body scope
separation, empty undeclarations, child-content facts, fixed-prefix collision
rejection when raw content really depends on the prefix, unchanged unsafe
scope preservation, and duplicate-marker removal rejection remain correct for
their covered inputs.

No additional findings were found in body, cell, paragraph, hyperlink, or run
item order, exact exposed raw bytes, public enum exhaustiveness, drawing and
field projections, legacy flattened accessors, producer-defined numbering
round trips, layout and exporter marker suppression, fail-closed ordinary or
deleted text decoding, Python error classification, OOXML child order, panic
safety, public API documentation, dependency structure, test naming, or the
repository structural rules. The complete 162-test `rdocx` regression binary
and all 278 `rdocx-oxml` unit tests plus its documentation test passed during
this review.
