# rdocx-html

`rdocx-html` converts parsed Word content to a complete HTML document, an HTML
fragment, or Markdown. It works from semantic WordprocessingML and does not
run the page layout engine. Use the high-level [`rdocx`](https://docs.rs/rdocx)
facade when starting from a DOCX file.

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
rdocx-html = "0.4"
```
