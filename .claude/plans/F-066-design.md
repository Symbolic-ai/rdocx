# F-066, The rdocx Theme adapter

**Status**: completed
**Sprint**: S15
**Size**: S
**Depends on**: F-065

## Problem

The released Word theme at `crates/rdocx-oxml/src/theme.rs:9` exposes twelve
optional RGB strings and Latin major and minor fonts. `rdocx-layout` consumes
that exact projection. F-065 introduces a richer shared theme, but no adapter
can produce the stable Word projection without duplicating mapping logic.

F-066 must provide the specified conversion while leaving the released Word
parser, tint and shade behaviour, layout input, and rendering path unchanged.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and its single theme
  adapter exception.
- `docs/hld/05-drawingml-model.md`, "What already exists", "Do not touch the
  Word path", and "Theme".
- `docs/hld/11-migration-plan.md`, "Order of operations", "Preserve behaviour,
  do not improve it", and "What happens to the published crates".
- `docs/hld/14-development-backlog.md`, "F-066, The rdocx Theme adapter".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

After F-065 is integrated, implement the exact trait in
`crates/oxml-drawing/src/theme.rs`:

```rust
impl From<&CT_OfficeStyleSheet> for rdocx_oxml::theme::Theme
```

Add `rdocx-oxml.workspace = true` to `crates/oxml-drawing/Cargo.toml` and update
`Cargo.lock`. This is the single documented `oxml-drawing -> rdocx-oxml`
dependency. Do not add `oxml-drawing` to the released crate.

Project all twelve colour slots through
`theme_elements.color_scheme.color(slot)`. Convert concrete sRGB values to
uppercase six-digit strings. For system colours, prefer `lastClr` and fall
back to the symbolic value, matching the legacy parser. Leave fields `None`
when the shared colour form has no concrete legacy RGB representation.

Copy only `major_font.latin.typeface` and `minor_font.latin.typeface`. Ignore
the format scheme, other font collections, transforms, and preserved raw XML
because the legacy type has no matching fields. Do not change or install the
adapter into the active Word parse or render path.

## Rejected alternatives

- Implement in `rdocx-oxml`. That reverses the documented edge and makes a
  released crate consume unpublished code.
- Replace `Theme::from_xml` with the shared parser. That changes the active
  released path and exceeds this adapter story.
- Change `LayoutInput.theme` to the shared type. The architecture retains the
  existing type specifically to avoid that churn.
- Correct `apply_tint_shade`. Its legacy behaviour is deliberately frozen.
- Add a feature flag or forwarding wrapper. Neither has a present second use,
  and both add unnecessary cases.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `shared_theme_adapter_matches_the_legacy_theme_projection` | Both parsers agree on twelve colours and Latin major and minor fonts |
| unit | `shared_theme_adapter_does_not_project_unresolved_colour_forms` | Unsupported legacy colour projections remain `None` |
| regression | existing `tint_shade_modifiers` | Word's current 0 to 255 tint and shade results remain unchanged |
| golden | `python3 scripts/hash_harness.py --check` | All 28 hashes remain unchanged |

The backlog test gate is: the hash harness is unchanged.

Focused checks include `cargo test -p oxml-drawing`, `cargo test -p
rdocx-oxml`, dependency trees in both directions, the released
`rdocx-oxml` package dry-run, and the hash harness.

## HLD impact

None. The trait, dependency direction, retained layout type, frozen Word colour
maths, and publication boundary are already documented.

## Risk routing

- Theme colour, tint, shade, and colour mapping: read
  `docs/hld/05-drawingml-model.md`. Confirm the Word tint and shade function is
  unchanged, run its regression, and require all 28 hashes unchanged.
- Crate dependency graph and a new cross-family `use`: read
  `docs/hld/03-architecture.md`. Add only `oxml-drawing -> rdocx-oxml`, inspect
  both dependency trees, and prove there is no cycle or reverse edge.

## Hash harness

Expected unchanged, all 28 entries. The adapter is not installed into the
active Word path. Any delta is undeclared and blocks integration.

## Implementation checklist

- [x] Wait until F-065 is integrated and use its approved concrete API.
- [x] Add the one documented dependency to `oxml-drawing` and update the lockfile.
- [x] Implement the trait in the shared theme module.
- [x] Project twelve concrete colours and two Latin fonts only.
- [x] Add inline comparison and unresolved-colour tests.
- [x] Leave released rdocx source and manifests unchanged.
- [x] Prove the dependency direction and released package dry-run.
- [x] Run focused checks and the unchanged hash gate.

## Open questions

None after F-065's public field and accessor contract is approved.
