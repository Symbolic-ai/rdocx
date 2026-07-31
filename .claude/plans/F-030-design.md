# F-030, Decouple line.rs

**Status**: completed
**Sprint**: S06
**Size**: L
**Depends on**: F-029

## Problem

`crates/rdocx-layout/src/line.rs` is otherwise format-neutral but imports
`CT_TabStop`, `ST_Jc`, `ST_TabJc`, `ST_Underline`, `ST_TabLeader`, and `Twips`
from `rdocx-oxml`. Copying that file unchanged would violate the dependency
rule for `oxml-layout`, while changing the released line breaker now would mix
staging with the deferred consumer cutover.

## Spec reference

- `docs/hld/01-glossary.md`, "Units".
- `docs/hld/03-architecture.md`, "The dependency rule" and "Why these seams".
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/11-migration-plan.md`, "The one piece of real API design" and
  "Preserve behaviour, do not improve it".
- `docs/hld/14-development-backlog.md`, "F-030, Decouple line.rs".

## Approach

Copy `line.rs` into the staged crate and replace every docx type with concrete
owned layout types in that same module:

```rust
pub struct TabStop {
    pub pos_pt: f64,
    pub align: TabAlign,
    pub leader: Option<TabLeader>,
}

pub enum Align { Start, Center, End, Justify, Distribute }
pub enum TabAlign { Left, Center, Right, Decimal, Bar }
pub enum TabLeader { None, Dot, Hyphen, Underscore, Heavy, MiddleDot }
pub enum Underline { Single, Words, Double, Thick, Dotted, Dash, DotDash, DotDotDash, Wave }
pub enum LineSpacing { Single, Multiple(f64), Exact(f64), AtLeast(f64) }
```

`TextSegment` uses `Option<Underline>`. `LineBreakParams` uses
`Vec<TabStop>`, `LineSpacing`, `Option<Align>`, and a new `wrap: bool` whose
default is true. Existing automatic breaks are guarded by `wrap`, while forced
line, page, and column breaks always remain effective. Tab positions and exact
or minimum spacing are stored directly in points. `Multiple` stores a factor,
so no staged type retains twips or the stringly `line_rule` field.

Rewrite the existing 11 tests against the owned types and add one regression
for `wrap: false`. Add `unicode-linebreak` to the staged crate only. Leave the
released `rdocx-layout` file, call sites, manifest, and future docx converter
unchanged.

## Rejected alternatives

- Depend on `rdocx-oxml`. This violates the format-neutral dependency rule and
  makes the staged crate unusable by PowerPoint.
- Add the rdocx conversion layer now. The backlog explicitly defers that work
  to the released consumer cutover.
- Keep twips plus a string spacing rule. The approved boundary uses points and
  makes the four spacing modes explicit.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `the_eleven_line_tests_pass_with_owned_types` | Empty lines, fitting, wrapping, explicit breaks, indents, spacing, and tab behavior match the copied implementation. |
| unit | `line_spacing_variants_preserve_existing_height_rules` | Single, multiple, exact, and at-least spacing compute their specified heights. |
| regression | `wrap_false_only_breaks_on_an_explicit_break` | Width overflow stays on one line while an explicit break still splits the result. |
| unit | `tab_stops_use_point_positions_and_owned_leaders` | The next owned tab stop produces the existing width and leader glyph. |
| regression | `released_line_breaker_is_unchanged` | No `rdocx-layout` source, manifest, or consumer changes appear in the story diff. |

The backlog test gate is that `line.rs`'s 11 tests rewritten on the new types
pass, and the hash harness is unchanged.

## HLD impact

None. The owned type boundary, explicit wrap behavior, and deferred converter
are already specified.

## Risk routing

- Unit conversion. Preserve twips-to-point truncation behavior only at the
  future rdocx boundary. The staged line breaker accepts already converted
  point values and tests exact and fractional inputs without rounding.
- Layout and line breaking. Run every copied test in deterministic font mode,
  add the no-wrap regression, and require the 28-entry hash harness to remain
  unchanged.
- Crate dependency graph. Run `cargo tree -p oxml-layout --edges normal` and
  confirm no `rdocx-*` or `rpptx*` edge appears after adding line breaking.
- Public API in an unpublished staged crate. Limit the surface to the specified
  concrete types and existing line-breaker API, then run package and archive
  size checks with publication disabled.
- New module and file copy. F-030 explicitly authorizes staged `line.rs`. Diff
  it against the released source and account for only owned types, explicit
  spacing, wrap behavior, imports, and rewritten tests.

## Hash harness

Expected to remain unchanged. No released call site consumes staged `line.rs`.

## Implementation checklist

- [x] Copy `line.rs` into the staged crate and add its existing dependency.
- [x] Replace all docx types with the approved concrete layout types.
- [x] Replace twips and string spacing rules with `LineSpacing`.
- [x] Add `wrap: bool` with true as the compatibility default.
- [x] Rewrite all 11 copied tests and add spacing and no-wrap coverage.
- [x] Confirm no released consumer change and run dependency, package, and hash
      riders.

## Open questions

None.
