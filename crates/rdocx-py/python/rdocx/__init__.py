"""Python bindings for rdocx."""


class StaleElementError(Exception):
    """A held content handle was invalidated by structural mutation."""


from ._rdocx import Document, Paragraph, ParagraphCollection, Run, RunCollection

__all__ = [
    "Document",
    "Paragraph",
    "ParagraphCollection",
    "Run",
    "RunCollection",
    "StaleElementError",
]
