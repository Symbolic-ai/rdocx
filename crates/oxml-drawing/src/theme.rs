use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{local_name, matches_local_name};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};
use std::fmt;

use crate::color::{ColorChoice, ColorError, ThemeColorSlot};
use crate::effect::{CT_EffectList, EffectError};
use crate::fill::{Fill, FillError};
use crate::line::{CT_LineProperties, LineError};
use crate::namespace::A_NS;
use crate::order::OrderedRawChildren;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use oxml_opc::OpcPackage;

    use crate::color::{ColorChoice, RgbColor};
    use crate::order::OrderedRawChildren;

    use super::{CT_OfficeStyleSheet, OFFICE_DEFAULT_XML};

    const POWERPOINT_VERSION: &str = "16.104";
    const POWERPOINT_BUILD: &str = "16.104.25121423";
    const POWERPOINT_APP_BUILD: &str = "1214";

    #[test]
    fn powerpoint_office_theme_round_trips_structurally() {
        let theme = CT_OfficeStyleSheet::from_xml(OFFICE_DEFAULT_XML.as_bytes()).unwrap();
        let reparsed = CT_OfficeStyleSheet::from_xml(&theme.to_xml().unwrap()).unwrap();

        assert_eq!(reparsed, theme);
    }

    #[test]
    #[ignore = "requires RDOCX_POWERPOINT_ORACLE_SHELL and pinned Microsoft PowerPoint"]
    fn office_default_theme_is_accepted_by_powerpoint() {
        assert_powerpoint_build();
        let shell = PathBuf::from(
            std::env::var_os("RDOCX_POWERPOINT_ORACLE_SHELL")
                .expect("RDOCX_POWERPOINT_ORACLE_SHELL must name a PowerPoint-authored PPTX"),
        );
        let output = std::env::temp_dir().join(format!(
            "rdocx-f065-office-default-{}.pptx",
            std::process::id()
        ));
        let mut package = OpcPackage::open(&shell).unwrap();
        package.set_part(
            "/ppt/theme/theme1.xml",
            CT_OfficeStyleSheet::office_default().to_xml().unwrap(),
        );
        package.save(&output).unwrap();
        open_without_repair(&output);
        fs::remove_file(output).unwrap();
    }

    #[test]
    fn theme_reads_any_prefix_and_writes_fixed_a_prefix_in_schema_order() {
        let xml = minimal_theme()
            .replace("name=\"Test\" custom", "name=\"Test &amp; Co\" custom")
            .replace("typeface=\"Major\"", "typeface=\"Major &amp; Display\"");
        let theme = CT_OfficeStyleSheet::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(theme.name.as_deref(), Some("Test & Co"));
        assert_eq!(
            theme.theme_elements.font_scheme.major_font.latin.typeface,
            "Major & Display"
        );
        let written = String::from_utf8(theme.to_xml().unwrap()).unwrap();
        let reparsed = CT_OfficeStyleSheet::from_xml(written.as_bytes()).unwrap();

        assert_eq!(reparsed, theme);
        assert!(
            written.starts_with(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><a:theme"
            )
        );
        assert!(written.contains("<a:themeElements><a:clrScheme"));
        assert!(written.find("<a:clrScheme").unwrap() < written.find("<a:fontScheme").unwrap());
        assert!(written.find("<a:fontScheme").unwrap() < written.find("<a:fmtScheme").unwrap());
        assert!(!written.contains("<z:themeElements"));
    }

    #[test]
    fn format_scheme_preserves_all_four_style_lists_in_order() {
        let theme = CT_OfficeStyleSheet::office_default();
        let matrix = &theme.theme_elements.format_scheme;

        assert_eq!(matrix.fill_styles.len(), 3);
        assert_eq!(matrix.line_styles.len(), 3);
        assert_eq!(matrix.effect_styles.len(), 3);
        assert_eq!(matrix.background_fill_styles.len(), 3);
        let written = String::from_utf8(theme.to_xml().unwrap()).unwrap();
        assert!(written.find("<a:fillStyleLst").unwrap() < written.find("<a:lnStyleLst").unwrap());
        assert!(
            written.find("<a:lnStyleLst").unwrap() < written.find("<a:effectStyleLst").unwrap()
        );
        assert!(
            written.find("<a:effectStyleLst").unwrap() < written.find("<a:bgFillStyleLst").unwrap()
        );
    }

    #[test]
    fn unknown_theme_children_round_trip_byte_for_byte_in_place() {
        let xml = minimal_theme()
            .replace(
                "<z:clrScheme",
                "<x:before id=\"1\"><x:child>one &amp; two</x:child><!--note--></x:before><z:clrScheme",
            )
            .replace("</z:clrScheme>", "</z:clrScheme><x:between x:id=\"2\"/>")
            .replace("</z:fmtScheme>", "</z:fmtScheme><x:after>kept</x:after>");
        let written = CT_OfficeStyleSheet::from_xml(xml.as_bytes())
            .unwrap()
            .to_xml()
            .unwrap();

        assert!(contains(
            &written,
            br#"<x:before id="1"><x:child>one &amp; two</x:child><!--note--></x:before>"#
        ));
        assert!(contains(&written, br#"<x:between x:id="2"/>"#));
        assert!(contains(&written, br#"<x:after>kept</x:after>"#));
        assert!(contains(&written, br#"custom="root""#));
    }

    #[test]
    fn qualified_extension_attributes_do_not_replace_unqualified_theme_attributes() {
        let xml = minimal_theme()
            .replace(
                "name=\"Test\" custom=\"root\"",
                "name=\"Test\" x:name=\"extension\" custom=\"root\"",
            )
            .replace(
                "<z:themeElements>",
                "<x:extension x:name=\"child\"/><z:themeElements>",
            );
        let theme = CT_OfficeStyleSheet::from_xml(xml.as_bytes()).unwrap();
        let written = theme.to_xml().unwrap();
        let reparsed = CT_OfficeStyleSheet::from_xml(&written).unwrap();

        assert_eq!(reparsed, theme);
        assert!(contains(&written, br#"xmlns:x="urn:test""#));
        assert!(contains(&written, br#"x:name="extension""#));
        assert!(contains(&written, br#"<x:extension x:name="child"/>"#));

        let missing_unqualified = minimal_theme().replace(
            "<z:clrScheme name=\"Test\">",
            "<z:clrScheme x:name=\"extension\">",
        );
        assert!(CT_OfficeStyleSheet::from_xml(missing_unqualified.as_bytes()).is_err());
    }

    #[test]
    fn office_default_contains_the_standard_colour_font_and_style_contract() {
        let theme = CT_OfficeStyleSheet::office_default();
        let elements = &theme.theme_elements;

        assert_eq!(elements.color_scheme.iter().count(), 12);
        assert_eq!(
            elements.font_scheme.major_font.latin.typeface,
            "Aptos Display"
        );
        assert_eq!(elements.font_scheme.minor_font.latin.typeface, "Aptos");
        assert!(elements.font_scheme.major_font.supplemental_fonts.len() > 40);
        assert!(elements.font_scheme.minor_font.supplemental_fonts.len() > 40);
        assert_eq!(elements.format_scheme.fill_styles.len(), 3);
        assert_eq!(elements.format_scheme.line_styles.len(), 3);
        assert_eq!(elements.format_scheme.effect_styles.len(), 3);
        assert_eq!(elements.format_scheme.background_fill_styles.len(), 3);
    }

    #[test]
    fn shared_theme_adapter_matches_the_legacy_theme_projection() {
        let shared = CT_OfficeStyleSheet::from_xml(OFFICE_DEFAULT_XML.as_bytes()).unwrap();
        let legacy = rdocx_oxml::theme::Theme::from_xml(OFFICE_DEFAULT_XML.as_bytes()).unwrap();
        let projected = rdocx_oxml::theme::Theme::from(&shared);

        for slot in [
            "dk1", "dk2", "lt1", "lt2", "accent1", "accent2", "accent3", "accent4", "accent5",
            "accent6", "hlink", "folHlink",
        ] {
            assert_eq!(projected.colors.get(slot), legacy.colors.get(slot));
        }
        assert_eq!(projected.major_font, legacy.major_font);
        assert_eq!(projected.minor_font, legacy.minor_font);
    }

    #[test]
    fn shared_theme_adapter_does_not_project_unresolved_colour_forms() {
        let mut shared = CT_OfficeStyleSheet::office_default();
        shared.theme_elements.color_scheme.accent1 = ColorChoice::Scheme {
            value: "accent2".to_owned(),
            transforms: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        };
        shared.theme_elements.color_scheme.accent2 = ColorChoice::Preset {
            value: "red".to_owned(),
            transforms: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        };
        shared.theme_elements.color_scheme.accent3 = ColorChoice::System {
            value: "windowText".to_owned(),
            last_color: None,
            transforms: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        };
        shared.theme_elements.color_scheme.accent4 = ColorChoice::System {
            value: "window".to_owned(),
            last_color: Some(RgbColor::new(0xab, 0xcd, 0xef)),
            transforms: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        };

        let projected = rdocx_oxml::theme::Theme::from(&shared);

        assert_eq!(projected.colors.accent1, None);
        assert_eq!(projected.colors.accent2, None);
        assert_eq!(projected.colors.accent3.as_deref(), Some("windowText"));
        assert_eq!(projected.colors.accent4.as_deref(), Some("ABCDEF"));
    }

    #[test]
    fn malformed_theme_returns_an_error_without_panicking() {
        assert!(CT_OfficeStyleSheet::from_xml(b"<a:theme/>").is_err());
        assert!(CT_OfficeStyleSheet::from_xml(b"<a:theme><a:themeElements/></a:theme>").is_err());
        assert!(CT_OfficeStyleSheet::from_xml(b"<a:theme><a:themeElements>").is_err());
        assert!(CT_OfficeStyleSheet::from_xml(b"<a:theme><a:themeElements><a:clrScheme name=\"x\"><a:dk1><a:srgbClr val=\"not-rgb\"/></a:dk1></a:clrScheme></a:themeElements></a:theme>").is_err());
        let duplicate = minimal_theme().replacen(
            "</z:clrScheme>",
            "<z:dk1><z:srgbClr val=\"000000\"/></z:dk1></z:clrScheme>",
            1,
        );
        assert!(CT_OfficeStyleSheet::from_xml(duplicate.as_bytes()).is_err());
    }

    fn minimal_theme() -> String {
        r#"<z:theme xmlns:z="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:test" name="Test" custom="root"><z:themeElements><z:clrScheme name="Test"><z:dk1><z:srgbClr val="000000"/></z:dk1><z:lt1><z:srgbClr val="FFFFFF"/></z:lt1><z:dk2><z:srgbClr val="111111"/></z:dk2><z:lt2><z:srgbClr val="EEEEEE"/></z:lt2><z:accent1><z:srgbClr val="100001"/></z:accent1><z:accent2><z:srgbClr val="200002"/></z:accent2><z:accent3><z:srgbClr val="300003"/></z:accent3><z:accent4><z:srgbClr val="400004"/></z:accent4><z:accent5><z:srgbClr val="500005"/></z:accent5><z:accent6><z:srgbClr val="600006"/></z:accent6><z:hlink><z:srgbClr val="0000FF"/></z:hlink><z:folHlink><z:srgbClr val="800080"/></z:folHlink></z:clrScheme><z:fontScheme name="Test"><z:majorFont><z:latin typeface="Major"/><z:ea typeface=""/><z:cs typeface=""/></z:majorFont><z:minorFont><z:latin typeface="Minor"/><z:ea typeface=""/><z:cs typeface=""/></z:minorFont></z:fontScheme><z:fmtScheme name="Test"><z:fillStyleLst><z:noFill/></z:fillStyleLst><z:lnStyleLst><z:ln/></z:lnStyleLst><z:effectStyleLst><z:effectStyle><z:effectLst/></z:effectStyle></z:effectStyleLst><z:bgFillStyleLst><z:noFill/></z:bgFillStyleLst></z:fmtScheme></z:themeElements></z:theme>"#.to_owned()
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|part| part == needle)
    }

    fn assert_powerpoint_build() {
        let app = "/Applications/Microsoft PowerPoint.app/Contents/Info.plist";
        let version = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", "Print :CFBundleShortVersionString", app])
            .output()
            .unwrap();
        let build = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", "Print :CFBundleVersion", app])
            .output()
            .unwrap();
        assert!(version.status.success());
        assert!(build.status.success());
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            POWERPOINT_VERSION
        );
        assert_eq!(
            String::from_utf8(build.stdout).unwrap().trim(),
            POWERPOINT_BUILD
        );
    }

    fn open_without_repair(path: &Path) {
        let path = path.to_string_lossy();
        let name = Path::new(path.as_ref())
            .file_name()
            .unwrap()
            .to_string_lossy();
        let script = format!(
            "with timeout of 120 seconds\ntell application \"Microsoft PowerPoint\"\nset previousStartUpDialog to start up dialog\ntry\nif (Version as text) is not \"{POWERPOINT_VERSION}\" then error \"PowerPoint version mismatch\"\nif (build as text) is not \"{POWERPOINT_APP_BUILD}\" then error \"PowerPoint build mismatch\"\nset start up dialog to false\nset deckPath to \"{path}\"\nopen my POSIX file deckPath\nset checkedDeck to presentation \"{name}\"\nif (full name of checkedDeck) is not deckPath then error \"exact path mismatch\"\nclose checkedDeck saving no\nset start up dialog to previousStartUpDialog\non error errorMessage number errorNumber\ntry\nclose checkedDeck saving no\nend try\nset start up dialog to previousStartUpDialog\nerror errorMessage number errorNumber\nend try\nend tell\nend timeout\n"
        );
        let result = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "PowerPoint no-repair open failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[derive(Debug)]
pub enum ThemeError {
    Xml(OxmlError),
    Color(ColorError),
    Fill(FillError),
    Line(LineError),
    Effect(EffectError),
    UnexpectedElement(String),
    MissingElement(String),
    DuplicateElement(String),
    MissingAttribute { element: String, attribute: String },
    ChildOutOfOrder { parent: String, child: String },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(e) => e.fmt(f),
            Self::Color(e) => e.fmt(f),
            Self::Fill(e) => e.fmt(f),
            Self::Line(e) => e.fmt(f),
            Self::Effect(e) => e.fmt(f),
            Self::UnexpectedElement(e) => write!(f, "unexpected DrawingML theme element: {e}"),
            Self::MissingElement(e) => write!(f, "DrawingML theme requires {e}"),
            Self::DuplicateElement(e) => write!(f, "DrawingML theme contains duplicate {e}"),
            Self::MissingAttribute { element, attribute } => {
                write!(f, "DrawingML {element} requires @{attribute}")
            }
            Self::ChildOutOfOrder { parent, child } => {
                write!(f, "DrawingML {child} is out of schema order in {parent}")
            }
        }
    }
}

impl std::error::Error for ThemeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(e) => Some(e),
            Self::Color(e) => Some(e),
            Self::Fill(e) => Some(e),
            Self::Line(e) => Some(e),
            Self::Effect(e) => Some(e),
            _ => None,
        }
    }
}

impl From<OxmlError> for ThemeError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<ColorError> for ThemeError {
    fn from(error: ColorError) -> Self {
        Self::Color(error)
    }
}

impl From<FillError> for ThemeError {
    fn from(error: FillError) -> Self {
        Self::Fill(error)
    }
}

impl From<LineError> for ThemeError {
    fn from(error: LineError) -> Self {
        Self::Line(error)
    }
}

impl From<EffectError> for ThemeError {
    fn from(error: EffectError) -> Self {
        Self::Effect(error)
    }
}

pub type Result<T> = std::result::Result<T, ThemeError>;

type RawAttributes = Vec<(String, String)>;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_OfficeStyleSheet {
    pub name: Option<String>,
    pub theme_elements: CT_ThemeElements,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_ThemeElements {
    pub color_scheme: CT_ColorScheme,
    pub font_scheme: CT_FontScheme,
    pub format_scheme: CT_StyleMatrix,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_ColorScheme {
    pub name: String,
    pub dark1: ColorChoice,
    pub light1: ColorChoice,
    pub dark2: ColorChoice,
    pub light2: ColorChoice,
    pub accent1: ColorChoice,
    pub accent2: ColorChoice,
    pub accent3: ColorChoice,
    pub accent4: ColorChoice,
    pub accent5: ColorChoice,
    pub accent6: ColorChoice,
    pub hyperlink: ColorChoice,
    pub followed_hyperlink: ColorChoice,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    slot_attributes: [RawAttributes; 12],
    slot_children: [OrderedRawChildren; 12],
}

impl CT_ColorScheme {
    pub const SLOTS: [ThemeColorSlot; 12] = [
        ThemeColorSlot::Dark1,
        ThemeColorSlot::Light1,
        ThemeColorSlot::Dark2,
        ThemeColorSlot::Light2,
        ThemeColorSlot::Accent1,
        ThemeColorSlot::Accent2,
        ThemeColorSlot::Accent3,
        ThemeColorSlot::Accent4,
        ThemeColorSlot::Accent5,
        ThemeColorSlot::Accent6,
        ThemeColorSlot::Hyperlink,
        ThemeColorSlot::FollowedHyperlink,
    ];

    pub fn color(&self, slot: ThemeColorSlot) -> &ColorChoice {
        match slot {
            ThemeColorSlot::Dark1 => &self.dark1,
            ThemeColorSlot::Light1 => &self.light1,
            ThemeColorSlot::Dark2 => &self.dark2,
            ThemeColorSlot::Light2 => &self.light2,
            ThemeColorSlot::Accent1 => &self.accent1,
            ThemeColorSlot::Accent2 => &self.accent2,
            ThemeColorSlot::Accent3 => &self.accent3,
            ThemeColorSlot::Accent4 => &self.accent4,
            ThemeColorSlot::Accent5 => &self.accent5,
            ThemeColorSlot::Accent6 => &self.accent6,
            ThemeColorSlot::Hyperlink => &self.hyperlink,
            ThemeColorSlot::FollowedHyperlink => &self.followed_hyperlink,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (ThemeColorSlot, &ColorChoice)> {
        Self::SLOTS.into_iter().map(|slot| (slot, self.color(slot)))
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_FontScheme {
    pub name: String,
    pub major_font: CT_FontCollection,
    pub minor_font: CT_FontCollection,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_FontCollection {
    pub latin: ThemeTypeface,
    pub east_asian: ThemeTypeface,
    pub complex_script: ThemeTypeface,
    pub supplemental_fonts: Vec<SupplementalFont>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemeTypeface {
    pub typeface: String,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplementalFont {
    pub script: String,
    pub typeface: String,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

impl From<&CT_OfficeStyleSheet> for rdocx_oxml::theme::Theme {
    fn from(theme: &CT_OfficeStyleSheet) -> Self {
        let colours = &theme.theme_elements.color_scheme;
        let fonts = &theme.theme_elements.font_scheme;
        Self {
            colors: rdocx_oxml::theme::ThemeColors {
                dk1: legacy_colour(colours.color(ThemeColorSlot::Dark1)),
                dk2: legacy_colour(colours.color(ThemeColorSlot::Dark2)),
                lt1: legacy_colour(colours.color(ThemeColorSlot::Light1)),
                lt2: legacy_colour(colours.color(ThemeColorSlot::Light2)),
                accent1: legacy_colour(colours.color(ThemeColorSlot::Accent1)),
                accent2: legacy_colour(colours.color(ThemeColorSlot::Accent2)),
                accent3: legacy_colour(colours.color(ThemeColorSlot::Accent3)),
                accent4: legacy_colour(colours.color(ThemeColorSlot::Accent4)),
                accent5: legacy_colour(colours.color(ThemeColorSlot::Accent5)),
                accent6: legacy_colour(colours.color(ThemeColorSlot::Accent6)),
                hlink: legacy_colour(colours.color(ThemeColorSlot::Hyperlink)),
                fol_hlink: legacy_colour(colours.color(ThemeColorSlot::FollowedHyperlink)),
            },
            major_font: Some(fonts.major_font.latin.typeface.clone()),
            minor_font: Some(fonts.minor_font.latin.typeface.clone()),
        }
    }
}

fn legacy_colour(colour: &ColorChoice) -> Option<String> {
    match colour {
        ColorChoice::Srgb { value, .. } => Some(value.to_string()),
        ColorChoice::System {
            value, last_color, ..
        } => Some(
            last_color
                .map(|resolved| resolved.to_string())
                .unwrap_or_else(|| value.clone()),
        ),
        ColorChoice::Scheme { .. } | ColorChoice::Preset { .. } => None,
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct StyleListRaw {
    attributes: RawAttributes,
    children: OrderedRawChildren,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_StyleMatrix {
    pub name: String,
    pub fill_styles: Vec<Fill>,
    pub line_styles: Vec<CT_LineProperties>,
    pub effect_styles: Vec<CT_EffectStyle>,
    pub background_fill_styles: Vec<Fill>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    fill_list_raw: StyleListRaw,
    line_list_raw: StyleListRaw,
    effect_list_raw: StyleListRaw,
    background_fill_list_raw: StyleListRaw,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_EffectStyle {
    pub effect_list: Option<CT_EffectList>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

impl CT_OfficeStyleSheet {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"theme") => {
                    return Self::read(&mut reader, &e);
                }
                Event::Empty(e) if matches_local_name(e.name().as_ref(), b"theme") => {
                    return Err(missing("themeElements"));
                }
                Event::Start(e) | Event::Empty(e) => return Err(unexpected(&e)),
                Event::Eof => return Err(missing("theme")),
                _ => {}
            }
            buf.clear();
        }
    }

    fn read(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut elements = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"themeElements") => {
                    reject_duplicate(&elements, "themeElements")?;
                    elements = Some(CT_ThemeElements::read(reader, &e)?);
                    boundary = 1;
                }
                Event::Empty(e) if matches_local_name(e.name().as_ref(), b"themeElements") => {
                    return Err(missing("themeElements children"));
                }
                Event::Start(e) => raw_children.push(boundary, capture_element(reader, &e)?),
                Event::Empty(e) => raw_children.push(boundary, capture_empty_element(&e)?),
                Event::End(e) if matches_local_name(e.name().as_ref(), b"theme") => break,
                Event::Eof => return Err(missing_end("theme")),
                _ => {}
            }
            buf.clear();
        }
        Ok(Self {
            name: decoded_attr(start, b"name")?,
            theme_elements: elements.ok_or_else(|| missing("themeElements"))?,
            raw_attributes: raw_attributes(start, &[b"name"], true)?,
            raw_children,
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        writer
            .write_event(Event::Decl(BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                Some("yes"),
            )))
            .map_err(OxmlError::from)?;
        let mut start = BytesStart::new("a:theme");
        start.push_attribute(("xmlns:a", A_NS));
        if let Some(name) = self.name.as_deref() {
            start.push_attribute(("name", name));
        }
        push_attributes(&mut start, &self.raw_attributes);
        write_start(&mut writer, start)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        self.theme_elements.write(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        write_end(&mut writer, "a:theme")?;
        Ok(writer.into_inner())
    }

    pub fn office_default() -> Self {
        Self::from_xml(OFFICE_DEFAULT_XML.as_bytes())
            .unwrap_or_else(|error| panic!("built-in Office theme is invalid: {error}"))
    }
}

impl CT_ThemeElements {
    fn read(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut colors = None;
        let mut fonts = None;
        let mut formats = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"clrScheme") => {
                    require_order("themeElements", "clrScheme", boundary, 0)?;
                    reject_duplicate(&colors, "clrScheme")?;
                    colors = Some(CT_ColorScheme::read(reader, &e)?);
                    boundary = 1;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"fontScheme") => {
                    require_order("themeElements", "fontScheme", boundary, 1)?;
                    reject_duplicate(&fonts, "fontScheme")?;
                    fonts = Some(CT_FontScheme::read(reader, &e)?);
                    boundary = 2;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"fmtScheme") => {
                    require_order("themeElements", "fmtScheme", boundary, 2)?;
                    reject_duplicate(&formats, "fmtScheme")?;
                    formats = Some(CT_StyleMatrix::read(reader, &e)?);
                    boundary = 3;
                }
                Event::Empty(e) if is_theme_elements_child(e.name().as_ref()) => {
                    return Err(missing(format!("{} children", element_name(&e))));
                }
                Event::Start(e) => raw_children.push(boundary, capture_element(reader, &e)?),
                Event::Empty(e) => raw_children.push(boundary, capture_empty_element(&e)?),
                Event::End(e) if matches_local_name(e.name().as_ref(), b"themeElements") => break,
                Event::Eof => return Err(missing_end("themeElements")),
                _ => {}
            }
            buf.clear();
        }
        Ok(Self {
            color_scheme: colors.ok_or_else(|| missing("clrScheme"))?,
            font_scheme: fonts.ok_or_else(|| missing("fontScheme"))?,
            format_scheme: formats.ok_or_else(|| missing("fmtScheme"))?,
            raw_attributes: raw_attributes(start, &[], false)?,
            raw_children,
        })
    }

    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<()> {
        let mut start = BytesStart::new("a:themeElements");
        push_attributes(&mut start, &self.raw_attributes);
        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        self.color_scheme.write(writer)?;
        emit_raw(writer, self.raw_children.at(1))?;
        self.font_scheme.write(writer)?;
        emit_raw(writer, self.raw_children.at(2))?;
        self.format_scheme.write(writer)?;
        emit_raw(writer, self.raw_children.at(3))?;
        write_end(writer, "a:themeElements")
    }
}

#[derive(Default)]
struct ParsedColors {
    values: [Option<ColorChoice>; 12],
    attributes: [RawAttributes; 12],
    children: [OrderedRawChildren; 12],
}

struct ParsedColorSlot {
    value: ColorChoice,
    attributes: RawAttributes,
    children: OrderedRawChildren,
}

impl CT_ColorScheme {
    fn read(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut parsed = ParsedColors::default();
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if color_slot_index(e.name().as_ref()).is_some() => {
                    let index = color_slot_index(e.name().as_ref()).unwrap();
                    let name = element_name(&e);
                    require_order("clrScheme", &name, boundary, index)?;
                    if parsed.values[index].is_some() {
                        return Err(duplicate(name));
                    }
                    let slot = read_color_slot(reader, &e)?;
                    parsed.values[index] = Some(slot.value);
                    parsed.attributes[index] = slot.attributes;
                    parsed.children[index] = slot.children;
                    boundary = index + 1;
                }
                Event::Empty(e) if color_slot_index(e.name().as_ref()).is_some() => {
                    return Err(missing(format!("{} colour", element_name(&e))));
                }
                Event::Start(e) => raw_children.push(boundary, capture_element(reader, &e)?),
                Event::Empty(e) => raw_children.push(boundary, capture_empty_element(&e)?),
                Event::End(e) if matches_local_name(e.name().as_ref(), b"clrScheme") => break,
                Event::Eof => return Err(missing_end("clrScheme")),
                _ => {}
            }
            buf.clear();
        }
        let [
            dark1,
            light1,
            dark2,
            light2,
            accent1,
            accent2,
            accent3,
            accent4,
            accent5,
            accent6,
            hyperlink,
            followed_hyperlink,
        ] = parsed.values;
        Ok(Self {
            name: required_attr(start, b"name")?,
            dark1: dark1.ok_or_else(|| missing("dk1"))?,
            light1: light1.ok_or_else(|| missing("lt1"))?,
            dark2: dark2.ok_or_else(|| missing("dk2"))?,
            light2: light2.ok_or_else(|| missing("lt2"))?,
            accent1: accent1.ok_or_else(|| missing("accent1"))?,
            accent2: accent2.ok_or_else(|| missing("accent2"))?,
            accent3: accent3.ok_or_else(|| missing("accent3"))?,
            accent4: accent4.ok_or_else(|| missing("accent4"))?,
            accent5: accent5.ok_or_else(|| missing("accent5"))?,
            accent6: accent6.ok_or_else(|| missing("accent6"))?,
            hyperlink: hyperlink.ok_or_else(|| missing("hlink"))?,
            followed_hyperlink: followed_hyperlink.ok_or_else(|| missing("folHlink"))?,
            raw_attributes: raw_attributes(start, &[b"name"], false)?,
            raw_children,
            slot_attributes: parsed.attributes,
            slot_children: parsed.children,
        })
    }

    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<()> {
        let mut start = BytesStart::new("a:clrScheme");
        start.push_attribute(("name", self.name.as_str()));
        push_attributes(&mut start, &self.raw_attributes);
        write_start(writer, start)?;
        for (index, (slot, color)) in self.iter().enumerate() {
            emit_raw(writer, self.raw_children.at(index))?;
            let tag = format!("a:{}", slot.as_str());
            let mut slot_start = BytesStart::new(tag.as_str());
            push_attributes(&mut slot_start, &self.slot_attributes[index]);
            write_start(writer, slot_start)?;
            emit_raw(writer, self.slot_children[index].at(0))?;
            color.to_xml(writer)?;
            emit_raw(writer, self.slot_children[index].at(1))?;
            write_end(writer, &tag)?;
        }
        emit_raw(writer, self.raw_children.at(12))?;
        write_end(writer, "a:clrScheme")
    }
}

fn read_color_slot(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<ParsedColorSlot> {
    let end = local_name(start.name().as_ref()).to_vec();
    let mut value = None;
    let mut children = OrderedRawChildren::default();
    let mut boundary = 0;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
            Event::Start(e) if is_color(e.name().as_ref()) => {
                reject_duplicate(&value, "theme colour choice")?;
                value = Some(ColorChoice::from_xml(reader, &e)?);
                boundary = 1;
            }
            Event::Empty(e) if is_color(e.name().as_ref()) => {
                reject_duplicate(&value, "theme colour choice")?;
                value = Some(ColorChoice::from_empty_xml(&e)?);
                boundary = 1;
            }
            Event::Start(e) => children.push(boundary, capture_element(reader, &e)?),
            Event::Empty(e) => children.push(boundary, capture_empty_element(&e)?),
            Event::End(e) if local_name(e.name().as_ref()) == end => break,
            Event::Eof => return Err(missing_end(String::from_utf8_lossy(&end))),
            _ => {}
        }
        buf.clear();
    }
    Ok(ParsedColorSlot {
        value: value.ok_or_else(|| missing("theme colour choice"))?,
        attributes: raw_attributes(start, &[], false)?,
        children,
    })
}

impl CT_FontScheme {
    fn read(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut major = None;
        let mut minor = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"majorFont") => {
                    require_order("fontScheme", "majorFont", boundary, 0)?;
                    reject_duplicate(&major, "majorFont")?;
                    major = Some(CT_FontCollection::read(reader, &e)?);
                    boundary = 1;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"minorFont") => {
                    require_order("fontScheme", "minorFont", boundary, 1)?;
                    reject_duplicate(&minor, "minorFont")?;
                    minor = Some(CT_FontCollection::read(reader, &e)?);
                    boundary = 2;
                }
                Event::Empty(e)
                    if matches_local_name(e.name().as_ref(), b"majorFont")
                        || matches_local_name(e.name().as_ref(), b"minorFont") =>
                {
                    return Err(missing(format!("{} children", element_name(&e))));
                }
                Event::Start(e) => raw_children.push(boundary, capture_element(reader, &e)?),
                Event::Empty(e) => raw_children.push(boundary, capture_empty_element(&e)?),
                Event::End(e) if matches_local_name(e.name().as_ref(), b"fontScheme") => break,
                Event::Eof => return Err(missing_end("fontScheme")),
                _ => {}
            }
            buf.clear();
        }
        Ok(Self {
            name: required_attr(start, b"name")?,
            major_font: major.ok_or_else(|| missing("majorFont"))?,
            minor_font: minor.ok_or_else(|| missing("minorFont"))?,
            raw_attributes: raw_attributes(start, &[b"name"], false)?,
            raw_children,
        })
    }

    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<()> {
        let mut start = BytesStart::new("a:fontScheme");
        start.push_attribute(("name", self.name.as_str()));
        push_attributes(&mut start, &self.raw_attributes);
        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        self.major_font.write(writer, "a:majorFont")?;
        emit_raw(writer, self.raw_children.at(1))?;
        self.minor_font.write(writer, "a:minorFont")?;
        emit_raw(writer, self.raw_children.at(2))?;
        write_end(writer, "a:fontScheme")
    }
}

impl CT_FontCollection {
    fn read(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let end = local_name(start.name().as_ref()).to_vec();
        let mut latin = None;
        let mut east_asian = None;
        let mut complex_script = None;
        let mut supplemental_fonts = Vec::new();
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"latin") => {
                    require_order("font collection", "latin", boundary, 0)?;
                    reject_duplicate(&latin, "latin")?;
                    latin = Some(read_typeface(reader, &e, false)?);
                    boundary = 1;
                }
                Event::Empty(e) if matches_local_name(e.name().as_ref(), b"latin") => {
                    require_order("font collection", "latin", boundary, 0)?;
                    reject_duplicate(&latin, "latin")?;
                    latin = Some(read_typeface(reader, &e, true)?);
                    boundary = 1;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"ea") => {
                    require_order("font collection", "ea", boundary, 1)?;
                    reject_duplicate(&east_asian, "ea")?;
                    east_asian = Some(read_typeface(reader, &e, false)?);
                    boundary = 2;
                }
                Event::Empty(e) if matches_local_name(e.name().as_ref(), b"ea") => {
                    require_order("font collection", "ea", boundary, 1)?;
                    reject_duplicate(&east_asian, "ea")?;
                    east_asian = Some(read_typeface(reader, &e, true)?);
                    boundary = 2;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"cs") => {
                    require_order("font collection", "cs", boundary, 2)?;
                    reject_duplicate(&complex_script, "cs")?;
                    complex_script = Some(read_typeface(reader, &e, false)?);
                    boundary = 3;
                }
                Event::Empty(e) if matches_local_name(e.name().as_ref(), b"cs") => {
                    require_order("font collection", "cs", boundary, 2)?;
                    reject_duplicate(&complex_script, "cs")?;
                    complex_script = Some(read_typeface(reader, &e, true)?);
                    boundary = 3;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"font") => {
                    if boundary < 3 {
                        return Err(out_of_order("font collection", "font"));
                    }
                    supplemental_fonts.push(read_supplemental_font(reader, &e, false)?);
                    boundary = 3 + supplemental_fonts.len();
                }
                Event::Empty(e) if matches_local_name(e.name().as_ref(), b"font") => {
                    if boundary < 3 {
                        return Err(out_of_order("font collection", "font"));
                    }
                    supplemental_fonts.push(read_supplemental_font(reader, &e, true)?);
                    boundary = 3 + supplemental_fonts.len();
                }
                Event::Start(e) => raw_children.push(boundary, capture_element(reader, &e)?),
                Event::Empty(e) => raw_children.push(boundary, capture_empty_element(&e)?),
                Event::End(e) if local_name(e.name().as_ref()) == end => break,
                Event::Eof => return Err(missing_end(String::from_utf8_lossy(&end))),
                _ => {}
            }
            buf.clear();
        }
        Ok(Self {
            latin: latin.ok_or_else(|| missing("latin"))?,
            east_asian: east_asian.ok_or_else(|| missing("ea"))?,
            complex_script: complex_script.ok_or_else(|| missing("cs"))?,
            supplemental_fonts,
            raw_attributes: raw_attributes(start, &[], false)?,
            raw_children,
        })
    }

    fn write(&self, writer: &mut Writer<Vec<u8>>, tag: &str) -> Result<()> {
        let mut start = BytesStart::new(tag);
        push_attributes(&mut start, &self.raw_attributes);
        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        self.latin.write(writer, "a:latin")?;
        emit_raw(writer, self.raw_children.at(1))?;
        self.east_asian.write(writer, "a:ea")?;
        emit_raw(writer, self.raw_children.at(2))?;
        self.complex_script.write(writer, "a:cs")?;
        for (index, font) in self.supplemental_fonts.iter().enumerate() {
            emit_raw(writer, self.raw_children.at(3 + index))?;
            font.write(writer)?;
        }
        emit_raw(
            writer,
            self.raw_children.at(3 + self.supplemental_fonts.len()),
        )?;
        write_end(writer, tag)
    }
}

fn read_typeface(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    empty: bool,
) -> Result<ThemeTypeface> {
    Ok(ThemeTypeface {
        typeface: required_attr(start, b"typeface")?,
        raw_attributes: raw_attributes(start, &[b"typeface"], false)?,
        raw_children: read_leaf_children(reader, start, empty)?,
    })
}

impl ThemeTypeface {
    fn write(&self, writer: &mut Writer<Vec<u8>>, tag: &str) -> Result<()> {
        let mut start = BytesStart::new(tag);
        start.push_attribute(("typeface", self.typeface.as_str()));
        push_attributes(&mut start, &self.raw_attributes);
        write_leaf(writer, start, tag, &self.raw_children)
    }
}

fn read_supplemental_font(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    empty: bool,
) -> Result<SupplementalFont> {
    Ok(SupplementalFont {
        script: required_attr(start, b"script")?,
        typeface: required_attr(start, b"typeface")?,
        raw_attributes: raw_attributes(start, &[b"script", b"typeface"], false)?,
        raw_children: read_leaf_children(reader, start, empty)?,
    })
}

impl SupplementalFont {
    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<()> {
        let mut start = BytesStart::new("a:font");
        start.push_attribute(("script", self.script.as_str()));
        start.push_attribute(("typeface", self.typeface.as_str()));
        push_attributes(&mut start, &self.raw_attributes);
        write_leaf(writer, start, "a:font", &self.raw_children)
    }
}

impl CT_StyleMatrix {
    fn read(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut fills = None;
        let mut lines = None;
        let mut effects = None;
        let mut background_fills = None;
        let mut fill_list_raw = StyleListRaw::default();
        let mut line_list_raw = StyleListRaw::default();
        let mut effect_list_raw = StyleListRaw::default();
        let mut background_fill_list_raw = StyleListRaw::default();
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"fillStyleLst") => {
                    require_order("fmtScheme", "fillStyleLst", boundary, 0)?;
                    reject_duplicate(&fills, "fillStyleLst")?;
                    let (values, raw) = read_fill_list(reader, &e, b"fillStyleLst")?;
                    fills = Some(values);
                    fill_list_raw = raw;
                    boundary = 1;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"lnStyleLst") => {
                    require_order("fmtScheme", "lnStyleLst", boundary, 1)?;
                    reject_duplicate(&lines, "lnStyleLst")?;
                    let (values, raw) = read_line_list(reader, &e)?;
                    lines = Some(values);
                    line_list_raw = raw;
                    boundary = 2;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"effectStyleLst") => {
                    require_order("fmtScheme", "effectStyleLst", boundary, 2)?;
                    reject_duplicate(&effects, "effectStyleLst")?;
                    let (values, raw) = read_effect_style_list(reader, &e)?;
                    effects = Some(values);
                    effect_list_raw = raw;
                    boundary = 3;
                }
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"bgFillStyleLst") => {
                    require_order("fmtScheme", "bgFillStyleLst", boundary, 3)?;
                    reject_duplicate(&background_fills, "bgFillStyleLst")?;
                    let (values, raw) = read_fill_list(reader, &e, b"bgFillStyleLst")?;
                    background_fills = Some(values);
                    background_fill_list_raw = raw;
                    boundary = 4;
                }
                Event::Empty(e) if is_style_list(e.name().as_ref()) => {
                    return Err(missing(format!("{} entries", element_name(&e))));
                }
                Event::Start(e) => raw_children.push(boundary, capture_element(reader, &e)?),
                Event::Empty(e) => raw_children.push(boundary, capture_empty_element(&e)?),
                Event::End(e) if matches_local_name(e.name().as_ref(), b"fmtScheme") => break,
                Event::Eof => return Err(missing_end("fmtScheme")),
                _ => {}
            }
            buf.clear();
        }
        Ok(Self {
            name: required_attr(start, b"name")?,
            fill_styles: fills.ok_or_else(|| missing("fillStyleLst"))?,
            line_styles: lines.ok_or_else(|| missing("lnStyleLst"))?,
            effect_styles: effects.ok_or_else(|| missing("effectStyleLst"))?,
            background_fill_styles: background_fills.ok_or_else(|| missing("bgFillStyleLst"))?,
            raw_attributes: raw_attributes(start, &[b"name"], false)?,
            raw_children,
            fill_list_raw,
            line_list_raw,
            effect_list_raw,
            background_fill_list_raw,
        })
    }

    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<()> {
        let mut start = BytesStart::new("a:fmtScheme");
        start.push_attribute(("name", self.name.as_str()));
        push_attributes(&mut start, &self.raw_attributes);
        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        write_fill_list(
            writer,
            "a:fillStyleLst",
            &self.fill_styles,
            &self.fill_list_raw,
        )?;
        emit_raw(writer, self.raw_children.at(1))?;
        write_line_list(writer, &self.line_styles, &self.line_list_raw)?;
        emit_raw(writer, self.raw_children.at(2))?;
        write_effect_style_list(writer, &self.effect_styles, &self.effect_list_raw)?;
        emit_raw(writer, self.raw_children.at(3))?;
        write_fill_list(
            writer,
            "a:bgFillStyleLst",
            &self.background_fill_styles,
            &self.background_fill_list_raw,
        )?;
        emit_raw(writer, self.raw_children.at(4))?;
        write_end(writer, "a:fmtScheme")
    }
}

