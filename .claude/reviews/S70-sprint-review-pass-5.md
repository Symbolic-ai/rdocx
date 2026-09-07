# S70 sprint review, pass 5

**Reviewed**: `sprint/s70` against
`59c47fb0877e2b1139f1d2c690bac57e1f8fb138`, 43 files, 6,575 changed
lines, crates: `rdocx`, `rdocx-layout`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Extension decision

This pass explicitly extends the default three-pass bound for the required
close-boundary review after Wave 4. Pass 4 was clean before F-X086 integrated.
The additional pass reviews the final combined sprint delta and is not a
continuation of unresolved findings.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M23 end gate is not due in S70. The backlog records 3 completed and 21
pending M23 stories at `docs/sprints/BACKLOG.md:41`, so this review makes no
claim that all five private references can already be generated from the blank
public facade.

The S70 gate holds. Reusable restart still requires the exact retained context,
font trace, provenance mode, and body note-reference sequence at
`crates/rdocx-layout/src/engine.rs:2033`. Whole-body equality remains explicitly
length aware at `crates/rdocx-layout/src/engine.rs:2047`. A body-length change
may select the last safe prefix checkpoint, while retained tail attachment
requires absent provenance or equal length at
`crates/rdocx-layout/src/engine.rs:2090` and
`crates/rdocx-layout/src/engine.rs:2100`.

The sourced insert and delete gate proves at most three recomputed pages plus
fresh layout and source-map equality at
`crates/rdocx-layout/src/engine.rs:12603`. The page-identity regression permits
only a contiguous retained prefix at
`crates/rdocx-layout/src/engine.rs:12647`, and the Enter, adjacent merge, and
selection-delete matrix carries the same bound at
`crates/rdocx-layout/src/engine.rs:12695`. The integrated full gate passed with
all 49 hash entries unchanged.

## Not found

- Interaction: F-X084 note filtering remains part of reusable-record equality,
  F-X085 supplies one memo to both length-sensitive scans, and F-X086 changes
  only prefix and tail eligibility at
  `crates/rdocx-layout/src/engine.rs:2033` through
  `crates/rdocx-layout/src/engine.rs:2106`.
- Duplication: the final wave adds three focused inline regressions to the
  existing engine test module and no second restart mechanism.
- Layering: no Cargo manifest or lockfile changed, and no shared crate gained a
  format dependency.
- Harness: every completed story records an unchanged 49-entry result. The
  F-X086 delivery record cites the exact integrated verification SHA at
  `docs/sprints/AS_BUILT.md:12324`.
- Gate: sourced structural edits now retain bounded prefix work without
  retaining stale tail provenance, matching the current rendering contract at
  `docs/hld/08-rendering-spec.md:816`.
- Docs: the final implementation updates exactly the three HLD files listed by
  its approved plan. The testing contract names the complete operation and
  identity matrix at `docs/hld/12-testing-strategy.md:784`.
- Dependencies: no dependency or feature changed.
- Surface: F-X086 changes private restart selection and inline tests only. It
  adds no public API.
