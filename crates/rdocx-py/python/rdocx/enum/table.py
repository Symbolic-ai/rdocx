"""Table-related public enumerations."""

from enum import IntEnum


class WD_TABLE_ALIGNMENT(IntEnum):
    """Table horizontal alignment."""

    LEFT = 0
    CENTER = 1
    RIGHT = 2


class WD_CELL_VERTICAL_ALIGNMENT(IntEnum):
    """Vertical alignment within a table cell."""

    TOP = 0
    CENTER = 1
    BOTTOM = 3


__all__ = ["WD_TABLE_ALIGNMENT", "WD_CELL_VERTICAL_ALIGNMENT"]