fn read_fill_list(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    end: &[u8],
) -> Result<(Vec<Fill>, StyleListRaw)> {
    let mut values = Vec::new();
    let mut raw = StyleListRaw {
        attributes: raw_attributes(start, &[], false)?,
        ..StyleListRaw::default()
    };
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
            Event::Start(e) if is_fill(e.name().as_ref()) => {
                let xml = capture_element(reader, &e)?;
                values.push(Fill::from_xml(&xml)?);
            }
            Event::Empty(e) if is_fill(e.name().as_ref()) => {
                let xml = capture_empty_element(&e)?;
                values.push(Fill::from_xml(&xml)?);
            }
            Event::Start(e) => raw
                .children
                .push(values.len(), capture_element(reader, &e)?),
            Event::Empty(e) => raw.children.push(values.len(), capture_empty_element(&e)?),
            Event::End(e) if local_name(e.name().as_ref()) == end => break,
            Event::Eof => return Err(missing_end(String::from_utf8_lossy(end))),
            _ => {}
        }
        buf.clear();
    }
    Ok((values, raw))
}

fn read_line_list(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
) -> Result<(Vec<CT_LineProperties>, StyleListRaw)> {
    let mut values = Vec::new();
    let mut raw = StyleListRaw {
        attributes: raw_attributes(start, &[], false)?,
        ..StyleListRaw::default()
    };
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
            Event::Start(e) if matches_local_name(e.name().as_ref(), b"ln") => {
                let xml = capture_element(reader, &e)?;
                values.push(CT_LineProperties::from_xml(&xml)?);
            }
            Event::Empty(e) if matches_local_name(e.name().as_ref(), b"ln") => {
                let xml = capture_empty_element(&e)?;
                values.push(CT_LineProperties::from_xml(&xml)?);
            }
            Event::Start(e) => raw
                .children
                .push(values.len(), capture_element(reader, &e)?),
            Event::Empty(e) => raw.children.push(values.len(), capture_empty_element(&e)?),
            Event::End(e) if matches_local_name(e.name().as_ref(), b"lnStyleLst") => break,
            Event::Eof => return Err(missing_end("lnStyleLst")),
            _ => {}
        }
        buf.clear();
    }
    Ok((values, raw))
}

