# F-115, all, pass 1

**Reviewed**: complete working tree diff, 7 files, 547 changed lines, comprising 536 insertions and 11 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, replacing a direct background discards its preserved producer XML

`crates/rpptx-oxml/src/slide_parts.rs:336`

`set_background` accepts an existing `BackgroundRendering::Properties`, then
line 350 replaces the complete `CT_Background` with a newly constructed
canonical subtree. A producer background can carry attributes on `p:bg` and
`p:bgPr`, effect children, extension children, comments, and other retained
payload around its fill. All of those bytes disappear when only the fill is
changed. This conflicts with the plan's raw-preservation requirement and with
the HLD statement that the complete captured background subtree remains the
serialization source. The setter must preserve the existing direct-background
container and replace only its fill, or reject the operation before mutation.

### D2, missing-core creation can overwrite an unowned conventional part

`crates/rpptx/src/lib.rs:427`

When no core-properties relationship exists, the facade selects
`/docProps/core.xml` without checking whether that part name is already
occupied. The dirty path then calls `set_part` at line 460, which silently
replaces any existing orphan or unrelated part and changes its content type.
Such a part is outside the relationship-owned core model and should retain its
exact source bytes. A package with no core relationship and an occupied
conventional name therefore loses data on an otherwise successful metadata
save. The staged save should detect this collision and return an error without
changing the source package.

### D3, the planned gates do not prove the values or preservation they name

`crates/rpptx/tests/integration.rs:65`

The facade gate checks only that some explicit background exists after reopen.
It never checks that the requested `345678` fill survived, so a hardcoded or
otherwise wrong background passes. The missing-core branch at line 127 checks
the relationship and content type but never asserts that the new part exists
or that its `subject` is `created`. The lower-level background test at
`crates/rpptx-oxml/tests/integration.rs:114` starts with no direct background,
then checks marker survival around the inserted and cleared node. It cannot
detect D1 because it never replaces a direct background carrying retained
attributes, effects, or raw children. These tests do not satisfy the plan's
gate that each property round-trips or its byte-preservation rider.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no further findings. Missing `show` maps to visible, explicit
  values use the required inverse semantics, both slide dimensions are checked
  before mutation, and `save_as_show` operates on a staged package clone.
- Contract: no further findings. The facade surface matches the approved plan,
  core properties resolve through the package relationship, immutable access
  does not dirty the source part, and slideshow mode is not stored on the
  presentation.
- Panics: zero findings. New production paths use checked results and optional
  access without new indexing, slicing, unchecked arithmetic, `unwrap`, or
  `expect` calls.
- OOXML: no further findings. Root `show` accepts both XML boolean spellings
  and writes fixed numeric spelling. New backgrounds occupy the schema slot
  before `p:spTree`, `p:bgRef` remains untouched, and slideshow output changes
  only the main-part content type.
- Tests: no further findings. The invalid-size test compares complete package
  bytes, the theme-reference test compares the captured background bytes, and
  the slideshow test compares parts, relationships, defaults, and overrides.
- Structure: zero findings. No file, module, trait, generic, feature,
  dependency, forwarding wrapper, or unjustified dynamic dispatch was added.
