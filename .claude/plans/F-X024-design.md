# F-X024, Move the theme adapter into rdocx-oxml

**Status**: completed
**Sprint**: S42
**Size**: M
**Depends on**: F-X020

## Problem

`crates/oxml-drawing/src/theme.rs:515` holds

```rust
impl From<&CT_OfficeStyleSheet> for rdocx_oxml::theme::Theme
```

which is the single documented exception to the rule in
`docs/hld/03-architecture.md` that nothing in `oxml-*` may depend on `rdocx-*`
or `rpptx-*`.

That one edge makes the two publication trains mutually dependent:

```
rdocx-layout (stable)     -> oxml-layout (incubating)
oxml-drawing (incubating) -> rdocx-oxml (stable)
```

`.github/workflows/publish.yml` publishes one train per tag, seven stable
packages on `v*` and fourteen incubating on `rpptx-v*`. That works only when the
train publishing first depends on an already-published version of the other.
S39 satisfied it because only one train moved.

S41 broke both public APIs, so both must bump, and the loop closes:

- **Stable first** would need `rdocx-layout 0.7.0` to build against the
  published `oxml-layout 0.2.0`. Its code uses `TextSegment::note`, which does
  not exist there. It will not compile.
- **Incubating first** would force `oxml-drawing 0.3.0` to keep pinning
  `rdocx-oxml 0.6.0`, the only published version. It would then ship an adapter
  producing 0.6.0's `Theme` while `rdocx-layout 0.7.0` expects 0.7.0's `Theme`.
  Two semver-incompatible copies of the Word model in one graph, and the one
  cross-family integration point stops type-checking.

The cycle is at the train level, not the package level. `oxml-drawing ->
rdocx-oxml -> oxml-core` is acyclic. It is the train split that cannot express
it.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and "The one exception",
  which this story deletes rather than amends.
- `docs/hld/05-drawingml-model.md`, "Theme", for what the adapter converts.
- `docs/hld/14-development-backlog.md`, "F-X024, Move the theme adapter into
  rdocx-oxml".

## Approach

Move the `impl` from `oxml-drawing` to `rdocx-oxml`. The orphan rule permits it:
`Theme` is local to `rdocx-oxml` and `CT_OfficeStyleSheet` is the foreign type,
so the crate that owns the target type may implement the conversion.

- `crates/rdocx-oxml/Cargo.toml` gains `oxml-drawing.workspace = true`.
- `crates/oxml-drawing/Cargo.toml` drops `rdocx-oxml.workspace = true`.
- The `impl` and its helpers move into `crates/rdocx-oxml/src/theme.rs`.
- The three test uses in `oxml-drawing/src/theme.rs` move with it, since they
  exercise the conversion rather than anything else in that file.

After the move the dependency runs one way, stable to incubating, so the rule in
`03-architecture.md` becomes absolute and its exception paragraph is deleted
rather than reworded.

**Accepted cost, stated rather than buried.** `rdocx-oxml` gains a dependency on
`oxml-drawing`, so a Word-only consumer now compiles DrawingML. That was chosen
over deleting the adapter, which has no caller in the workspace today and would
have been the smaller diff. The adapter exists so `rdocx-layout`'s
`LayoutInput.theme` does not churn when PresentationML themes reach Word layout,
and keeping it preserves that intent.

**Public API.** `oxml-drawing` loses a trait implementation and `rdocx-oxml`
gains it. Both are breaking, and both crates are already taking a breaking minor
bump in this sprint, so the change costs no additional version churn.

## Rejected alternatives

- **Delete the adapter.** Smallest diff, zero cost, no caller today. Rejected
  by explicit decision: it is the documented bridge for a planned integration,
  and removing it would have to be undone later on the other side anyway.
- **Feature-gate the dependency in `oxml-drawing`.** An optional dependency
  still has to resolve at publish time, so the cycle survives. It also adds a
  feature flag with no consumer, which the structural rules discourage.
- **Publish all 21 packages from one tag in topological order.** Correct, and a
  larger change to `publish.yml` that leaves the architectural exception in
  place to cause the same problem next time.
- **Release neither train.** Defers the decision without removing the cause.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `the_theme_adapter_produces_the_same_theme_after_the_move` | Converting the office default `CT_OfficeStyleSheet` yields a `Theme` equal to the one the pre-move adapter produced, field for field |
| regression | `no_shared_crate_depends_on_a_format_crate` | `cargo metadata` shows no `oxml-*` package with a `rdocx-*` or `rpptx-*` dependency, which is the invariant the story exists to restore |
| unit | the moved tests | The three conversion tests that lived in `oxml-drawing` still pass from their new home |

**Test gate**, from the backlog: the conversion regression plus the dependency
invariant.

## HLD impact

- `docs/hld/03-architecture.md`. "The dependency rule" loses the phrase "with
  exactly one documented exception below", and the paragraph describing that
  exception is deleted. The dependency diagram's `oxml-drawing -> rdocx-oxml`
  edge is redrawn as `rdocx-oxml -> oxml-drawing`.
- `CLAUDE.md`. The layout table names the same exception and must lose it too.

## Risk routing

Two rows match.

- **Crate dependency graph, a new `use` across families.** This is the row's
  exact subject. The check it demands, that no `oxml-*` depends on `rdocx-*` or
  `rpptx-*`, becomes a test rather than a convention.
- **Public API of a published crate.** A trait impl moves between two published
  crates. Both are taking a breaking minor bump this sprint, so no extra
  version consequence. Stated at completion.

The layout row does not match: no rendering, pagination or shaping code is
touched, and the harness must stay flat.

## Hash harness

**Expected unchanged.** The conversion is byte-for-byte the same code in a
different crate, and nothing calls it in a rendering path today. A delta would
mean the move changed behaviour and blocks the story.

## Implementation checklist

- [x] Record the pre-change harness state
- [x] Capture the current conversion output as the regression's expectation
- [x] Move the impl and its helpers into `rdocx-oxml/src/theme.rs`
- [x] Swap the two manifest dependencies
- [x] Move the three conversion tests
- [x] Add the dependency-invariant regression
- [x] Update `03-architecture.md` and `CLAUDE.md` to drop the exception
- [x] Full suite, harness, WASM, no-default-features
- [x] `/microscope F-X024 --working`
- [x] `/verify`

## Open questions

None. The one decision, move rather than delete, was taken explicitly.