fn read_effect_style_list(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
) -> Result<(Vec<CT_EffectStyle>, StyleListRaw)> {
    let mut values = Vec::new();
    let mut raw = StyleListRaw {
        attributes: raw_attributes(start, &[], false)?,
        ..StyleListRaw::default()
    };
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
            Event::Start(e) if matches_local_name(e.name().as_ref(), b"effectStyle") => {
                values.push(CT_EffectStyle::read(reader, &e)?);
            }
            Event::Empty(e) if matches_local_name(e.name().as_ref(), b"effectStyle") => {
                values.push(CT_EffectStyle {
                    effect_list: None,
                    raw_attributes: raw_attributes(&e, &[], false)?,
                    raw_children: OrderedRawChildren::default(),
                });
            }
            Event::Start(e) => raw
                .children
                .push(values.len(), capture_element(reader, &e)?),
            Event::Empty(e) => raw.children.push(values.len(), capture_empty_element(&e)?),
            Event::End(e) if matches_local_name(e.name().as_ref(), b"effectStyleLst") => break,
            Event::Eof => return Err(missing_end("effectStyleLst")),
            _ => {}
        }
        buf.clear();
    }
    Ok((values, raw))
}

impl CT_EffectStyle {
    fn read(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut effect_list = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
                Event::Start(e) if matches_local_name(e.name().as_ref(), b"effectLst") => {
                    reject_duplicate(&effect_list, "effectLst")?;
                    effect_list = Some(CT_EffectList::from_xml(&capture_element(reader, &e)?)?);
                    boundary = 1;
                }
                Event::Empty(e) if matches_local_name(e.name().as_ref(), b"effectLst") => {
                    reject_duplicate(&effect_list, "effectLst")?;
                    effect_list = Some(CT_EffectList::from_xml(&capture_empty_element(&e)?)?);
                    boundary = 1;
                }
                Event::Start(e) => raw_children.push(boundary, capture_element(reader, &e)?),
                Event::Empty(e) => raw_children.push(boundary, capture_empty_element(&e)?),
                Event::End(e) if matches_local_name(e.name().as_ref(), b"effectStyle") => break,
                Event::Eof => return Err(missing_end("effectStyle")),
                _ => {}
            }
            buf.clear();
        }
        Ok(Self {
            effect_list,
            raw_attributes: raw_attributes(start, &[], false)?,
            raw_children,
        })
    }

    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<()> {
        let mut start = BytesStart::new("a:effectStyle");
        push_attributes(&mut start, &self.raw_attributes);
        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(effects) = &self.effect_list {
            effects.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        write_end(writer, "a:effectStyle")
    }
}

