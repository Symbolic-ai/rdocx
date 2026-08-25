# F-182, all aspects, pass 2

**Reviewed**: the complete current working-tree implementation diff, 7 files
with 1,863 additions and 3 deletions, plus the pass 1 findings and their
remediation
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, the shadow filter still has a fixed clipping region

`crates/rdocx/src/svg.rs:452`

Pass 1 replaced the object-bounding-box filter region with a fixed user-space
box from -1,000,000 through 1,000,000. Layout coordinates and effect offsets
are `f64` values with no matching bound. A group whose visible child coordinate
or shadow offset falls outside that box is still clipped, even when a group
transform would move the result onto the page. The regression at line 1061
asserts the magic constant rather than proving that supported content cannot be
clipped. The backend needs bounds derived from the page and transformed effect
extent, or another region that is not a finite undocumented content limit.

### D2, a group clip also clips its outer shadow

`crates/rdocx/src/svg.rs:400`
`crates/rdocx/src/svg.rs:455`

The clip path and shadow filter are attributes on the same emitted group. SVG
applies the clip to the filtered group result, so a shadow offset or blur beyond
the local clip is removed. The shared raster contract first renders children
through the local clip, derives the shadow from that clipped source, and then
clips the shadow only through the parent mask. A group with both a clip and an
`OuterShadow` therefore differs between SVG and PNG. The golden nests a clipped
group inside a separate effected group, so it does not exercise this ordering.

### D3, the golden still does not exercise shadow blur

`crates/rdocx/src/svg.rs:1445`

The representative golden now covers the required element families, but its
only shadow has `blur: 0.0`. Changing the production `stdDeviation` conversion
or breaking nonzero blur leaves the 0.99 SSIM gate unchanged. The other
nonzero-shadow tests inspect strings and never compare pixels. The golden gate
therefore still does not prove the visible lowering of the one supported effect
against the PNG backend.

### D4, a safe link can still contain an XML-invalid scalar

`crates/rdocx/src/svg.rs:693`
`crates/rdocx/src/svg.rs:752`

Link validation rejects Unicode control characters, while attribute escaping
only replaces XML metacharacters. XML 1.0 noncharacters U+FFFE and U+FFFF are
not Rust control characters, so a target such as
`https://example.test/\u{FFFE}` passes the scheme check and is emitted unchanged.
The resulting SVG is not a valid XML 1.0 document. Pass 1 sanitized visible
text, but the same scalar validation must cover emitted attribute values or
reject the link with a diagnostic.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx svg::tests -- --nocapture` passed all 12 focused
  renderer tests, including the calibrated 150 dpi resvg 0.48.1 oracle.
- `cargo tree` confirms `base64` 0.23.1 is a normal `rdocx` dependency and
  exact `resvg` 0.48.1 is development-only.
- The new dependency graph declares Rust versions no later than 1.85, below
  the workspace MSRV of 1.93.
- `git diff --check` passed before this review record was written.

## Not found

Pass 1 D2 is fixed by an explicit resvg resolver from every synthetic family to
the exact loaded `FontId` and collection face. Pass 1 D3 now has a stable TTC
face-index diagnostic. Pass 1 D4 is fixed by shared stop sorting, NaN handling,
clamping, and last-repeated-stop selection. Pass 1 D6 is fixed because every
shadow branch reads `SourceAlpha` independently and merges once with the source.
Pass 1 D7 is fixed for visible text through deterministic XML-scalar
replacement and diagnostics.

No additional defect was found in page dimensions, normal and deterministic
cache selection, out-of-range handling, font first-use order, searchable text
placement under the approved scalar-count rule, solid geometry, fill rules,
gradient geometry, stroke caps, joins and dashes, PNG and JPEG embedding,
relative and reviewed-scheme link filtering apart from D4, recursive child
order, transforms, marked-content recursion, group opacity, independent shadow
branches, unsupported paint and effect diagnostics, ordered layout and lowering
diagnostics, source-document preservation, public result shape, binding-surface
containment, runtime and development dependency placement, MSRV, deterministic
serialization, or the approved private-module structure. No trait, generic,
feature, crate, or test binary was added.
