# 04, OPC and packaging

Owner: `oxml-opc`, with media naming in `oxml-media`.

## The package

```rust
pub struct OpcPackage {
    pub content_types: ContentTypes,
    pub package_rels:  Relationships,
    pub part_rels:     HashMap<String, Relationships>,  // key: "/ppt/slides/slide1.xml"
    pub parts:         HashMap<String, Vec<u8>>,        // same key shape
}
```

Fully in memory. Every part is decompressed at open. Part names are normalised
to a leading slash. This design is already format-neutral and is carried over
essentially unchanged from `rdocx-opc`.

**Saves are deterministic.** Both `part_rels` and `parts` are emitted in sorted
key order, so writing the same package twice produces byte-identical output.
That property is load-bearing for the round-trip corpus and must not regress.

## Generalising the constructors

The only docx-specific code in the existing crate is two constructors. They are
replaced by:

```rust
impl OpcPackage {
    /// Empty package: minimal content types, no parts, no relationships.
    pub fn new() -> Self;

    /// Package whose officeDocument relationship points at `part_name`.
    /// Package-relative, no leading slash: "word/document.xml", "ppt/presentation.xml".
    pub fn with_main_part(part_name: &str, content_type: &str) -> Self;
}

impl ContentTypes {
    /// Only the two universal defaults, "rels" and "xml".
    pub fn minimal() -> Self;
}
```

The docx presets become a short private helper in `crates/rdocx/src/document.rs`,
and the pptx presets one in `crates/rpptx/src/package.rs`.

Rejected alternatives, recorded so they are not revisited: a `PackageKind` enum
forces the leaf crate to carry every format's content-type table and grows a
variant per format. Feature-gated `new_docx` / `new_pptx` helpers make a leaf
crate feature-conditional for two functions' worth of string constants, and
features are additive so a workspace containing both compiles both anyway.

## What transfers unmodified

**`main_document_part()`** keys off the `officeDocument` relationship type,
which PowerPoint uses for `/ppt/presentation.xml`. It reads a `.pptx` today.

**`resolve_rel_target(source_part, target)`** joins a relative target against
the source part's directory and collapses `.` and `..`. It already handles
`../slideLayouts/slideLayout1.xml` from `/ppt/slides/slide1.xml` correctly, and
it clamps a traversal that escapes the root rather than allowing zip-slip.

**`rels_path_to_part_name`** and its inverse are generic path algebra.

## Relationship types

`rel_types` stays one flat module, grouped by comment. The existing thirteen
constants are kept. Added:

```rust
// Package-level. Note the package namespace, not officeDocument.
CORE_PROPERTIES, THUMBNAIL

// Shared officeDocument
EXTENDED_PROPERTIES   // docProps/app.xml
CUSTOM_PROPERTIES     // docProps/custom.xml

// PresentationML
SLIDE, SLIDE_LAYOUT, SLIDE_MASTER, NOTES_SLIDE, NOTES_MASTER,
PRES_PROPS, VIEW_PROPS, TABLE_STYLES, HANDOUT_MASTER
```

A `content_types` constants module is added alongside, so neither format crate
hand-types the long MIME strings.

Both facades resolve core properties through the package-level
`CORE_PROPERTIES` relationship and retain its normalized target. Immutable
property access leaves the source part bytes untouched. Mutable access marks
the typed `CoreProperties` model for serialization to that target with its
content-type override. A package that creates metadata without an existing
relationship uses `/docProps/core.xml` and adds the missing package
relationship. If that conventional part name is already occupied without the
core-properties relationship, serialization returns an error before changing
the package.

## Part naming

**Numeric suffixes are allocated after the greatest positive parsed suffix,
never as `count + 1`.** Deleting slide 2 and then adding a slide must not
collide with slide 3. `MediaNamer` scans positive decimal suffixes in the
requested directory and stem, including `usize::MAX`, and ignores missing,
signed, zero, nonnumeric and unrelated suffixes. Ordinary packages allocate
`1 + max(existing suffix)`. At the finite boundary, checked increment wraps
from `usize::MAX` to 1 and skips every occupied parsed suffix until a free
positive number is found. Both facades use this allocator, so allocation never
creates `image0` or overwrites an existing numbered image part.

Canonical part layouts:

