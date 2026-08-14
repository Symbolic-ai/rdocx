# rdocx-pdf

`rdocx-pdf` is a deprecated compatibility shim for the shared
[`oxml-pdf`](https://docs.rs/oxml-pdf) renderer. Existing imports continue to
work because the public functions are exact re-exports. New code should use
`oxml-pdf`, or call PDF and PNG rendering directly on
[`rdocx::Document`](https://docs.rs/rdocx).

```rust,no_run
use rdocx_pdf::render_to_pdf;

let renderer = render_to_pdf;
let _ = renderer;
```

```toml
[dependencies]
rdocx-pdf = "0.5"
```

For new code, replace both the dependency and the import with `oxml-pdf` and
`oxml_pdf`.
