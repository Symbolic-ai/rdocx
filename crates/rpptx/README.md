# rpptx

`rpptx` gives Rust applications one `Presentation` for complete
PowerPoint-compatible package workflows. Open or create decks, edit content,
retain unmodelled package data, validate invariants, and produce deterministic
presentation, notes, handout, PDF, and animation outputs.

## Capabilities

- Open, create, validate, and save PPTX or PPSX packages.
- Encrypt and sign packages through the opt-in `agile-encryption` and
  `digital-signatures` features.
- Add, remove, move, duplicate, and transfer slides.
- Author and edit text, pictures, shapes, groups, tables, charts, comments,
  SmartArt text, and media.
- Produce PDF, PDF/A, resolved slide frames, notes, handouts, and animations.
- Import HTML, ODP, and PDF through explicit facade APIs.

## Use it when

Use this crate for complete PPTX applications. Choose the lower-level `rpptx-oxml` only for schema-level PresentationML work.

## Relationship

The facade owns package preservation and delegates part modeling, inheritance
resolution, charts, and layout lowering to the specialist `rpptx-*` and
`oxml-*` crates.

## Example

```rust,no_run
use rpptx::Presentation;

let deck = Presentation::new()?;
let bytes = deck.to_bytes()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

```toml
[dependencies]
rpptx = "0.11.0"
```

Enable native encryption and signing explicitly:

```toml
[dependencies]
rpptx = { version = "0.11.0", features = ["agile-encryption", "digital-signatures"] }
```