fn write_fill_list(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    values: &[Fill],
    raw: &StyleListRaw,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, &raw.attributes);
    write_start(writer, start)?;
    for (index, value) in values.iter().enumerate() {
        emit_raw(writer, raw.children.at(index))?;
        value.write_xml(writer)?;
    }
    emit_raw(writer, raw.children.at(values.len()))?;
    write_end(writer, tag)
}

fn write_line_list(
    writer: &mut Writer<Vec<u8>>,
    values: &[CT_LineProperties],
    raw: &StyleListRaw,
) -> Result<()> {
    let mut start = BytesStart::new("a:lnStyleLst");
    push_attributes(&mut start, &raw.attributes);
    write_start(writer, start)?;
    for (index, value) in values.iter().enumerate() {
        emit_raw(writer, raw.children.at(index))?;
        value.write_xml(writer)?;
    }
    emit_raw(writer, raw.children.at(values.len()))?;
    write_end(writer, "a:lnStyleLst")
}

fn write_effect_style_list(
    writer: &mut Writer<Vec<u8>>,
    values: &[CT_EffectStyle],
    raw: &StyleListRaw,
) -> Result<()> {
    let mut start = BytesStart::new("a:effectStyleLst");
    push_attributes(&mut start, &raw.attributes);
    write_start(writer, start)?;
    for (index, value) in values.iter().enumerate() {
        emit_raw(writer, raw.children.at(index))?;
        value.write(writer)?;
    }
    emit_raw(writer, raw.children.at(values.len()))?;
    write_end(writer, "a:effectStyleLst")
}

