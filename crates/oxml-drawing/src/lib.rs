pub mod color;
pub mod effect;
pub mod fill;
pub mod geometry;
pub mod line;
pub mod namespace;
pub mod order;
pub mod shape_props;
pub mod style_ref;
pub mod table;
pub mod text;
pub mod theme;
pub mod xfrm;

#[cfg(test)]
mod table_gate_tests {
    use crate::table::CT_Table;

    #[test]
    fn merged_table_round_trips_with_merge_origins_intact() {
        let xml = br#"<a:tbl xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:tblGrid><a:gridCol w="1000"/><a:gridCol w="2000"/><a:gridCol w="3000"/></a:tblGrid><a:tr h="4000"><a:tc rowSpan="2" gridSpan="2"><a:txBody><a:bodyPr/><a:lstStyle/><a:p/></a:txBody><a:tcPr/></a:tc><a:tc hMerge="1"><a:tcPr/></a:tc><a:tc><a:tcPr/></a:tc></a:tr><a:tr h="5000"><a:tc vMerge="1"><a:tcPr/></a:tc><a:tc hMerge="1" vMerge="1"><a:tcPr/></a:tc><a:tc><a:tcPr/></a:tc></a:tr></a:tbl>"#;
        let table = CT_Table::from_xml(xml).unwrap();
        assert_eq!(
            table
                .grid
                .columns
                .iter()
                .map(|width| width.0)
                .collect::<Vec<_>>(),
            vec![1000, 2000, 3000]
        );
        assert_eq!(table.rows[0].height.0, 4000);
        assert_eq!(table.rows[1].height.0, 5000);
        let origin = &table.rows[0].cells[0];
        assert_eq!((origin.row_span, origin.grid_span), (2, 2));
        assert!(!origin.horizontal_merge && !origin.vertical_merge);
        assert!(table.rows[0].cells[1].horizontal_merge);
        assert!(table.rows[1].cells[0].vertical_merge);
        assert!(table.rows[1].cells[1].horizontal_merge);
        assert!(table.rows[1].cells[1].vertical_merge);
        let written = table.to_xml().unwrap();
        assert_eq!(table, CT_Table::from_xml(&written).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::namespace::{A_NS, A_PREFIX, PIC_NS, PIC_PREFIX};

    #[test]
    fn drawingml_namespace_uris_match_the_specification() {
        assert_eq!(
            A_NS,
            "http://schemas.openxmlformats.org/drawingml/2006/main"
        );
        assert_eq!(A_PREFIX, "a");
        assert_eq!(
            PIC_NS,
            "http://schemas.openxmlformats.org/drawingml/2006/picture"
        );
        assert_eq!(PIC_PREFIX, "pic");
    }

    #[test]
    fn oxml_drawing_is_an_unpublished_workspace_member() {
        let manifest = include_str!("../Cargo.toml");
        assert!(manifest.contains("version = \"0.0.0\""));
        assert!(manifest.contains("publish = false"));
    }
}
