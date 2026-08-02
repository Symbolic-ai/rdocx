# F-085, Typeface resolution

**Status**: completed
**Sprint**: S20
**Size**: S
**Depends on**: F-065

## Problem

Theme font collections expose Latin, East Asian, complex-script, and
supplemental faces at `crates/oxml-drawing/src/theme.rs:477`, while text fonts
retain their typeface tokens at
`crates/oxml-drawing/src/text/paragraph.rs:581`. No resolver turns tokens such
as `+mn-lt` into a concrete face. The current run model also does not infer a
script, so supplemental lookup needs an explicit script input.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Theme" and "Text".
- `docs/hld/07-inheritance-and-resolution.md`, "Fonts" and "The resolver".
- `docs/hld/14-development-backlog.md`, "F-085, Typeface resolution".

## Approach

Add `font.rs` to `rpptx-layout` with a concrete method:

```rust
pub fn resolve_typeface(
    &self,
    typeface: &str,
    script: Option<&str>,
) -> String;
```

Resolve the symmetrical major and minor Latin, East Asian, and complex-script
tokens. For a recognised token and supplied script, select the first matching
supplemental font in the chosen collection. When no override matches, fall
back to the token's Latin, East Asian, or complex-script base face. Preserve an
ordinary font and an unknown theme-like token unchanged. First-match behavior
for duplicate supplemental scripts follows their retained document order.

Keep script detection outside this story. Callers that know the text script
pass its ISO 15924 tag. Callers without that information get the base face.
Use the theme reference already stored by F-081, with no new trait or generic.

## Rejected alternatives

- Infer script from Unicode in the resolver. Script segmentation belongs to
  later text shaping and is not represented by the current run model.
- Return an error for an unknown theme-like token. Preserving an unfamiliar
  token is forward-compatible and matches ordinary typeface behavior.
- Add a font-provider trait. There is only one theme font collection
  implementation today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `minor_latin_theme_token_resolves_to_minor_latin_typeface` | `+mn-lt` resolves to the theme minor Latin face |
| unit | `major_and_minor_tokens_select_each_base_face` | All six major/minor Latin, East Asian, and complex-script aliases resolve correctly |
| unit | `supplemental_script_face_overrides_the_base` | A matching script uses the selected collection's supplemental face |
| unit | `missing_script_override_falls_back_to_the_base` | Missing or unknown script retains the token-specific base face |
| unit | `ordinary_and_unknown_typefaces_pass_through` | Non-theme and unfamiliar theme-like values remain unchanged |
| unit | `duplicate_supplemental_scripts_use_document_order` | The first retained matching entry wins deterministically |

The backlog test gate is named explicitly:
`minor_latin_theme_token_resolves_to_minor_latin_typeface`.

## HLD impact

- `docs/hld/07-inheritance-and-resolution.md`

## Risk routing

- A new module or file. `src/font.rs` is justified by the current F-085
  implementation and requires the shared explicit approval recorded in F-081.

## Hash harness

Expected to be unchanged. Typeface token lookup is not connected to the Word
renderer.

## Implementation checklist

- [x] Add the six major and minor theme-token mappings.
- [x] Apply first-match supplemental script overrides.
- [x] Add base-face fallback and pass-through behavior.
- [x] Add focused tests for all token and script policies.
- [x] Document the explicit script input and fallback in the HLD.

## Open questions

None. The only shared blocker is explicit approval for the new crate and source
files recorded in F-081.
