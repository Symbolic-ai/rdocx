# F-182, all aspects, recovery pass 2

**Reviewed**: the complete current working-tree implementation diff, 7
production files with 2,614 additions and 3 deletions, plus all four prior
review passes, the progress record, and their remediation
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the runtime manifest exceeds the approved dependency contract

`crates/rdocx/Cargo.toml:44`

The implementation adds `ttf-parser` as a direct production dependency of the
published `rdocx` crate. The approved design authorizes `base64` as the one new
runtime dependency and exact resvg 0.48.1 as development-only validation at
`.claude/plans/F-182-design.md:116` and repeats that closed decision at
`.claude/plans/F-182-design.md:199`. The risk route and implementation checklist
likewise inventory only those two dependency changes. The singular-transform
bounds remediation has a concrete use for `ttf-parser`, but that does not
revise the design contract or its release checks. Remove the direct dependency,
reuse an approved existing boundary, or revise and approve the plan so the
published runtime graph and required verification are explicit before
completion.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx svg::tests -- --nocapture` passed all 19 focused SVG
  tests, including the pinned resvg 0.48.1 golden at 150 dpi.
- `font_diagnostics_are_emitted_at_first_use_in_element_order` proves an
  earlier tile-paint diagnostic remains before the later first-use TTC
  diagnostic.
- `non_uniform_shadow_blur_is_diagnosed_without_dropping_output` proves the
  anisotropic case retains its transform, filter, and supported source while
  reporting the exact effect path and stable approximation text.
- `three_group_noncommuting_transform_order_matches_flattened_pixels` proves
  three nested scale, rotation, and skew transforms rasterise identically to
  the correct flattened composition. Its reversed-composition negative control
  produces different pixels.
- The representative golden contains the same three transform classes, a
  nonzero blurred shadow, exact-font text, an image, gradient geometry, clip,
  opacity, marked content, a safe link, and a diagnosed fallback. It passes the
  global luminance SSIM threshold of 0.99, while its one-point perturbation
  fails the threshold.
- `cargo tree -p rdocx --edges normal --depth 1` confirms `base64` and
  `ttf-parser` are direct runtime dependencies. The development-only tree
  contains exact resvg 0.48.1.
- `git diff --check` and the prose check over all prior F-182 reviews passed
  before this record was written.

## Not found

Recovery pass 1 D1 is fixed. Font definitions remain deterministic in
depth-first first-use order, while missing-face and TTC diagnostics are delayed
until the exact first text traversal position. Layout diagnostics still precede
all lowering diagnostics.

Recovery pass 1 D2 is fixed as the approved bounded approximation. A nonzero
blur under non-uniform scale or skew emits one path-specific diagnostic, keeps
the SVG filter and all supported source output, and remains represented in the
calibrated golden. Conformal scale and rotation do not produce the diagnostic.

Recovery pass 1 S1 is fixed. Recursive affine coverage reaches three group
levels with noncommuting scale, rotation, and skew, compares against the correct
flattened mapping, and includes a reversed-order negative control.

No additional defect was found in page bounds, normal and deterministic cache
selection, out-of-range handling, recursive child order, affine emission,
marked-content recursion, group opacity, local clip and shadow ordering,
independent shadow branches, singular and invertible filter regions, solid
geometry, path fill rules, gradient geometry and stop normalization, stroke
caps, joins and dashes, PNG and JPEG embedding, safe link schemes, XML scalar
handling, text and attribute escaping, searchable text retention, exact scalar
positioning under the approved count rule, font resource order, TTC reporting,
complex-text reporting, unsupported paint and effect diagnostics, source
document preservation, public result shape, binding-surface containment,
deterministic serialization, package structure, or the private module boundary.
No untrusted production panic, OOXML mutation, trait, generic, feature, crate,
or test binary was introduced. Correctness, contract, panics, OOXML, tests, and
structure were all checked afresh. Apart from D1, those aspects produced zero
findings.
