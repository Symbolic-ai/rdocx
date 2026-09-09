# rpptx-wasm

`rpptx-wasm` provides a compact browser and bundler facade for complete PPTX
byte round trips. It creates decks from the bundled template, opens existing
package bytes, adds slides by layout, and optionally produces deterministic
PDF.

## Capabilities

- Create or open a presentation from bytes.
- Serialize the complete package and count slides.
- Add a slide from a layout index.
- Keep rendering out of the default profile.
- Add deterministic PDF through the optional `render` feature.

## Use it when

Build the local `@tensorbee/rpptx-wasm` package for browser or bundler projects.
It is deliberately unpublished to npm and crates.io.

## Relationship

The wrapper owns a real `rpptx::Presentation`. Its default profile excludes rendering, while the `render` feature adds the shared renderer and bundled fonts.

## Example

```javascript
import init, { WasmPresentation } from "@tensorbee/rpptx-wasm";

await init();
const deck = new WasmPresentation();
const bytes = deck.toBytes();
```

Build the checked bundler package locally:

```sh
wasm-pack build --target bundler --scope tensorbee crates/rpptx-wasm --out-name rpptx_wasm
```
