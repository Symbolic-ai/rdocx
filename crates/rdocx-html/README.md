# rdocx-html

`rdocx-html` converts assembled WordprocessingML content into portable web
text. It produces complete HTML, embeddable fragments, and Markdown directly
from semantic Word content without running pagination.

## Capabilities

- Complete HTML document output and body-only fragments.
- Markdown conversion from the same semantic input.
- Style, numbering, link, and image projection through `HtmlInput`.
- A flow-output path with no page-layout or fixed-output dependency.

## Use it when

Use this crate when an application already owns `HtmlInput`. Use the high-level
[`rdocx`](https://docs.rs/rdocx) facade when starting from a DOCX package or
when HTML import is required.

## Relationship

`rdocx` prepares the `HtmlInput` consumed here. This conversion path is
independent of pagination and PDF rendering.

## Example

```rust,no_run
use rdocx_html::{HtmlInput, HtmlOptions, to_html_document, to_markdown};

fn export(input: &HtmlInput) -> (String, String) {
    (
        to_html_document(input, &HtmlOptions::default()),
        to_markdown(input),
    )
}
```

```toml
[dependencies]
rdocx-html = "0.13.1"
```
