# F-110, all, pass 2

**Reviewed**: uncommitted working diff against `HEAD`, 5 files, 804 additions and 5 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 1 resolution

D1 is resolved. `CT_ShapeTree::append_child` moves every raw child at the old
trailing boundary before adding the new typed child, while raw children at
earlier boundaries remain anchored to their existing typed successors. This
keeps multiple trailing raw subtrees together after the appended member and
preserves raw subtrees at every earlier boundary. All four facade constructors
use this helper. The constructor integration test also verifies that the
schema-final extension remains byte-identical and follows each appended child.

D2 is resolved. The allocator combines typed recursive ids with a namespace
resolved scan of the complete serialized tree. The scan accepts arbitrary
prefixes bound to PresentationML, ignores foreign `cNvPr` elements, and walks
every preserved `mc:AlternateContent` branch rather than only the selected
fallback projection. Allocation still begins at 2 and fills unused gaps while
reserving each result, and every facade constructor rescans immediately before
append. The raw-member and non-selected-choice test exercises occupied ids in
both opaque forms.

D3 is resolved. `CT_PresetGeometry2D::new` checks the generated ECMA preset
table and returns `UnknownPreset` for any missing value. The facade constructs
and validates the shape before appending it. The integration test proves that
an unknown preset returns an error and leaves the serialized presentation
unchanged. The generated table gate confirms all 187 pinned definitions.

D4 is resolved. The reopen test asserts the textbox and ordinary-shape offsets,
extents, presets, text shells, and fill states. It also asserts connector
offset, extent, both flips, and preset, plus the empty group shell and absence
of group bounds or members.

## Not found

Correctness produced no additional findings. Connector normalization covers
all four endpoint directions, vertical and horizontal zero extents, and the
checked failure at the full signed EMU span. Repeated rescans prevent duplicate
ids without skipping available low ids.

Contract produced no findings. The four approved methods, deterministic names,
fixed shells, and top-of-z-order behavior match the design plan.

Panics produced no findings. The post-append `last_mut` and fixed-shell group
name `expect` are justified by immediate local invariants. Endpoint subtraction
uses `abs_diff` followed by a checked conversion.

OOXML produced no findings. Required children are emitted in schema order,
namespace matching uses the PresentationML URI, and opaque raw subtrees remain
the serialization source for all alternate branches. Multiple raw boundaries
retain their relative placement when a child is appended.

Tests produced no findings. `cargo test -p oxml-drawing`,
`cargo test -p rpptx-oxml`, and `cargo test -p rpptx --test integration` passed.
The ignored native helper was invoked explicitly and passed against pinned
PowerPoint 16.104 build 16.104.25121423. `cargo fmt --all --check` also passed.

Structure produced no findings. No new module, trait, generic parameter,
feature, dependency, forwarding wrapper, or erased concrete type was added.
