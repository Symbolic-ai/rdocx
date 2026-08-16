# oxml-media

Dependency-free media format detection, collision-safe naming, and intrinsic image sizing for OOXML packages.

## Use it when

Use this crate when an OOXML writer must identify or size image bytes without decoding the complete image.

## Relationship

DOCX and PPTX package facades use these helpers before adding media parts and relationships.

## Example

```rust,no_run
use oxml_media::{ImageFormat, resolve};

let format = resolve(b"\x89PNG\r\n\x1a\n", "image.bin");
assert_eq!(format, ImageFormat::Png);
```

Add `oxml-media = "0.3.0"` to your dependencies. See the [API documentation](https://docs.rs/oxml-media) for supported formats and sizing functions.
