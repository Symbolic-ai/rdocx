# oxml-drawing

Parse, edit, resolve, and serialize reusable DrawingML content without
depending on a document format.

## Capabilities

- Typed colors with theme and color-map resolution.
- Solid, gradient, pattern, image, and no-fill models.
- Preset and custom geometry, lines, transforms, text bodies, and tables.
- Schema-ordered writing with ordered preservation of unmodelled children.

## Use it when

Use this crate when reading or writing DrawingML shared by DOCX and PPTX packages. Use `rpptx` or `rdocx` for complete documents.

## Relationship

It consumes format-neutral OOXML primitives and supplies drawing models to
presentation and rendering crates. Format-specific anchors, wrappers,
packaging, and rendering belong elsewhere. Effects and DrawingML elements that
are not typed remain preserved rather than being interpreted.

## Example

```rust,no_run
use oxml_drawing::fill::Fill;

let fill = Fill::from_xml(
    br#"<a:noFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#,
)?;
assert!(matches!(fill, Fill::NoFill(_)));
# Ok::<(), oxml_drawing::fill::FillError>(())
```

Add `oxml-drawing = "0.11.0"` to your dependencies. Browse the [typed DrawingML API](https://docs.rs/oxml-drawing) before constructing schema-level values directly.
