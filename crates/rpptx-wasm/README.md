# rpptx-wasm

WebAssembly bindings for PPTX round trips and optional deterministic PDF rendering.

## Use it when

Use the generated `@tensorbee/rpptx-wasm` package in browser or bundler projects. This Cargo package remains unpublished on crates.io.

## Relationship

The wrapper owns a real `rpptx::Presentation`. Its default profile excludes rendering, while the `render` feature adds the shared renderer and bundled fonts.

## Example

```javascript
import init, { WasmPresentation } from "@tensorbee/rpptx-wasm";

await init();
const deck = new WasmPresentation();
const bytes = deck.toBytes();
```
