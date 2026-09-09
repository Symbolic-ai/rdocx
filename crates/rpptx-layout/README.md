# rpptx-layout

`rpptx-layout` resolves PresentationML inheritance into renderer-ready slides.
It combines slide, layout, master, theme, placeholder, relationship, and
resource state into owned shapes with concrete visual properties.

## Capabilities

- Slide, layout, master, theme, and placeholder inheritance.
- Concrete shape styles, backgrounds, transforms, and geometry.
- Text, bullets, spacing, autofit, and script-aware fonts.
- Tables, source-scoped media, charts, links, and timeline evaluation.

## Use it when

Use this crate when resolving slide, layout, master, theme, and placeholder state into a renderable slide model. Use `rpptx` for complete deck operations.

## Relationship

It sits between `rpptx-oxml` and `rpptx-render`. It does not own package I/O or
emit final PDF or raster output.

## Example

```rust,no_run
use rpptx_layout::{FlattenedSource, ScopedMediaIds};

let media = ScopedMediaIds::default();
assert_eq!(media.get(FlattenedSource::Slide, "rId1"), None);
```

Add `rpptx-layout = "0.11.0"` to your dependencies. See the [resolver API](https://docs.rs/rpptx-layout) for the resolved-slide contract.
