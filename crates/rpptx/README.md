# rpptx

High-level Rust facade for reading, editing, creating, validating, and rendering PowerPoint-compatible presentations.

## Use it when

Use this crate for complete PPTX applications. Choose the lower-level `rpptx-oxml` only for schema-level PresentationML work.

## Relationship

The facade owns package preservation and delegates layout, charts, and rendering to the specialised `rpptx-*` crates.

## Example

```rust,no_run
use rpptx::Presentation;

let deck = Presentation::new()?;
let bytes = deck.to_bytes()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
