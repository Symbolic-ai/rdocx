# rdocx-py

PyO3 implementation of the Python `rdocx` package for reading, editing, and rendering DOCX files.

## Use it when

Use the installed Python distribution from Python applications. This Cargo package is an unpublished binding implementation.

## Relationship

It wraps the real `rdocx` facade and shares binding conventions with `rpptx-py` through `oxml-py-support`.

## Example

```python
from rdocx import Document

doc = Document()
doc.add_paragraph("Hello from Python")
doc.save("hello.docx")
```
