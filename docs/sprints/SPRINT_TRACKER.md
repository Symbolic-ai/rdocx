# Sprint Tracker

Velocity log. One row per completed F-ID, appended by `/complete-feature`, plus
a per-sprint summary appended by `/close-sprint`.

Estimates come from `docs/hld/14-development-backlog.md`. Actuals are recorded
so the velocity assumption can be corrected against reality rather than
defended.

`S = 1d`, `M = 2-3d`, `L = 4-5d`, `XL = split me`.

## Per-sprint summary

| Sprint | Milestone | Planned | Done | Carried | Est. days | Actual days | Notes |
|--------|-----------|---------|------|---------|-----------|-------------|-------|
| S01 | M1 | 6 | - | - | 10 | - | in progress |

## Completed features

| F-ID | Sprint | Size | Est. days | Actual days | Completed | Notes |
|------|--------|------|-----------|-------------|-----------|-------|
| F-001 | S01 | M | 2 | 1 | 2026-07-29 | Deterministic bundled-font path |
| F-002 | S01 | S | 1 | 1 | 2026-07-29 | Rust 1.97.1 toolchain pin |
| F-003 | S01 | L | 4 | 1 | 2026-07-29 | Initial 28-entry hash baseline |
| F-004 | S01 | S | 1 | 1 | 2026-07-29 | Caladea licence and notice |
| F-005 | S01 | S | 1 | 1 | 2026-07-29 | Collision-safe image suffix allocation |
| F-006 | S01 | S | 1 | 1 | 2026-07-29 | Safe JPEG standalone-marker walk |

## Velocity

Recalculated at each sprint close. The backlog assumes about 2 stories per week
sustained, and the whole plan is sized at roughly 390 developer-days. If the
first three sprints diverge from that by more than 30 percent, replan rather
than absorb it.

| Window | Stories | Days | Stories/week |
|--------|---------|------|--------------|
| _(no data yet)_ | | | |

## Escalation record

Logged when an escalation trigger from `.claude/WORKFLOW.md` fires, with what
was done about it. Empty is the expected state.

| Date | Trigger | F-ID or sprint | Response |
|------|---------|----------------|----------|
