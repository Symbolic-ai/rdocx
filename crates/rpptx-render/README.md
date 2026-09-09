# rpptx-render

`rpptx-render` lowers resolved presentation content into the shared positioned
page model. It assembles source-scoped relationships and media, lays out one
page per slide, and offers deterministic bundled-font output for stable
rendering pipelines.

## Capabilities

- `RenderInput` assembly for slides, media, fonts, and metadata.
- Source-scoped slide, layout, and master relationships.
- Shapes, text, tables, images, groups, and backgrounds lowered to page frames.
- Structured diagnostics and `LayoutResult` output for downstream backends.

## Use it when

Use this crate when integrating presentation layout below the `rpptx` facade.
Applications normally call deterministic output methods on
`rpptx::Presentation`.

## Relationship

It consumes `rpptx-layout` output and produces `oxml-layout` page frames. It
does not write PDF or raster files. Those formats belong to downstream output
backends and the high-level facade.

## Example

```rust,no_run
use rpptx_render::{RelScope, RelScopes};

let relationships = RelScopes::default();
let missing = relationships.get(RelScope::Slide, "rId1");
assert!(missing.is_err());
```

Add `rpptx-render = "0.11.0"` to your dependencies. See the [rendering API](https://docs.rs/rpptx-render) for input and output types.
