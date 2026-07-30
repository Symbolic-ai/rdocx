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
| S01 | M1 | 6 | 6 | 0 | 10 | 6 | Completed with no carries |
| S02 | M1 | 6 | 6 | 0 | 8 | 6 | Completed M1 and published rdocx 0.4.1 |
| S03 | M2 | 5 | 3 | 2 | 8 | 3 | F-015 and F-016 carried to S04 to keep rdocx 0.5.0 independent of unpublished oxml-core |
| S04 | M2 | 7 | 4 | 3 | 9 | 4 | F-015, F-016, and F-022 carried to S32.2 so development crates remain unpublished until PowerPoint is complete |
| S05 | M3 | 4 | 4 | 0 | 8 | 4 | Completed isolated unpublished oxml-media staging, with F-027 and F-028 remaining planned for S32.2 |

## Completed features

| F-ID | Sprint | Size | Est. days | Actual days | Completed | Notes |
|------|--------|------|-----------|-------------|-----------|-------|
| F-001 | S01 | M | 2 | 1 | 2026-07-29 | Deterministic bundled-font path |
| F-002 | S01 | S | 1 | 1 | 2026-07-29 | Rust 1.97.1 toolchain pin |
| F-003 | S01 | L | 4 | 1 | 2026-07-29 | Initial 28-entry hash baseline |
| F-004 | S01 | S | 1 | 1 | 2026-07-29 | Caladea licence and notice |
| F-005 | S01 | S | 1 | 1 | 2026-07-29 | Collision-safe image suffix allocation |
| F-006 | S01 | S | 1 | 1 | 2026-07-29 | Safe JPEG standalone-marker walk |
| F-007 | S02 | S | 1 | 1 | 2026-07-30 | Relationship-based core properties |
| F-008 | S02 | M | 2 | 1 | 2026-07-30 | 61 non-consuming setter twins |
| F-009 | S02 | M | 2 | 1 | 2026-07-30 | Thread-safe two-mode layout cache |
| F-010 | S02 | S | 1 | 1 | 2026-07-30 | Fourteen crates.io names reserved |
| F-011 | S02 | S | 1 | 1 | 2026-07-30 | Unit truncation behavior pinned |
| F-012 | S02 | S | 1 | 1 | 2026-07-30 | Published and tagged rdocx 0.4.1 |
| F-013 | S03 | M | 2 | 1 | 2026-07-30 | Unpublished shared OOXML core |
| F-014 | S03 | M | 2 | 1 | 2026-07-30 | Shared schema unit types |
| F-017 | S03 | M | 2 | 1 | 2026-07-30 | Shared app and custom properties |
| F-018 | S04 | M | 2 | 1 | 2026-07-30 | Unpublished format-neutral OPC package |
| F-019 | S04 | S | 1 | 1 | 2026-07-30 | PresentationML package constants |
| F-020 | S04 | M | 2 | 1 | 2026-07-30 | Code-built PowerPoint OPC proof |
| F-021 | S04 | S | 1 | 1 | 2026-07-30 | Canonical ZIP entry normalization |
| F-023 | S05 | M | 2 | 1 | 2026-07-30 | Dependency-free image format sniffing |
| F-024 | S05 | L | 4 | 1 | 2026-07-30 | Safe image metadata and DPI probing |
| F-025 | S05 | S | 1 | 1 | 2026-07-30 | Collision-free shared media naming |
| F-026 | S05 | S | 1 | 1 | 2026-07-30 | Dependency-free native EMU sizing |

## Velocity

Recalculated at each sprint close. The backlog assumes about 2 stories per week
sustained, and the whole plan is sized at roughly 390 developer-days. If the
first three sprints diverge from that by more than 30 percent, replan rather
than absorb it.

Stories per week is completed stories divided by actual days, multiplied by
five working days.

| Window | Stories | Days | Stories/week |
|--------|---------|------|--------------|
| S01 | 6 | 6 | 5.00 |
| S02 | 6 | 6 | 5.00 |
| S03 | 3 | 3 | 5.00 |
| S04 | 4 | 4 | 5.00 |
| S05 | 4 | 4 | 5.00 |

## Escalation record

Logged when an escalation trigger from `.claude/WORKFLOW.md` fires, with what
was done about it. Empty is the expected state.

| Date | Trigger | F-ID or sprint | Response |
|------|---------|----------------|----------|
| 2026-07-30 | Three-sprint velocity variance exceeded 30 percent | S01 to S03 | Reforecast 366 remaining estimated days to 45 to 50 active weeks, retain dependency-defined boundaries, and recalibrate after S06 |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S05 | Record 4 actual days against 8 estimated, retain the 45 to 50 active week reforecast, and recalibrate after S06 |
