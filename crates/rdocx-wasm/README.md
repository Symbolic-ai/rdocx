# rdocx-wasm

WebAssembly bindings for reading, creating, round-tripping, and rendering DOCX documents in JavaScript.

## Use it when

Use the generated `@tensorbee/rdocx-wasm` package in browser or bundler projects. This Cargo package remains unpublished on crates.io.

## Relationship

The wrapper owns a real `rdocx::Document` and uses deterministic bundled fonts for PDF output.

## Example

```javascript
import init, { WasmDocument } from "@tensorbee/rdocx-wasm";

await init();
const doc = new WasmDocument();
const bytes = doc.toDocxBytes();
```