fn read_leaf_children(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    empty: bool,
) -> Result<OrderedRawChildren> {
    if empty {
        return Ok(OrderedRawChildren::default());
    }
    let end = local_name(start.name().as_ref()).to_vec();
    let mut raw = OrderedRawChildren::default();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(OxmlError::from)? {
            Event::Start(e) => raw.push(0, capture_element(reader, &e)?),
            Event::Empty(e) => raw.push(0, capture_empty_element(&e)?),
            Event::End(e) if local_name(e.name().as_ref()) == end => break,
            Event::Eof => return Err(missing_end(String::from_utf8_lossy(&end))),
            _ => {}
        }
        buf.clear();
    }
    Ok(raw)
}

fn write_leaf(
    writer: &mut Writer<Vec<u8>>,
    start: BytesStart<'_>,
    tag: &str,
    raw: &OrderedRawChildren,
) -> Result<()> {
    if raw.is_empty() {
        writer
            .write_event(Event::Empty(start))
            .map_err(OxmlError::from)?;
        return Ok(());
    }
    write_start(writer, start)?;
    emit_raw(writer, raw.at(0))?;
    write_end(writer, tag)
}

fn raw_attributes(
    start: &BytesStart<'_>,
    modelled: &[&[u8]],
    is_root: bool,
) -> Result<RawAttributes> {
    let mut raw = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let key = attribute.key.as_ref();
        if modelled.contains(&key) {
            continue;
        }
        if is_root && key == b"xmlns:a" {
            continue;
        }
        let name = std::str::from_utf8(key)
            .map_err(OxmlError::from)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        raw.push((name, value));
    }
    Ok(raw)
}

