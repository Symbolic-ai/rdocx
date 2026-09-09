# rdocx-py

The Python `rdocx` package brings the native DOCX facade to Python
applications. It creates and opens documents, edits content, saves complete
DOCX packages, and renders PDF or page images.

## Capabilities

- File and byte-based DOCX input and output.
- Paragraphs, runs, fonts, tables, rows, and cells.
- PDF plus PNG, JPEG, and TIFF page output.
- Pythonic collections, negative indexes, iteration, and stale-handle errors.

## Use it when

Choose this binding when Python code needs the native DOCX workflow.

## Relationship

It wraps the real `rdocx::Document` and shares binding conventions with
`rpptx-py` through `oxml-py-support`. The Cargo package is an unpublished
binding implementation, not the user-facing installation target.

## Example

```python
from rdocx import Document

doc = Document()
doc.add_paragraph("Hello from Python")
doc.save("hello.docx")
```
