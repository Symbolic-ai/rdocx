# rpptx-chart

`rpptx-chart` keeps existing presentation chart imports compiling while callers
migrate to [`oxml-chart`](https://docs.rs/oxml-chart). Every public item is an
exact re-export of the shared ChartML model and renderer.

## Capabilities

- Existing `rpptx_chart` imports continue to compile.
- Exact `oxml-chart` models, validators, and geometry.
- No duplicate chart implementation or presentation package policy.
- A direct dependency and import migration path.

## Use it when

Use this crate only while migrating an existing dependency. New code should
use `oxml-chart` directly.

## Relationship

The shim preserves the former PowerPoint-family package name without owning
ChartML parsing, serialization, validation, or rendering.

## Example

```rust,no_run
use rpptx_chart::AxisId;

let category_axis = AxisId::new(10_000_001)?;
let value_axis = AxisId::new(10_000_002)?;
assert_ne!(category_axis, value_axis);
# Ok::<(), rpptx_chart::ChartError>(())
```

```toml
[dependencies]
rpptx-chart = "0.11.0"
```

For new code, replace both the dependency and the import with `oxml-chart` and
`oxml_chart`.
