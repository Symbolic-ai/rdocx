# oxml-layout

Backend-neutral layout frames, paths, paints, text fragments, font discovery, and deterministic bundled fonts.

## Use it when

Use this crate when producing or consuming positioned layout output independently of DOCX or PPTX. Most applications should use a document facade and renderer instead.

## Relationship

`rdocx-layout` and `rpptx-render` produce this model. `oxml-pdf` consumes it.

## Example

```rust,no_run
use oxml_layout::Color;

let accent = Color::from_hex("3366CC");
assert_eq!((accent.r, accent.g, accent.b), (0.2, 0.4, 0.8));
```

Add `oxml-layout = { version = "0.4.0", default-features = false }` to your dependencies. Enable the default `system-fonts` feature only when host font discovery is intended.
