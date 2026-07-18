
#[test]
fn line_spacing_multiple_round_trips() {
    let path = std::env::temp_dir().join("rdocx-line-spacing-test.docx");
    let mut doc = rdocx::Document::new();
    {
        let p = doc.add_paragraph("double spaced");
        p.line_spacing_multiple(2.0);
    }
    doc.save(path.to_str().unwrap()).unwrap();
    let doc = rdocx::Document::open(path.to_str().unwrap()).unwrap();
    let paras = doc.paragraphs();
    let p = paras.first().unwrap();
    assert_eq!(p.line_spacing_multiple(), Some(2.0));
    let _ = std::fs::remove_file(&path);
}
