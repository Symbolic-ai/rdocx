use std::error::Error;
use std::path::Path;

use rpptx::{Presentation, ShapeRef};

pub fn records_for_path(path: &Path) -> Result<String, rpptx::Error> {
    let presentation = Presentation::open(path)?;
    Ok(records(&presentation))
}

fn records(presentation: &Presentation) -> String {
    let mut lines = Vec::new();
    for (slide_index, slide) in presentation.slides().enumerate() {
        lines.push(format!(
            "slide\t{slide_index}\t{}\t{}\t{}\t{}",
            slide.id(),
            owned_option_record(slide.name()),
            escape(&slide.text()),
            owned_option_record(slide.notes_text().as_deref()),
        ));
        for (shape_index, shape) in slide.shapes().enumerate() {
            append_shape(&mut lines, slide_index, &shape_index.to_string(), shape);
        }
    }
    lines.join("\n")
}

fn append_shape(lines: &mut Vec<String>, slide_index: usize, path: &str, shape: ShapeRef<'_>) {
    lines.push(format!(
        "shape\t{slide_index}\t{path}\t{:?}\t{}",
        shape.kind(),
        owned_option_record(shape.text().as_deref()),
    ));
    for (index, child) in shape.children().enumerate() {
        append_shape(lines, slide_index, &format!("{path}.{index}"), child);
    }
}

fn owned_option_record(value: Option<&str>) -> String {
    value.map_or_else(|| "-".to_owned(), |text| format!("+{}", escape(text)))
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

#[allow(dead_code)]
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: dump_deck <presentation.pptx>")?;
    println!("{}", records_for_path(Path::new(&path))?);
    Ok(())
}
