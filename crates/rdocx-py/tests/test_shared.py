import io
import zipfile

import pytest


def test_alignment_center_and_inches_match_python_contract():
    from rdocx import Inches, WD_ALIGN_PARAGRAPH

    assert WD_ALIGN_PARAGRAPH.CENTER == 1
    assert Inches(1) == 914400


def test_length_is_an_int_with_unit_properties():
    from rdocx import Cm, Emu, Inches, Length, Mm, Pt, RGBColor
    from rdocx.shared import Inches as SharedInches
    from rdocx.shared import RGBColor as SharedRGBColor

    assert SharedInches is Inches
    assert SharedRGBColor is RGBColor
    assert Cm(cm=2.54) == 914400
    assert Mm(mm=25.4) == 914400

    values = (Inches(1), Cm(2.54), Mm(25.4), Pt(72), Emu(914400))
    assert all(isinstance(value, Length) for value in values)
    assert all(isinstance(value, int) for value in values)
    assert Inches(1).inches == pytest.approx(1.0)
    assert Cm(2.54).cm == pytest.approx(2.54)
    assert Mm(25.4).mm == pytest.approx(25.4)
    assert Pt(72).pt == pytest.approx(72.0)
    assert Emu(914400).emu == 914400
    assert Length(914400).twips == 1440

    color = RGBColor(0x12, 0xAB, 0x00)
    assert color == (0x12, 0xAB, 0x00)
    assert str(color) == "12AB00"
    assert RGBColor.from_string("12AB00") == color
    assert RGBColor(r=0x12, g=0xAB, b=0x00) == color
    with pytest.raises(ValueError):
        RGBColor(256, 0, 0)


def test_fractional_lengths_truncate_toward_zero():
    from rdocx import Cm, Emu, Inches, Mm, Pt

    cases = (
        (Inches, 914400.0),
        (Cm, 360000.0),
        (Mm, 36000.0),
        (Pt, 12700.0),
        (Emu, 1.0),
    )
    for constructor, factor in cases:
        assert constructor(1.75 / factor) == 1
        assert constructor(-1.75 / factor) == -1


def test_approved_enums_have_exact_values_and_docs():
    from rdocx import (
        WD_ALIGN_PARAGRAPH,
        WD_CELL_VERTICAL_ALIGNMENT,
        WD_TABLE_ALIGNMENT,
        WD_UNDERLINE,
    )
    from rdocx.enum.table import (
        WD_CELL_VERTICAL_ALIGNMENT as TableCellVerticalAlignment,
    )
    from rdocx.enum.table import WD_TABLE_ALIGNMENT as TableAlignment
    from rdocx.enum.text import WD_ALIGN_PARAGRAPH as TextParagraphAlignment
    from rdocx.enum.text import WD_UNDERLINE as TextUnderline

    assert TextParagraphAlignment is WD_ALIGN_PARAGRAPH
    assert TextUnderline is WD_UNDERLINE
    assert TableAlignment is WD_TABLE_ALIGNMENT
    assert TableCellVerticalAlignment is WD_CELL_VERTICAL_ALIGNMENT

    expected = {
        WD_ALIGN_PARAGRAPH: {
            "LEFT": 0,
            "CENTER": 1,
            "RIGHT": 2,
            "JUSTIFY": 3,
        },
        WD_TABLE_ALIGNMENT: {"LEFT": 0, "CENTER": 1, "RIGHT": 2},
        WD_CELL_VERTICAL_ALIGNMENT: {"TOP": 0, "CENTER": 1, "BOTTOM": 3},
        WD_UNDERLINE: {
            "NONE": 0,
            "SINGLE": 1,
            "WORDS": 2,
            "DOUBLE": 3,
            "DOTTED": 4,
            "THICK": 6,
            "DASH": 7,
            "DOT_DASH": 9,
            "DOT_DOT_DASH": 10,
            "WAVY": 11,
        },
    }
    for enum_type, members in expected.items():
        assert enum_type.__doc__
        assert {member.name: member.value for member in enum_type} == members


def test_exceptions_have_the_required_hierarchy():
    from rdocx import LayoutError, PackageError, RdocxError, StaleElementError, XmlError

    assert issubclass(RdocxError, Exception)
    for error_type in (PackageError, XmlError, StaleElementError, LayoutError):
        assert issubclass(error_type, RdocxError)


def test_binding_errors_raise_public_exception_classes():
    from rdocx import Document, PackageError, RdocxError, StaleElementError, XmlError

    with pytest.raises(PackageError):
        Document.from_bytes(b"not an OPC package")

    source = Document()
    source.add_paragraph("valid before corruption")
    source_bytes = io.BytesIO(source.to_bytes())
    corrupted_bytes = io.BytesIO()
    with zipfile.ZipFile(source_bytes) as source_zip:
        with zipfile.ZipFile(corrupted_bytes, "w") as corrupted_zip:
            for info in source_zip.infolist():
                data = source_zip.read(info.filename)
                if info.filename == "word/document.xml":
                    data = b"</w:nope>"
                corrupted_zip.writestr(info, data)

    with pytest.raises(XmlError):
        Document.from_bytes(corrupted_bytes.getvalue())

    live = Document()
    live.add_paragraph("zero")
    held = live.paragraphs[0]
    live.add_paragraph("one")
    with pytest.raises(StaleElementError) as raised:
        _ = held.text
    assert isinstance(raised.value, RdocxError)
