# rpptx-py

PyO3 implementation of the Python `rpptx` package for reading, editing, and rendering PPTX files.

## Use it when

Use the installed Python distribution from Python applications. This Cargo package is an unpublished binding implementation.

## Relationship

It wraps the real `rpptx` facade and shares paths, revisions, units, and errors through `oxml-py-support`.

## Example

```python
from rpptx import Presentation

presentation = Presentation("deck.pptx")
print(len(presentation.slides))
```