fn required_attr(start: &BytesStart<'_>, name: &[u8]) -> Result<String> {
    decoded_attr(start, name)?.ok_or_else(|| ThemeError::MissingAttribute {
        element: element_name(start),
        attribute: String::from_utf8_lossy(name).into_owned(),
    })
}

fn decoded_attr(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<String>> {
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        if attribute.key.as_ref() == name {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
                    .map_err(OxmlError::from)?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &RawAttributes) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn reject_duplicate<T>(value: &Option<T>, name: impl Into<String>) -> Result<()> {
    if value.is_some() {
        Err(duplicate(name))
    } else {
        Ok(())
    }
}

fn require_order(parent: &str, child: &str, boundary: usize, index: usize) -> Result<()> {
    if boundary > index {
        Err(out_of_order(parent, child))
    } else {
        Ok(())
    }
}

fn color_slot_index(name: &[u8]) -> Option<usize> {
    match local_name(name) {
        b"dk1" => Some(0),
        b"lt1" => Some(1),
        b"dk2" => Some(2),
        b"lt2" => Some(3),
        b"accent1" => Some(4),
        b"accent2" => Some(5),
        b"accent3" => Some(6),
        b"accent4" => Some(7),
        b"accent5" => Some(8),
        b"accent6" => Some(9),
        b"hlink" => Some(10),
        b"folHlink" => Some(11),
        _ => None,
    }
}

fn is_color(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"srgbClr" | b"schemeClr" | b"sysClr" | b"prstClr"
    )
}

fn is_fill(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"noFill" | b"solidFill" | b"gradFill" | b"pattFill" | b"blipFill"
    )
}

