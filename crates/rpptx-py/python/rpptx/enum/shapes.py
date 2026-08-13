"""Shape enumerations needed by the documented examples."""

from enum import IntEnum


class MSO_SHAPE(IntEnum):
    """Supported Microsoft Office preset shape identifiers."""

    PENTAGON = 51
    CHEVRON = 52


__all__ = ["MSO_SHAPE"]
