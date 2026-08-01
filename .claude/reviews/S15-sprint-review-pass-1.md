# S15 sprint review, pass 1

**Reviewed**: `sprint/s15` against `93cb5d19272aa18cbbf780e2fb5fc422d077a88d`, 15 files, 2,108 changed lines, crates: `oxml-drawing`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, namespaced extension attributes collide with modelled attributes

`crates/oxml-drawing/src/theme.rs:1357`
`crates/oxml-drawing/src/theme.rs:1387`

Both the raw-attribute filter and the modelled-attribute lookup compare only an
attribute's local name. OOXML's modelled `name`, `typeface`, and `script`
attributes are unqualified. An extension attribute such as `x:name` can
therefore satisfy the required `@name` lookup and is then removed from the raw
attribute set. A namespace declaration such as `xmlns:name` is affected by the
same collision, which can leave a preserved raw child with an unbound prefix.
The next write either changes the modelled value, drops extension data, or
emits invalid XML. This breaks the sprint's unsupported-XML preservation
contract at `docs/sprints/CURRENT_SPRINT.md:42`. Match modelled attributes by
their exact unqualified key, preserve qualified attributes, and add a
round-trip regression with a colliding qualified attribute and a raw child that
uses its namespace.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The S15 definition does not fully hold until B1 is fixed. The pinned Office
theme structural test is present at
`crates/oxml-drawing/src/theme.rs:32`, and the explicit PowerPoint acceptance
test asserts version 16.104, plist build 16.104.25121423, and AppleScript build
1214 at `crates/oxml-drawing/src/theme.rs:28`. The tracked worker result records
that the generated theme opened in that build without repair at
`docs/sprints/AS_BUILT.md:1833`. The adapter comparison covers all twelve
colours and both Latin fonts at `crates/oxml-drawing/src/theme.rs:149`, released
rdocx source and manifests are absent from the sprint diff, and the integrated
record reports all 28 hashes unchanged at `docs/sprints/AS_BUILT.md:1853`.

The M7 corpus gate is not marked met. Its approved contract now carries every
`a:txBody` and `a:spPr` structural round-trip to F-067 at S16 entry, after that
story creates the external corpus harness, as recorded at
`docs/hld/14-development-backlog.md:443` and
`docs/hld/14-development-backlog.md:569`. That transfer is consistent with the
S15 definition at `docs/sprints/CURRENT_SPRINT.md:50` and is not a separate
finding. It remains mandatory before M8 model work begins.

## Not found

- Interaction: the F-066 projection consumes the F-065 model without changing
  the active Word parse, tint, shade, layout, or rendering paths.
- Duplication: no second theme parser, format-list helper, or adapter was added
  within the sprint delta.
- Layering: the only new edge is the documented
  `oxml-drawing -> rdocx-oxml` Theme adapter exception at
  `crates/oxml-drawing/Cargo.toml:15`. The reverse edge is absent and the Cargo
  graph is acyclic.
- Harness: the hash manifest has no sprint delta, feature commits defer the
  consolidated check explicitly, and the sprint record reports all 28 entries
  unchanged.
- Documentation: the PowerPoint pin and the approved corpus-gate transfer agree
  across the plans, current sprint, testing strategy, backlog, and completion
  record.
- Dependencies: `rdocx-oxml` is the only new dependency and its present
  consumer is the exact F-066 `From` implementation at
  `crates/oxml-drawing/src/theme.rs:461`.
- Public surface: the exported theme module, error, model types, slot accessors,
  parser, writer, default, and conversion are all called for by F-065 or F-066.
