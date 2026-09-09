# oxml-core

Share exact OOXML units, XML helpers, and package-independent document property
models.

## Capabilities

- Lengths in twips, EMUs, half-points, centipoints, and physical units.
- Angles and thousandths-of-a-percent values used by OOXML schemas.
- Core, application, and custom property parsing and writing.
- XML 1.0 lexical validation, namespace handling, decoding, and raw capture.

## Use it when

Use this crate when implementing OOXML infrastructure shared by WordprocessingML and PresentationML. End-user document applications should prefer `rdocx` or `rpptx`.

## Relationship

Higher-level `oxml-*`, `rdocx-*`, and `rpptx-*` crates build on these primitives
without introducing a format-specific dependency here. These are focused OOXML
building blocks, not a general DOM or schema implementation. Unit conversions
follow the repository's exact truncation rules.

## Example

```rust,no_run
use oxml_core::Length;

let page_width = Length::inches(8.5);
assert_eq!(page_width.to_twips(), 12_240);
```

Add `oxml-core = "0.11.0"` to your dependencies. See the [API documentation](https://docs.rs/oxml-core) for XML and property types.