```
docx                      pptx
/word/document.xml        /ppt/presentation.xml
/word/styles.xml          /ppt/slides/slideN.xml
/word/media/imageN.ext    /ppt/slideLayouts/slideLayoutN.xml
                          /ppt/slideMasters/slideMasterN.xml
                          /ppt/notesSlides/notesSlideN.xml
                          /ppt/theme/themeN.xml
                          /ppt/media/imageN.ext
                          /ppt/charts/chartN.xml
                          /ppt/embeddings/WorkbookN.xlsx
```

## Media

`oxml-media` owns everything about an image byte string.

```rust
pub enum ImageFormat { Png, Jpeg, Gif, Bmp, Tiff, Webp, Svg, Emf, Wmf }

impl ImageFormat {
    pub fn sniff(data: &[u8]) -> Option<Self>;
    pub fn from_extension(ext: &str) -> Option<Self>;
    pub fn extension(self) -> &'static str;
    pub fn content_type(self) -> &'static str;
}

/// Sniff first, fall back to the extension, default to PNG.
pub fn resolve(data: &[u8], filename: &str) -> ImageFormat;

pub struct ImageInfo {
    pub format: ImageFormat,
    pub width_px: u32, pub height_px: u32,
    pub dpi_x: Option<f64>, pub dpi_y: Option<f64>,  // None means the file declares none
    pub bit_depth: u8, pub channels: u8, pub has_alpha: bool,
}
pub fn probe(data: &[u8]) -> Option<ImageInfo>;

pub struct NativeSize {
    pub width_emu: i64, pub height_emu: i64,
}

impl ImageInfo {
    pub fn native_size(&self, default_dpi: f64) -> Option<NativeSize>;
}

pub struct MediaNamer { /* dir, stem, next */ }
impl MediaNamer {
    pub fn scan<'a>(dir: &str, stem: &str, existing: impl Iterator<Item = &'a str>) -> Self;
    pub fn next_part_name(&mut self, ext: &str) -> String;
}
```

**Sniffing beats the extension.** `resolve` uses detected image bytes before a
filename extension, so a `.png` that is really a JPEG receives the JPEG
extension and content type. Unknown bytes fall back through a recognised
filename extension and finally to PNG for compatibility.

`rdocx::Document` scans existing `/word/media/imageN.ext` parts into a
`MediaNamer` when it opens. Every body, header, footer, and raw-XML image path
uses the allocator and registers the sniffed canonical extension and content
type before adding its relationship. HTML and layout extraction resolve MIME
from the stored bytes first, so a misleading package part name cannot override
the actual image format.

**`native_size` takes the DPI rather than baking one in**, because the right
default differs by consumer: python-docx assumes 72 when a file declares none,
while Word assumes 96. Each declared finite positive axis DPI takes precedence
over the caller default. Missing or invalid declared DPI falls back per axis.
The conversion multiplies pixels by 914400 EMU per inch and truncates toward
zero. The method returns `None` if either effective DPI is not finite and
positive, or if a converted dimension is outside the `i64` range.

`NativeSize` keeps the result dependency-free and exposes explicit EMU fields.
The PresentationML picture insertion path supplies 72 for python-pptx parity
without adding an `oxml-core` edge to `oxml-media`.

`rpptx::Presentation` scans `/ppt/media/` into a content-hash `MediaStore` when
it opens. Insertion compares the complete byte string inside each hash bucket,
reuses an equal package-wide media part, and otherwise allocates the next
numbered part after the greatest occupied suffix. The sniffed canonical
extension and content type are registered with the package. Each source slide
creates or reuses its own internal image relationship to that shared part, with
a relative target resolved from the slide part name.

Slide removal considers only `/ppt/media/` targets reached from the removed
slide and its removed notes relationship scopes. A candidate part is deleted
only when no remaining internal package relationship reaches it. Pre-existing
orphan media outside that candidate set is left untouched. The facade rebuilds
its content-hash media index after the graph change, so a later insertion sees
the surviving package state.

Header parsing is lifted from the PDF crate, where `jpeg_dimensions` and the
PNG IHDR reader are currently private. The JPEG walk classifies SOI, TEM and
RST0 through RST7 as standalone markers with no length field, and EOI terminates
the codestream. Length-bearing segments are validated for a present length of
at least two bytes and bounds within the input before the walk indexes or
advances. Fill bytes and truncated input return safely. Preserve these
invariants when the reader moves to `oxml-media`.

## Package integrity

Both facades expose a `validate()` that is cheap, non-panicking, and run
automatically under `debug_assertions` before `save`. It checks dangling
relationship ids, missing content-type overrides, relationship targets that
resolve to no part, and orphan media. `rpptx` adds its own presentation-specific
checks, listed in `06-presentationml-model.md`.
