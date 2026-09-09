# rdocx-pdf

`rdocx-pdf` preserves the former Word-family renderer import path while
applications migrate to [`oxml-pdf`](https://docs.rs/oxml-pdf). Its public API
is an exact re-export of the shared fixed-output backend.

## Capabilities

- Existing `rdocx_pdf` imports continue to compile.
- Exact `oxml-pdf` functions and types.
- No duplicate renderer implementation or document parsing.
- A direct migration path to `oxml-pdf` or `rdocx::Document`.

## Use it when

Use this crate only while migrating an existing dependency. New code should use
`oxml-pdf`, or call PDF and PNG rendering directly on
[`rdocx::Document`](https://docs.rs/rdocx).

## Relationship

The shim forwards the shared renderer API without owning document layout or
package behavior.

## Example

```rust,no_run
use rdocx_pdf::render_to_pdf;

let renderer = render_to_pdf;
let _ = renderer;
```

```toml
[dependencies]
rdocx-pdf = "0.13.1"
```

For new code, replace both the dependency and the import with `oxml-pdf` and
`oxml_pdf`.