fn is_theme_elements_child(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"clrScheme" | b"fontScheme" | b"fmtScheme"
    )
}

fn is_style_list(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"fillStyleLst" | b"lnStyleLst" | b"effectStyleLst" | b"bgFillStyleLst"
    )
}

fn write_start(writer: &mut Writer<Vec<u8>>, start: BytesStart<'_>) -> Result<()> {
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_end(writer: &mut Writer<Vec<u8>>, name: &str) -> Result<()> {
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn emit_raw<'a>(
    writer: &mut Writer<Vec<u8>>,
    values: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    for value in values {
        writer.get_mut().extend_from_slice(value);
    }
    Ok(())
}

fn element_name(start: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(local_name(start.name().as_ref())).into_owned()
}

fn unexpected(start: &BytesStart<'_>) -> ThemeError {
    ThemeError::UnexpectedElement(String::from_utf8_lossy(start.name().as_ref()).into_owned())
}

fn missing(name: impl Into<String>) -> ThemeError {
    ThemeError::MissingElement(name.into())
}

fn missing_end(name: impl fmt::Display) -> ThemeError {
    ThemeError::Xml(OxmlError::MissingElement(format!(
        "closing DrawingML {name}"
    )))
}

fn duplicate(name: impl Into<String>) -> ThemeError {
    ThemeError::DuplicateElement(name.into())
}

fn out_of_order(parent: impl Into<String>, child: impl Into<String>) -> ThemeError {
    ThemeError::ChildOutOfOrder {
        parent: parent.into(),
        child: child.into(),
    }
}

const OFFICE_DEFAULT_XML: &str = r###"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme"><a:themeElements><a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="0E2841"/></a:dk2><a:lt2><a:srgbClr val="E8E8E8"/></a:lt2><a:accent1><a:srgbClr val="156082"/></a:accent1><a:accent2><a:srgbClr val="E97132"/></a:accent2><a:accent3><a:srgbClr val="196B24"/></a:accent3><a:accent4><a:srgbClr val="0F9ED5"/></a:accent4><a:accent5><a:srgbClr val="A02B93"/></a:accent5><a:accent6><a:srgbClr val="4EA72E"/></a:accent6><a:hlink><a:srgbClr val="467886"/></a:hlink><a:folHlink><a:srgbClr val="96607D"/></a:folHlink></a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Aptos Display" panose="02110004020202020204"/><a:ea typeface=""/><a:cs typeface=""/><a:font script="Jpan" typeface="游ゴシック Light"/><a:font script="Hang" typeface="맑은 고딕"/><a:font script="Hans" typeface="等线 Light"/><a:font script="Hant" typeface="新細明體"/><a:font script="Arab" typeface="Times New Roman"/><a:font script="Hebr" typeface="Times New Roman"/><a:font script="Thai" typeface="Angsana New"/><a:font script="Ethi" typeface="Nyala"/><a:font script="Beng" typeface="Vrinda"/><a:font script="Gujr" typeface="Shruti"/><a:font script="Khmr" typeface="MoolBoran"/><a:font script="Knda" typeface="Tunga"/><a:font script="Guru" typeface="Raavi"/><a:font script="Cans" typeface="Euphemia"/><a:font script="Cher" typeface="Plantagenet Cherokee"/><a:font script="Yiii" typeface="Microsoft Yi Baiti"/><a:font script="Tibt" typeface="Microsoft Himalaya"/><a:font script="Thaa" typeface="MV Boli"/><a:font script="Deva" typeface="Mangal"/><a:font script="Telu" typeface="Gautami"/><a:font script="Taml" typeface="Latha"/><a:font script="Syrc" typeface="Estrangelo Edessa"/><a:font script="Orya" typeface="Kalinga"/><a:font script="Mlym" typeface="Kartika"/><a:font script="Laoo" typeface="DokChampa"/><a:font script="Sinh" typeface="Iskoola Pota"/><a:font script="Mong" typeface="Mongolian Baiti"/><a:font script="Viet" typeface="Times New Roman"/><a:font script="Uigh" typeface="Microsoft Uighur"/><a:font script="Geor" typeface="Sylfaen"/><a:font script="Armn" typeface="Arial"/><a:font script="Bugi" typeface="Leelawadee UI"/><a:font script="Bopo" typeface="Microsoft JhengHei"/><a:font script="Java" typeface="Javanese Text"/><a:font script="Lisu" typeface="Segoe UI"/><a:font script="Mymr" typeface="Myanmar Text"/><a:font script="Nkoo" typeface="Ebrima"/><a:font script="Olck" typeface="Nirmala UI"/><a:font script="Osma" typeface="Ebrima"/><a:font script="Phag" typeface="Phagspa"/><a:font script="Syrn" typeface="Estrangelo Edessa"/><a:font script="Syrj" typeface="Estrangelo Edessa"/><a:font script="Syre" typeface="Estrangelo Edessa"/><a:font script="Sora" typeface="Nirmala UI"/><a:font script="Tale" typeface="Microsoft Tai Le"/><a:font script="Talu" typeface="Microsoft New Tai Lue"/><a:font script="Tfng" typeface="Ebrima"/></a:majorFont><a:minorFont><a:latin typeface="Aptos" panose="02110004020202020204"/><a:ea typeface=""/><a:cs typeface=""/><a:font script="Jpan" typeface="游ゴシック"/><a:font script="Hang" typeface="맑은 고딕"/><a:font script="Hans" typeface="等线"/><a:font script="Hant" typeface="新細明體"/><a:font script="Arab" typeface="Arial"/><a:font script="Hebr" typeface="Arial"/><a:font script="Thai" typeface="Cordia New"/><a:font script="Ethi" typeface="Nyala"/><a:font script="Beng" typeface="Vrinda"/><a:font script="Gujr" typeface="Shruti"/><a:font script="Khmr" typeface="DaunPenh"/><a:font script="Knda" typeface="Tunga"/><a:font script="Guru" typeface="Raavi"/><a:font script="Cans" typeface="Euphemia"/><a:font script="Cher" typeface="Plantagenet Cherokee"/><a:font script="Yiii" typeface="Microsoft Yi Baiti"/><a:font script="Tibt" typeface="Microsoft Himalaya"/><a:font script="Thaa" typeface="MV Boli"/><a:font script="Deva" typeface="Mangal"/><a:font script="Telu" typeface="Gautami"/><a:font script="Taml" typeface="Latha"/><a:font script="Syrc" typeface="Estrangelo Edessa"/><a:font script="Orya" typeface="Kalinga"/><a:font script="Mlym" typeface="Kartika"/><a:font script="Laoo" typeface="DokChampa"/><a:font script="Sinh" typeface="Iskoola Pota"/><a:font script="Mong" typeface="Mongolian Baiti"/><a:font script="Viet" typeface="Arial"/><a:font script="Uigh" typeface="Microsoft Uighur"/><a:font script="Geor" typeface="Sylfaen"/><a:font script="Armn" typeface="Arial"/><a:font script="Bugi" typeface="Leelawadee UI"/><a:font script="Bopo" typeface="Microsoft JhengHei"/><a:font script="Java" typeface="Javanese Text"/><a:font script="Lisu" typeface="Segoe UI"/><a:font script="Mymr" typeface="Myanmar Text"/><a:font script="Nkoo" typeface="Ebrima"/><a:font script="Olck" typeface="Nirmala UI"/><a:font script="Osma" typeface="Ebrima"/><a:font script="Phag" typeface="Phagspa"/><a:font script="Syrn" typeface="Estrangelo Edessa"/><a:font script="Syrj" typeface="Estrangelo Edessa"/><a:font script="Syre" typeface="Estrangelo Edessa"/><a:font script="Sora" typeface="Nirmala UI"/><a:font script="Tale" typeface="Microsoft Tai Le"/><a:font script="Talu" typeface="Microsoft New Tai Lue"/><a:font script="Tfng" typeface="Ebrima"/></a:minorFont></a:fontScheme><a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:lumMod val="110000"/><a:satMod val="105000"/><a:tint val="67000"/></a:schemeClr></a:gs><a:gs pos="50000"><a:schemeClr val="phClr"><a:lumMod val="105000"/><a:satMod val="103000"/><a:tint val="73000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:lumMod val="105000"/><a:satMod val="109000"/><a:tint val="81000"/></a:schemeClr></a:gs></a:gsLst><a:lin ang="5400000" scaled="0"/></a:gradFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:satMod val="103000"/><a:lumMod val="102000"/><a:tint val="94000"/></a:schemeClr></a:gs><a:gs pos="50000"><a:schemeClr val="phClr"><a:satMod val="110000"/><a:lumMod val="100000"/><a:shade val="100000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:lumMod val="99000"/><a:satMod val="120000"/><a:shade val="78000"/></a:schemeClr></a:gs></a:gsLst><a:lin ang="5400000" scaled="0"/></a:gradFill></a:fillStyleLst><a:lnStyleLst><a:ln w="12700" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/><a:miter lim="800000"/></a:ln><a:ln w="19050" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/><a:miter lim="800000"/></a:ln><a:ln w="25400" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/><a:miter lim="800000"/></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst><a:outerShdw blurRad="57150" dist="19050" dir="5400000" algn="ctr" rotWithShape="0"><a:srgbClr val="000000"><a:alpha val="63000"/></a:srgbClr></a:outerShdw></a:effectLst></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"><a:tint val="95000"/><a:satMod val="170000"/></a:schemeClr></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"><a:tint val="93000"/><a:satMod val="150000"/><a:shade val="98000"/><a:lumMod val="102000"/></a:schemeClr></a:gs><a:gs pos="50000"><a:schemeClr val="phClr"><a:tint val="98000"/><a:satMod val="130000"/><a:shade val="90000"/><a:lumMod val="103000"/></a:schemeClr></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"><a:shade val="63000"/><a:satMod val="120000"/></a:schemeClr></a:gs></a:gsLst><a:lin ang="5400000" scaled="0"/></a:gradFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements><a:objectDefaults><a:lnDef><a:spPr/><a:bodyPr/><a:lstStyle/><a:style><a:lnRef idx="2"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="0"><a:schemeClr val="accent1"/></a:fillRef><a:effectRef idx="1"><a:schemeClr val="accent1"/></a:effectRef><a:fontRef idx="minor"><a:schemeClr val="tx1"/></a:fontRef></a:style></a:lnDef></a:objectDefaults><a:extraClrSchemeLst/><a:extLst><a:ext uri="{05A4C25C-085E-4340-85A3-A5531E510DB2}"><thm15:themeFamily xmlns:thm15="http://schemas.microsoft.com/office/thememl/2012/main" name="Office Theme" id="{2E142A2C-CD16-42D6-873A-C26D2A0506FA}" vid="{1BDDFF52-6CD6-40A5-AB3C-68EB2F1E4D0A}"/></a:ext></a:extLst></a:theme>"###;
