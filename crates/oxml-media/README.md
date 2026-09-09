# oxml-media

Identify OOXML media safely, derive intrinsic image sizing, and allocate
collision-free package part names without dependencies.

## Capabilities

- Magic-byte and filename identification for common image formats.
- Native sizing for PNG, JPEG, GIF, BMP, and WebP.
- Parameter-free MIME token validation and sequential media naming.
- Limited MP3, RIFF WAVE, and ISO base media signature checks.

## Use it when

Use this crate when an OOXML writer must identify or size image bytes, validate
a package content type, or check MP3, RIFF WAVE, and ISO base media signatures
without decoding the complete payload.

## Relationship

DOCX and PPTX package facades use these helpers before adding media parts and
relationships. This crate identifies containers and metadata. It does not
decode media or identify codecs.

## Example

```rust,no_run
use oxml_media::{ImageFormat, resolve};

let format = resolve(b"\x89PNG\r\n\x1a\n", "image.bin");
assert_eq!(format, ImageFormat::Png);
```

Add `oxml-media = "0.11.0"` to your dependencies. See the [API documentation](https://docs.rs/oxml-media) for supported formats and sizing functions.
