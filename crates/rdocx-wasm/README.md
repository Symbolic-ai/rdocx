# rdocx-wasm

`rdocx-wasm` runs a focused DOCX workflow in browser and bundler applications.
It creates or opens documents from bytes, performs basic edits, preserves the
complete package on serialization, and exports DOCX, PDF, HTML, or Markdown
without a server.

## Capabilities

- Create a document or open complete DOCX bytes.
- Add paragraphs, headings, bold paragraphs, and tables.
- Extract text, count paragraphs, and replace literal placeholders.
- Export DOCX, deterministic PDF, HTML, HTML fragments, and Markdown.

## Use it when

Build the local `@tensorbee/rdocx-wasm` package for browser or bundler projects.
It is deliberately unpublished to npm and crates.io.

## Relationship

The wrapper owns a real `rdocx::Document` and uses deterministic bundled fonts for PDF output.

## Example

```javascript
import init, { WasmDocument } from "@tensorbee/rdocx-wasm";

await init();
const doc = new WasmDocument();
const bytes = doc.toDocxBytes();
```

Build the checked bundler package locally:

```sh
wasm-pack build --target bundler --scope tensorbee crates/rdocx-wasm --out-name rdocx_wasm
```
