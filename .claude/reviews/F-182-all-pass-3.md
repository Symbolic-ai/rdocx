# F-182, all aspects, pass 3

**Reviewed**: the complete current working-tree implementation diff, 7 files
with 2,170 additions and 3 deletions, plus both prior review passes and their
remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, link validation can trim away an XML-invalid character that is still emitted

`crates/rdocx/src/svg.rs:902`
`crates/rdocx/src/svg.rs:341`

`is_safe_link` trims the target before checking control and XML character
rules, but `emit_link` escapes and writes the original untrimmed value. U+000B
is both Unicode whitespace and forbidden by XML 1.0. A target ending in U+000B,
such as `https://example.test/\u{000B}`, is therefore trimmed to a safe URL,
accepted, and then emitted with the forbidden scalar still present. The result
is not parseable XML. Validate the exact value that is emitted, or emit the
validated trimmed value, and add the trailing-whitespace control case to the
XML parse regression.

### D2, singular-transform filter bounds are not conservative for supported source geometry

`crates/rdocx/src/svg.rs:525`
`crates/rdocx/src/svg.rs:675`
`crates/rdocx/src/svg.rs:698`

When the accumulated transform is singular, the filter region falls back to
hand-estimated child bounds. Text uses a fixed box from two font sizes above
the baseline through one below it, which is not a bound on arbitrary embedded
font ink. A stroked path expands its command bounds by only half the stroke
width plus one point, while the emitted default miter join can extend to four
times the half-width. A valid singular transform that preserves the axis of an
acute miter, or the axis of outlying glyph ink, therefore places supported
source pixels outside the filter region. SVG clips both `SourceGraphic` and
the derived shadow there. The singular regression uses only a filled
rectangle, so it cannot detect either under-bound. The fallback must use a
conservative bound for every supported source or avoid a content-derived
finite region when it cannot prove one.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx svg::tests -- --nocapture` passed all 14 focused
  renderer tests, including the pinned resvg 0.48.1 oracle at 150 dpi.
- The representative golden contains exact-font text, image data, gradient
  geometry, a transformed and clipped group, opacity, nonzero shadow blur, a
  safe link, marked content, and one diagnosed fallback. Its one-point
  perturbation remains below the 0.99 SSIM threshold.
- `cargo tree -p rdocx --edges normal --depth 1` shows `base64` 0.23.1 in the
  runtime graph and no `resvg`. The development graph contains exact `resvg`
  0.48.1.
- `git diff --check` passed before this review record was written.

## Not found

Pass 2 D1 is fixed for invertible transforms and large local coordinates by
mapping the page viewport into group-local filter units. Pass 2 D2 is fixed by
placing the local clip on the inner source wrapper, so the filter derives a
shadow from clipped content without clipping that shadow at the same level.
Pass 2 D3 is fixed by the representative golden's nonzero blur. Pass 2 D4 is
fixed for XML-invalid scalars that remain inside the validated link value.

No additional defect was found in page bounds, normal and deterministic cache
selection, out-of-range handling, recursive child order, affine composition,
marked-content recursion, group opacity, independent shadow branches,
clip/source/filter ordering, page background, solid geometry, path fill rules,
gradient geometry and stop normalization, stroke caps, joins and dashes, PNG
and JPEG embedding, safe scheme filtering apart from D1, text and attribute
metacharacter escaping, XML-invalid visible-text replacement, font first-use
order, exact oracle face resolution, TTC fallback diagnostics, complex-text
diagnostics, unsupported paint and effect diagnostics, ordered layout and
lowering diagnostics, source-document preservation, public result shape,
binding-surface containment, dependency-family direction, development-only
oracle placement, deterministic serialization, or the approved private-module
structure. No panic on an untrusted production path, OOXML mutation, trait,
generic, feature, crate, or test binary was added. Correctness, contract,
panics, OOXML, tests, and structure were all checked afresh. Apart from D1 and
D2, those aspects produced zero findings.
