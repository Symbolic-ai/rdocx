# S15 sprint review, pass 2

**Reviewed**: `sprint/s15` against `93cb5d19272aa18cbbf780e2fb5fc422d077a88d`, 16 files, 2,210 changed lines, crates: `oxml-drawing`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The S15 definition holds. Pass 1 B1 is resolved because raw-attribute
filtering now removes only exact unqualified modelled keys at
`crates/oxml-drawing/src/theme.rs:1386`, and modelled lookup applies the same
exact rule at `crates/oxml-drawing/src/theme.rs:1414`. The regression at
`crates/oxml-drawing/src/theme.rs:131` proves that `x:name`, its `xmlns:x`
declaration, and a raw child using that namespace survive a structural
round-trip. It also proves that `x:name` cannot satisfy a missing unqualified
`@name` at `crates/oxml-drawing/src/theme.rs:150`.

The integrated `oxml-drawing` suite reports 86 passed, zero failed, and two
ignored explicit PowerPoint oracle tests. The pinned no-repair result remains
recorded with PowerPoint 16.104, plist build 16.104.25121423, and AppleScript
build 1214 at `docs/sprints/AS_BUILT.md:1833`. The Word
`tint_shade_modifiers` regression passes, released rdocx source and manifests
remain absent from the sprint diff, and a fresh hash-harness check reports all
28 entries unchanged.

The M7 corpus gate is not marked met. Its approved contract carries every
`a:txBody` and `a:spPr` structural round-trip to F-067 at S16 entry, after that
story creates the external corpus harness, at
`docs/hld/14-development-backlog.md:443` and
`docs/hld/14-development-backlog.md:569`. This transfer is part of the S15
definition at `docs/sprints/CURRENT_SPRINT.md:50` and remains mandatory before
M8 model work begins.

## Not found

- Interaction: the corrected F-065 parser feeds the F-066 projection without
  altering the active Word parser, tint and shade function, layout input, or
  rendering path.
- Duplication: no duplicate parser, writer, attribute helper, or adapter exists
  in the sprint delta.
- Layering: the only new edge is the documented
  `oxml-drawing -> rdocx-oxml` Theme adapter exception at
  `crates/oxml-drawing/Cargo.toml:15`. The reverse edge is absent and the Cargo
  graph remains acyclic.
- Harness: the baseline manifest has no sprint delta, and a fresh integrated
  check reports all 28 entries unchanged.
- Gate: the structural theme, schema-order, raw-preservation, Office-default,
  adapter, legacy Word, and unchanged-hash evidence all pass. The external
  corpus boundary is recorded as carried, not represented by inline fixtures.
- Documentation: the PowerPoint pin, unpublished development boundary, and
  approved corpus-gate transfer agree across the plans, current sprint, HLD,
  backlog, and completion record.
- Dependencies: `rdocx-oxml` is the only new dependency and its named consumer
  is the F-066 conversion at `crates/oxml-drawing/src/theme.rs:488`.
- Public surface: the theme model, parser, writer, default, accessors, error,
  and conversion are called for directly by F-065 or F-066.
