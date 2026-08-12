"""Python bindings for rpptx."""

from .enum.shapes import MSO_SHAPE
from .util import Inches, Length, Pt


class RpptxError(Exception):
    """Base class for errors raised by rpptx."""


class PackageError(RpptxError):
    """An OPC package, file, or presentation operation failed."""


class XmlError(RpptxError):
    """PresentationML could not be parsed or serialized."""


class StaleElementError(RpptxError):
    """A held content handle was invalidated by structural mutation."""


from ._rpptx import Presentation

__all__ = [
    "Inches",
    "Length",
    "MSO_SHAPE",
    "PackageError",
    "Presentation",
    "Pt",
    "RpptxError",
    "StaleElementError",
    "XmlError",
]
