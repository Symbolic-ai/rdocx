from pathlib import Path
from typing import TYPE_CHECKING, Callable

from rpptx import Inches, Length, MSO_SHAPE, Presentation, Pt
from rpptx._rpptx import (
    Cell,
    Column,
    ColumnCollection,
    Font,
    Paragraph,
    ParagraphCollection,
    PlaceholderCollection,
    Run,
    RunCollection,
    Shape,
    ShapeCollection,
    Slide,
    SlideCollection,
    SlideLayout,
    SlideLayoutCollection,
    Table,
    TextFrame,
)


def exercise_rpptx_types(path: Path) -> None:
    presentation = Presentation(path)
    layout: SlideLayout = presentation.slide_layouts[0]
    slide: Slide = presentation.slides.add_slide(layout)
    shape: Shape = slide.shapes.add_textbox(
        Inches(1), Inches(1), Inches(4), Inches(2)
    )
    shape.text = "typed"
    paragraph: Paragraph = presentation.slides[0].shapes[-1].text_frame.paragraphs[0]
    paragraph.level = 1
    paragraph.font.bold = True
    paragraph.font.size = Pt(12)
    returned_size: Length | None = (
        presentation.slides[0].shapes[-1].text_frame.paragraphs[0].font.size
    )
    broad_shape_factory: Callable[[int, int, int, int, int], Shape] = (
        presentation.slides[0].shapes.add_shape  # type: ignore[assignment]
    )
    length: Length = Pt(12)
    points: float = length.pt
    emu: int = length.emu
    shape = presentation.slides[0].shapes.add_shape(
        MSO_SHAPE.CHEVRON, Inches(1), Inches(1), Inches(2), Inches(1)
    )
    shape = presentation.slides[0].shapes.add_table(
        1, 1, Inches(1), Inches(1), Inches(4), Inches(1)
    )
    table: Table = shape.table
    column: Column = table.columns[0]
    column.width = Inches(2)
    cell: Cell = table.cell(0, 0)
    cell.text = paragraph.text
    shapes: list[Shape] = presentation.slides[0].shapes[:]
    for current_slide in presentation.slides:
        for current_shape in current_slide.shapes:
            current_shape.has_text_frame
    package_bytes: bytes = presentation.to_bytes()
    presentation.save(path)
    package_bytes, shapes, returned_size, points, emu, broad_shape_factory


if TYPE_CHECKING:
    Cell()  # type: ignore[call-arg]
    Column()  # type: ignore[call-arg]
    ColumnCollection()  # type: ignore[call-arg]
    Font()  # type: ignore[call-arg]
    Paragraph()  # type: ignore[call-arg]
    ParagraphCollection()  # type: ignore[call-arg]
    PlaceholderCollection()  # type: ignore[call-arg]
    Run()  # type: ignore[call-arg]
    RunCollection()  # type: ignore[call-arg]
    Shape()  # type: ignore[call-arg]
    ShapeCollection()  # type: ignore[call-arg]
    Slide()  # type: ignore[call-arg]
    SlideCollection()  # type: ignore[call-arg]
    SlideLayout()  # type: ignore[call-arg]
    SlideLayoutCollection()  # type: ignore[call-arg]
    Table()  # type: ignore[call-arg]
    TextFrame()  # type: ignore[call-arg]
