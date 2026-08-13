from pathlib import Path
from typing import TYPE_CHECKING, assert_type

from rdocx import (
    Cell,
    CellCollection,
    CellParagraphCollection,
    Document,
    Font,
    Inches,
    RGBColor,
    Paragraph,
    ParagraphCollection,
    ParagraphFormat,
    Row,
    RowCollection,
    Run,
    RunCollection,
    Table,
    TableCollection,
)


def exercise_rdocx_types(path: Path) -> None:
    document = Document(path)
    opened: Document = Document.open(path)
    loaded: Document = Document.from_bytes(b"")
    paragraph: Paragraph = document.add_paragraph("typed")
    run: Run = paragraph.add_run(" run")
    font: Font = run.font
    font.bold = True
    font.size = Inches(1)
    color = RGBColor(1, 2, 3)
    assert_type(color[0], int)
    channels: tuple[int, int, int] = color
    paragraph_format: ParagraphFormat = paragraph.paragraph_format
    paragraph_format.keep_together = None
    paragraphs: ParagraphCollection = document.paragraphs
    first: Paragraph = paragraphs[0]
    sliced: list[Paragraph] = paragraphs[:]
    for item in paragraphs:
        item.text
    table: Table = document.add_table(1, 1)
    row: Row = table.rows[0]
    cell: Cell = row.cells[0]
    cell.text = first.text
    package_bytes: bytes = loaded.to_bytes()
    pdf_bytes: bytes = opened.to_pdf()
    pages: list[bytes] = opened.render_all_pages()
    maybe_page: bytes | None = opened.render_page_to_png(0)
    document.save(path)
    document.remove_content(0)
    package_bytes, pdf_bytes, pages, maybe_page, sliced, channels


if TYPE_CHECKING:
    Cell()  # type: ignore[call-arg]
    CellCollection()  # type: ignore[call-arg]
    CellParagraphCollection()  # type: ignore[call-arg]
    Font()  # type: ignore[call-arg]
    Paragraph()  # type: ignore[call-arg]
    ParagraphCollection()  # type: ignore[call-arg]
    ParagraphFormat()  # type: ignore[call-arg]
    Row()  # type: ignore[call-arg]
    RowCollection()  # type: ignore[call-arg]
    Run()  # type: ignore[call-arg]
    RunCollection()  # type: ignore[call-arg]
    Table()  # type: ignore[call-arg]
    TableCollection()  # type: ignore[call-arg]
