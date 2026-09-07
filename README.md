# rdocx

[![CI](https://github.com/tensorbee/rdocx/actions/workflows/ci.yml/badge.svg)](https://github.com/tensorbee/rdocx/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/rdocx.svg)](https://crates.io/crates/rdocx)
[![docs.rs](https://docs.rs/rdocx/badge.svg)](https://docs.rs/rdocx)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/rdocx.svg)](LICENSE)
[![MSRV: 1.93](https://img.shields.io/badge/MSRV-1.93-blue.svg)](https://blog.rust-lang.org/2026/01/09/Rust-1.93.0.html)

rdocx is a Rust toolkit for creating, reading, editing, preserving, and
rendering Word documents. The same document object can produce DOCX, PDF, PNG,
HTML, and Markdown without shelling out to Microsoft Word or LibreOffice.

The public API deliberately distinguishes what can be authored today from what
can only be read or preserved. The summary below is backed by the
[canonical modern DOCX capability matrix](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix).
The matrix is the authority when this overview and an individual property need
different levels of detail.

## Capability status

The classifications have precise meanings:

- **complete** means the public facade supports the matrix operations and the
  modeled state survives save and reopen.
- **partial** means part of the property family is public. The matrix names the
  owner of the remaining work.
- **unsupported** means the current facade cannot perform the operation. The
  matrix names its owner.
- **preserve-only** means input is retained without a modeled authoring
  surface.
- **permanent-non-goal** means the operation is intentionally outside the
  product boundary.

Reading, mutation, and preservation are separate claims. A partial row does not
mean that every property in that family is publicly creatable.

| Major category | Current public boundary | Classification | Matrix |
|---|---|---|---|
| DOCX package I/O | Open, save, and byte serialization | complete | [DOCX-001](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Styles | Paragraph, character, and table style graphs have a bounded public surface | partial | [DOCX-011](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Numbering | Lists expose a useful subset of levels and instances | partial | [DOCX-012](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Headers and footers | Per-section default, first, and even stories have a bounded surface | partial | [DOCX-017](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Tables | Grids, widths, borders, and layout mode are partly public | partial | [DOCX-022](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Paragraphs | Ordinary text, alignment, spacing, indentation, and pagination | complete | [DOCX-029](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Runs | Fonts, emphasis, color, language, and ordinary inline content | complete | [DOCX-031](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Fields | Simple and complex field construction has a bounded surface | partial | [DOCX-045](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Forms | Content control creation and lifecycle are partly public | partial | [DOCX-052](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Collaboration | Comments, replies, people, and modern metadata are partly public | partial | [DOCX-060](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Drawings | Picture anchors, wrapping, crop, transforms, and effects are partly public | partial | [DOCX-064](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Equations | Transitional OfficeMath authoring and conversion | complete | [DOCX-079](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Unknown safe producer XML | Retained byte for byte when it is not modeled | preserve-only | [DOCX-080](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Legacy Word formats | Binary DOC, Word 2003 XML, and pre-OOXML payloads | permanent-non-goal | [DOCX-082](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |
| Executable payloads | VBA, ActiveX, OLE, add-in, and embedded application execution | permanent-non-goal | [DOCX-083](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix) |

The [active backlog](docs/sprints/BACKLOG.md) records the owner and status of
every partial or unsupported row. The
[current sprint](docs/sprints/CURRENT_SPRINT.md) is the shorter delivery view.

## Installation

Most Rust applications need only the facade:

```toml
[dependencies]
rdocx = "0.13.1"
```

Bundled metric-compatible fonts are always available through deterministic
rendering. The default feature also discovers system fonts. Disable default
features when an application must use only the bundled set:

```toml
[dependencies]
rdocx = { version = "0.13.1", default-features = false }
```

The workspace requires Rust 1.93 or newer and uses edition 2024.

## Examples

### Create a document

```rust,no_run
use rdocx::{Document, Length};

let mut document = Document::new();
document.add_paragraph("Quarterly report");

let mut summary = document.add_paragraph("");
summary.add_run("Status: ").bold(true);
summary.add_run("approved");

let mut table = document.add_table(1, 2);
assert!(table.set_column_width(0, Length::inches(2.0)));
assert!(table.set_column_width(1, Length::inches(4.0)));

document.save("report.docx")?;
# Ok::<(), rdocx::Error>(())
```

### Read and update a document

```rust,no_run
use rdocx::Document;
use std::collections::HashMap;

let mut document = Document::open("template.docx")?;
for paragraph in document.paragraphs() {
    println!("{}", paragraph.text());
}

let mut replacements = HashMap::new();
replacements.insert("{{status}}", "Approved");
document.replace_all(&replacements);
document.save("approved.docx")?;
# Ok::<(), rdocx::Error>(())
```

### Render and export

```rust,no_run
use rdocx::Document;

let document = Document::open("report.docx")?;
document.save_pdf("report.pdf")?;
let first_page_png = document.render_page_to_png_deterministic(0, 150.0)?;
let html = document.to_html();
let markdown = document.to_markdown();

assert!(first_page_png.as_ref().is_some_and(|png| !png.is_empty()));
assert!(!html.is_empty());
assert!(!markdown.is_empty());
# Ok::<(), rdocx::Error>(())
```

## Surfaces

| Surface | Boundary |
|---|---|
| [rdocx](https://docs.rs/rdocx) | Native Rust facade for complete DOCX packages, authoring, conversion, and rendering |
| [rdocx-oxml](https://docs.rs/rdocx-oxml) | Typed WordprocessingML for callers that already own the lower-level model |
| [rdocx-layout](https://docs.rs/rdocx-layout) | Word flow layout for callers that already own layout input |
| [rdocx-html](https://docs.rs/rdocx-html) | HTML and Markdown conversion from parsed Word content |
| [rdocx-cli](crates/rdocx-cli/README.md) | Shell commands for inspection, conversion, validation, rendering, replacement, and diffing |
| [rdocx-py](crates/rdocx-py/README.md) | Python binding with a deliberately narrower facade |
| [rdocx-wasm](crates/rdocx-wasm/README.md) | Browser and JavaScript binding with a deliberately narrower facade |

The lower-level compatibility shims and all workspace crates are described in
their crate-local READMEs. The
[binding specification](docs/hld/10-bindings-spec.md#native-word-facade-stability)
defines where Python, WASM, and CLI intentionally expose less than native Rust.

## CLI

Install the CLI version that belongs to the same stable family:

```sh
cargo install rdocx-cli --version '^0.13.1'
```

Common commands:

```sh
rdocx inspect report.docx
rdocx text report.docx
rdocx convert report.docx --to pdf -o report.pdf
rdocx convert report.docx --to html -o report.html
rdocx convert report.docx --to md -o report.md
rdocx replace report.docx --placeholder "Draft" --value "Final" -o final.docx
rdocx diff before.docx after.docx
```

## Evidence-based alternatives

This table limits itself to functionality, license, language runtime, and host
dependencies stated by each project's official documentation. It makes no
performance, popularity, or footprint claim.

| Library | Officially documented functionality | License | Runtime or host boundary | Official evidence |
|---|---|---|---|---|
| python-docx | Create, read, and update DOCX with paragraphs, tables, and inline pictures | MIT | Python with declared Python dependencies | [Quickstart](https://python-docx.readthedocs.io/en/latest/user/quickstart.html), [project](https://github.com/python-openxml/python-docx) |
| docx-rs | Generate and parse DOCX from Rust, WebAssembly, and JavaScript | MIT | Rust, with optional WebAssembly and JavaScript surfaces | [project](https://github.com/bokuweb/docx-rs) |
| docx4j | Create, edit, save, and convert Open XML packages | Apache-2.0 | Java with one selected JAXB implementation | [project](https://github.com/plutext/docx4j) |
| Aspose.Words | Create, modify, convert, render, and print documents without Office automation | Commercial license or limited evaluation | Java without Microsoft Word as a host application | [product overview](https://docs.aspose.com/words/java/product-overview/), [licensing](https://docs.aspose.com/words/java/licensing/) |

## Roadmap and status

- [Modern DOCX capability matrix](docs/hld/02-scope-and-non-goals.md#modern-docx-capability-matrix)
- [Development backlog](docs/hld/14-development-backlog.md#milestone-23-from-scratch-business-documents-about-22-weeks)
- [Current sprint](docs/sprints/CURRENT_SPRINT.md)
- [Delivery backlog](docs/sprints/BACKLOG.md)
- [Unreleased changelog](CHANGELOG.md#unreleased)

## License

Licensed under either of:

- MIT license ([LICENSE](LICENSE) or <https://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 (<https://www.apache.org/licenses/LICENSE-2.0>)

at your option.
