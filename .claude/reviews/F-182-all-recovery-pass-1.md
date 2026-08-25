# F-182, all aspects, recovery pass 1

**Reviewed**: the complete current working-tree implementation diff, 7
production files with 2,388 additions and 3 deletions, plus all three original
review passes and their remediation
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, font diagnostics can precede earlier element diagnostics

`crates/rdocx/src/svg.rs:46`
`crates/rdocx/src/svg.rs:139`
`crates/rdocx/src/svg.rs:71`

The renderer collects every used font and emits all font diagnostics before it
traverses page elements. A page whose first element uses unsupported tile paint
and whose second element uses a nonzero TTC face therefore reports the second
element's font approximation before the first element's paint omission. The
approved contract requires layout diagnostics first and then all SVG lowering
diagnostics in traversal order. First-use ordering among fonts does not preserve
that order relative to other lowering diagnostics.

### D2, non-uniform transforms change supported shadow blur semantics silently

`crates/rdocx/src/svg.rs:409`
`crates/rdocx/src/svg.rs:462`
`crates/oxml-pdf/src/raster.rs:506`

SVG applies one local-coordinate Gaussian blur and then applies the group's
affine transform to the filtered result. The shared PNG backend first renders
the transformed subtree in device space, derives one average transform scale,
and applies an isotropic device-space box blur. A valid group scaled by 4 on x
and 0.25 on y consequently gets a strongly anisotropic SVG shadow but an
isotropic PNG shadow. The implementation emits no approximation diagnostic for
this supported `OuterShadow`, and the golden uses translations only. This can
produce a visible unexplained difference from the same `PageFrame` rather than
the declared bounded lowering.

## Smells

### S1, recursive transform tests cannot detect composition-order regressions

`crates/rdocx/src/svg.rs:1110`
`crates/rdocx/src/svg.rs:1932`

The recursion regression has one translated outer group and one identity inner
group. The representative golden has two nested groups, but both transforms are
translations. Translations commute, so reversing the parent and child affine
composition order would leave both tests green. Neither fixture reaches the
plan's stated three nested group levels. Add a non-commuting scale, rotation,
or skew inside three group levels so the regression proves recursive affine
order rather than only recursion through containers.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx svg::tests -- --nocapture` passed all 16 focused
  renderer tests, including the pinned resvg 0.48.1 golden.
- `svg_export_preserves_searchable_text_geometry_fonts_images_links_and_clips`
  passed in the integration binary.
- `svg_facade_options_share_the_existing_layout_paths_and_bounds_contract`
  passed in the regression binary.
- `cargo tree -p rdocx --edges normal --depth 1` confirms `base64` and
  `ttf-parser` are runtime dependencies. The development-only tree contains
  exact `resvg` 0.48.1.
- `git diff --check` passed before this review record was written.

## Not found

The recovery fixes for the prior two defects are present. Link safety validates
the exact untrimmed string that is emitted, including forbidden XML scalars in
leading or trailing whitespace. Singular-transform source bounds parse the
selected font collection face, use its global ink bounds, include the SVG
default miter extent, intersect local clips, and expand for blur plus shadow
offset. When a supported source cannot be bounded, effects are omitted with a
stable path-specific diagnostic while source content and siblings remain.

No additional defect was found in page dimensions, normal and deterministic
cache selection, out-of-range handling, visible child order, marked-content
recursion, group opacity, local clip and shadow ordering, independent shadow
branches, solid geometry, path fill rules, gradient geometry and stop
normalization, stroke caps, joins and dashes, PNG and JPEG embedding, reviewed
link schemes, XML scalar handling, text and attribute escaping, searchable
text retention, exact scalar positioning under the approved count rule, font
first-use resource order, TTC approximation reporting, complex-text reporting,
unsupported paint and effect omission, source-document preservation, public
result shape, binding-surface containment, dependency-family direction,
development-only oracle placement, deterministic serialization, package
structure, or the approved private-module boundary. No untrusted production
panic, OOXML mutation, trait, generic, feature, crate, or test binary was
introduced. Correctness, contract, panics, OOXML, tests, and structure were all
checked afresh. Apart from D1, D2, and S1, those aspects produced zero findings.
