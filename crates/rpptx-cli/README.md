# rpptx-cli

`rpptx-cli` makes complete PPTX workflows available to shell scripts. It
inspects and extracts deck content, validates package invariants, compares or
replaces text, and produces deterministic fixed output.

## Capabilities

- Human-readable and JSON inspection.
- Slide-order text extraction and recursive outlines.
- PDF, PNG, JPEG, and multi-page TIFF conversion.
- Selected-slide rendering, thumbnails, text diff, replacement, and validation.

## Use it when

Use the CLI for shell automation. Use `rpptx` when the same operations belong inside a Rust application.

## Relationship

It uses `oxml-cli-support` for shared command conventions and the real `rpptx` facade for document behavior.

## Example

```sh
cargo install rpptx-cli --version '^0.11.0'
rpptx inspect deck.pptx --json
rpptx convert deck.pptx --to pdf -o deck.pdf
rpptx thumbnail deck.pptx -o thumbnail.png
```

Run `rpptx --help` or `rpptx <command> --help` for the complete command surface.
