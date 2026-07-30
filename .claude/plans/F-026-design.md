# F-026, native_size with explicit DPI

**Status**: approved
**Sprint**: S05
**Size**: S
**Depends on**: F-024

## Problem

`ImageInfo` will expose pixel dimensions and optional file DPI, but callers
need a physical OOXML size. The media contract at
`docs/hld/04-opc-and-packaging.md:145` specifies
`native_size(default_dpi)` returning `Length`, while the architecture at
`docs/hld/03-architecture.md:65` says `oxml-media` has no dependencies. Those
requirements cannot both hold because `Length` lives in `oxml-core`.

## Spec reference

- `docs/hld/01-glossary.md`, "Units".
- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "Media".
- `docs/hld/14-development-backlog.md`, "F-026, native_size with explicit DPI".

## Approach

Keep `oxml-media` dependency-free. Add the concrete public
`NativeSize { width_emu: i64, height_emu: i64 }` and make
`ImageInfo::native_size(default_dpi)` return `Option<NativeSize>`. The method
uses each declared axis DPI when it is finite and positive, otherwise it uses
the finite positive caller default. It converts pixels through 914400 EMU per
inch and truncates toward zero to match the repository's pinned unit behavior.
Invalid effective DPI returns `None` rather than producing infinite or
saturated dimensions. No rdocx call site changes in this sprint.

## Rejected alternatives

- Bake in 72 DPI. Word and python-docx use different defaults, so the caller
  must choose explicitly.
- Round EMU values. Existing unit constructors deliberately truncate toward
  zero, and a rounding change would shift output at the later cutover.
- Return raw inches. OOXML consumers need exact EMU dimensions and should not
  repeat the conversion.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `declared_dpi_overrides_the_caller_default` | A 96 DPI image uses 96 even when the default is 72 and yields the expected EMU. |
| unit | `missing_axis_dpi_uses_the_explicit_default` | Each absent axis falls back independently. |
| regression | `native_size_truncates_fractional_emu_toward_zero` | Conversion preserves pinned truncation semantics. |
| unit | `invalid_effective_dpi_returns_none` | Zero, negative, NaN, and infinite effective DPI cannot produce a size. |

The test gate is a 96 DPI PNG probed at `default_dpi = 72` yields the expected
EMU.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/14-development-backlog.md`

Only the files needed to resolve the contradictory return type and dependency
statement change. The implementation behavior is otherwise already specified.

## Risk routing

- Unit conversion and EMU. Read `docs/hld/01-glossary.md`, "Units", and
  preserve the pinned truncation semantics from `CLAUDE.md`. Run positive and
  fractional conversion cases and require the hash harness to remain
  unchanged.
- Crate dependency graph. Inspect `cargo tree -p oxml-media --edges normal`
  and confirm the crate remains dependency-free with no `rdocx-*` or `rpptx*`
  edge.
- Public API of a reserved crate. State the chosen return type in the HLD, run
  package and size checks, and keep the crate at 0.0.0 with publication
  disabled.

## Hash harness

Expected to remain unchanged. The native-size API has no released consumer in
S05.

## Implementation checklist

- [ ] Resolve the return type and dependency contract.
- [ ] Add per-axis declared-DPI and caller-default selection.
- [ ] Convert pixels to EMU with truncation toward zero.
- [ ] Reject invalid effective DPI.
- [ ] Add the declared-DPI, fallback, truncation, and invalid-input tests.
- [ ] Run dependency, package, unit-conversion, and unchanged-hash riders.

## Open questions

None. The user chose dependency-free `NativeSize`, preserving the dependency
DAG owned by `docs/hld/03-architecture.md`.
