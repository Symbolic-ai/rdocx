# rdocx-cli

`rdocx-cli` turns DOCX files into inspectable and scriptable shell artifacts.
It extracts content, reports structure, validates packages, applies changes,
and produces fixed or flow output without an Office host.

## Capabilities

- Human-readable or JSON structure and metadata inspection.
- Text extraction across body paragraphs and table cells.
- PDF, HTML, Markdown, PNG, JPEG, and multi-page TIFF conversion.
- Page-range rendering, literal replacement, diffing, and validation verdicts.

## Use it when

Use this crate for shell automation. Use the
[`rdocx`](https://docs.rs/rdocx) library when these operations need to run
inside a Rust application.

## Relationship

The binary delegates document behavior to `rdocx` and shares path and JSON
conventions with `rpptx-cli` through `oxml-cli-support`.

## Example

```sh
cargo install rdocx-cli --version '^0.13.1'

rdocx inspect report.docx
rdocx text report.docx
rdocx convert report.docx --to pdf -o report.pdf
rdocx validate report.docx
rdocx render report.docx --page 0 -o rendered
```

Run `rdocx --help` or `rdocx <command> --help` for the complete option set.
