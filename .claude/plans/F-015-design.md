# F-015, rdocx-oxml becomes a facade

**Status**: completed
**Sprint**: S32.2
**Size**: S
**Depends on**: F-013, F-X005

## Problem

After F-013 stages the shared implementation, `rdocx-oxml` still owns duplicate
copies through the module declarations at `crates/rdocx-oxml/src/lib.rs:14`.
Changing its hundreds of namespace-helper call sites would create broad churn
with no behavioural value. F-X005 must first make the shared implementation
resolvable from crates.io by released-package archive verification.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "What
  stays put".
- `docs/hld/11-migration-plan.md`, "The facade trick" and "Order of
  operations".
- `docs/hld/14-development-backlog.md`, "F-015, rdocx-oxml becomes a facade".

## Approach

Add the `oxml-core` dependency to `rdocx-oxml`. Replace the shared module
declarations with the exact re-export block specified by the migration plan:
public `core_properties`, `raw_xml`, and `units`, public error types, and a
crate-visible `xml_text`. Keep `W_NS` and `W_PREFIX` in `namespace.rs`, while
re-exporting `matches_local_name`, `R_NS`, and `MC_NS` from
`oxml_core::xml`.

Delete the five staged source duplicates from `rdocx-oxml`. Do not modify any
of the existing helper call sites or WordprocessingML modules.

## Rejected alternatives

- Rewrite imports to `oxml_core`. The approved facade exists specifically to
  avoid that migration risk.
- Leave duplicate modules permanently. Two implementations would drift and
  violate the one-owner architecture.
- Replace `rdocx-oxml` with a forwarding-only crate. It remains the permanent
  owner of the WordprocessingML model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | scoped `git diff --stat` assertion | Only `lib.rs`, `namespace.rs`, and `Cargo.toml` change in place and five shared files are deleted |
| integration | `cargo test -p rdocx-oxml` | All existing internal paths compile and tests pass through the facade |
| regression | existing workspace tests | No public rdocx path or caller changes |

The backlog test gate is the specified mechanical diff shape plus green
workspace tests.

## HLD impact

- `docs/hld/11-migration-plan.md`
- `docs/hld/14-development-backlog.md`

Remove the stale numeric call-site count and record the published shared-crate
prerequisite while preserving the exact facade contract.

## Risk routing

- Crate dependency graph. Confirm the edge is `rdocx-oxml -> oxml-core` and
  that `oxml-core` has no reverse dependency.
- Parser and serializer ownership cutover. Run the existing namespace, raw XML,
  child-order, and round-trip tests through the facade without changing parser
  or serializer bodies.
- Public API of a published crate. Confirm the existing paths remain source
  compatible, resolve `oxml-core` 0.1.2 from the registry, run both package
  checks, and assert archives remain below 10 MiB.
- File move with no behaviour change. Assert zero call-site edits and an
  unchanged deterministic hash harness.

## Hash harness

Expected to remain unchanged. Any output delta is a defect.

## Implementation checklist

- [x] Add the one-way `oxml-core` dependency.
- [x] Apply the exact facade and namespace re-exports.
- [x] Delete the five duplicate source files.
- [x] Assert the mechanical diff shape and zero call-site churn.
- [x] Run focused crate, package, workspace, and hash checks.

## Open questions

None. F-X005 publishes `oxml-core` 0.1.2 before this consumer cutover.
