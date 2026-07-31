# F-035, The walk helper

**Status**: approved
**Sprint**: S07
**Size**: S
**Depends on**: F-034

## Problem

The staged output model has no common traversal seam for nested groups. The
rendering specification identifies three later PDF collection passes that
would silently skip grouped fonts, images, and links if each continued to
iterate only the top-level page elements. F-035 must make recursive traversal
and transform accumulation explicit before those consumers migrate.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Why `Group` is the whole design" and "The
  recursion hazard".
- `docs/hld/12-testing-strategy.md`, "oxml-layout".
- `docs/hld/13-risks-and-open-questions.md`, "R7, group-blind collection
  passes".
- `docs/hld/14-development-backlog.md`, "F-035, The walk helper".

## Approach

Add the specified helper beside the output representation and export it from
the crate root:

```rust
pub fn walk(
    elements: &[PositionedElement],
    f: &mut impl FnMut(&PositionedElement, &Transform),
);
```

Start with `Transform::IDENTITY`. Visit every non-group leaf exactly once in
depth-first document order. Do not invoke the callback for group containers.
For a group, compose the child-local transform before the accumulated
parent-to-page transform:

```rust
let accumulated = group.transform.then(parent_accumulated);
```

Then recurse into its children with that value. This follows F-031's
self-first `Transform::then` contract and F-034's child-local to parent
direction.

## Rejected alternatives

- Yield group containers as well as leaves. The helper is specified to flatten
  groups for collection passes.
- Return an allocated vector. A callback avoids cloning elements and
  accumulating a second tree-sized collection.
- Add a traversal trait. There is one representation and one implementation
  today.
- Put the helper in a new module. Its behavior is short and inseparable from
  the output enum it traverses.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `three_deep_groups_yield_every_leaf_once_with_the_correct_accumulated_transform` | Every nested leaf appears once in document order with the hand-computed transform. |
| unit | `nested_group_transform_order_applies_child_before_parent` | Composition follows the self-first `then` contract. |
| unit | `walk_does_not_yield_group_nodes` | Only leaf elements reach the callback. |
| unit | `walk_passes_identity_for_root_leaves` | Top-level leaves receive the identity transform. |

The backlog test gate is that a three-deep nested group yields every leaf
exactly once with the correct accumulated transform.

## HLD impact

None. The rendering specification already defines the helper signature,
flattening behavior, and accumulation requirement.

## Risk routing

- Layout geometry. Use the hand-computed nested transform regression, run the
  consolidated hash gate in deterministic font mode, and require all 28
  entries to remain unchanged.

The consolidated gate also runs both `oxml-layout` feature modes, dependency
inspection, and a package dry-run with the existing sub-10 MiB bound. The
package must not be published.

## Hash harness

Expected to remain unchanged. The helper has no released consumer.

## Implementation checklist

- [ ] Wait for integrated F-034.
- [ ] Add and export the callback-based leaf traversal helper.
- [ ] Accumulate child-local transforms in the documented order.
- [ ] Add the three-deep regression and focused traversal tests.
- [ ] Run the scoped checks and consolidated sprint riders.

## Open questions

None, assuming F-034 adopts the documented child-local to parent transform.
