# F-016, Length re-export

**Status**: approved
**Sprint**: S32.2
**Size**: S
**Depends on**: F-013, F-X005

## Problem

F-013 stages `Length` in the shared crate, but the public rdocx facade still
declares and exports its original module at `crates/rdocx/src/lib.rs:26` and
`crates/rdocx/src/lib.rs:34`. Keeping both definitions would create distinct,
incompatible Rust types with the same API.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and "Facade
  conventions".
- `docs/hld/11-migration-plan.md`, "The facade trick" and "Order of
  operations".
- `docs/hld/14-development-backlog.md`, "F-016, Length re-export".

## Approach

Add a direct `oxml-core` dependency to `rdocx`, delete
`crates/rdocx/src/length.rs`, remove the private module declaration, and replace
`pub use length::Length` with `pub use oxml_core::Length`. Keep the public
`rdocx::Length` path and every call site unchanged. F-X005 must first make
`oxml-core` 0.1.1 available to released-package archive verification.

## Rejected alternatives

- Keep a wrapper `rdocx::Length`. It would be a forwarding-only type and would
  make shared units incompatible at API boundaries.
- Rewrite callers to `oxml_core::Length`. The story requires zero call-site
  churn and preserves the shipped facade.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cargo check -p rdocx --all-targets` | The facade and every current call site compile unchanged |
| regression | existing `Length` unit and rdocx integration tests | Constructor, accessor, and truncation behaviour survives the re-export |
| integration | scoped diff inspection | Only `lib.rs`, `Cargo.toml`, and the deleted `length.rs` are involved |

The backlog test gate is a compiling workspace with no call-site changes.

## HLD impact

- `docs/hld/11-migration-plan.md`

Record the completed consumer boundary after the implementation lands.

## Risk routing

- Crate dependency graph. Confirm `rdocx -> oxml-core` and no reverse edge.
- Public API of a published crate. Confirm `rdocx::Length` remains source
  compatible, resolve `oxml-core` 0.1.1 from the registry, run package checks,
  and assert archive sizes.
- File move with no behaviour change. Require zero caller edits and an
  unchanged deterministic hash harness.
- Unit conversion. Run all moved truncation tests without changing casts.

## Hash harness

Expected to remain unchanged. Any delta blocks integration.

## Implementation checklist

- [ ] Add the direct workspace dependency.
- [ ] Replace the module export with the shared type re-export.
- [ ] Delete the duplicate implementation without editing callers.
- [ ] Run focused rdocx, package, workspace, and hash checks.

## Open questions

None. F-X005 publishes `oxml-core` 0.1.1 before this consumer cutover.
