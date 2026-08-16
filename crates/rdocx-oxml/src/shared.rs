//! Shared simple types and enums used across OOXML elements.

use crate::error::{OxmlError, Result};

/// `ST_Jc` — Paragraph justification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_Jc {
    Start,
    End,
    Center,
    Both,
    Distribute,
    Left,
    Right,
}

impl ST_Jc {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "start" | "left" => Ok(ST_Jc::Left),
            "end" | "right" => Ok(ST_Jc::Right),
            "center" => Ok(ST_Jc::Center),
            // Kashida justification stretches Arabic text by elongating the
            // connecting stroke rather than by widening spaces. Shaping that
            // faithfully is beyond this crate, and justified is what the three
            // values mean at the paragraph level. Rejecting them instead failed
            // the whole document open.
            "both" | "justify" | "lowKashida" | "mediumKashida" | "highKashida" => Ok(ST_Jc::Both),
            "distribute" => Ok(ST_Jc::Distribute),
            _ => Err(OxmlError::InvalidValue(format!("invalid ST_Jc: {s}"))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            ST_Jc::Start | ST_Jc::Left => "left",
            ST_Jc::End | ST_Jc::Right => "right",
            ST_Jc::Center => "center",
            ST_Jc::Both => "both",
            ST_Jc::Distribute => "distribute",
        }
    }
}

/// `ST_OnOff` — Boolean toggle, can be represented as "true"/"false", "1"/"0", or attribute absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_OnOff {
    On,
    Off,
}

impl ST_OnOff {
    pub fn from_str_or_default(s: Option<&str>) -> Self {
        match s {
            // If the attribute is absent or empty, the element presence means "on"
            None | Some("") | Some("true") | Some("1") | Some("on") => ST_OnOff::On,
            Some("false") | Some("0") | Some("off") => ST_OnOff::Off,
            Some(_) => ST_OnOff::Off,
        }
    }

    pub fn is_on(self) -> bool {
        self == ST_OnOff::On
    }

    pub fn to_str(self) -> &'static str {
        match self {
            ST_OnOff::On => "true",
            ST_OnOff::Off => "false",
        }
    }
}

/// `ST_UnderlineType` — Underline styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_Underline {
    None,
    Single,
    Words,
    Double,
    Thick,
    Dotted,
    Dash,
    DotDash,
    DotDotDash,
    Wave,
}

impl ST_Underline {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "none" => Ok(ST_Underline::None),
            "single" => Ok(ST_Underline::Single),
            "words" => Ok(ST_Underline::Words),
            "double" => Ok(ST_Underline::Double),
            "thick" => Ok(ST_Underline::Thick),
            "dotted" => Ok(ST_Underline::Dotted),
            "dash" => Ok(ST_Underline::Dash),
            "dotDash" => Ok(ST_Underline::DotDash),
            "dotDotDash" => Ok(ST_Underline::DotDotDash),
            "wave" => Ok(ST_Underline::Wave),
            _ => Err(OxmlError::InvalidValue(format!(
                "invalid ST_Underline: {s}"
            ))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            ST_Underline::None => "none",
            ST_Underline::Single => "single",
            ST_Underline::Words => "words",
            ST_Underline::Double => "double",
            ST_Underline::Thick => "thick",
            ST_Underline::Dotted => "dotted",
            ST_Underline::Dash => "dash",
            ST_Underline::DotDash => "dotDash",
            ST_Underline::DotDotDash => "dotDotDash",
            ST_Underline::Wave => "wave",
        }
    }
}

/// `ST_Border` — Border styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_Border {
    None,
    Single,
    Thick,
    Double,
    Dotted,
    Dashed,
    DotDash,
    DotDotDash,
    Triple,
    ThinThickSmallGap,
    ThickThinSmallGap,
    ThinThickMediumGap,
    ThickThinMediumGap,
    ThinThickLargeGap,
    ThickThinLargeGap,
    Wave,
    DoubleWave,
    ThreeDEmboss,
    ThreeDEngrave,
    Outset,
    Inset,
}

impl ST_Border {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "none" | "nil" => Ok(Self::None),
            "single" => Ok(Self::Single),
            "thick" => Ok(Self::Thick),
            "double" => Ok(Self::Double),
            "dotted" => Ok(Self::Dotted),
            "dashed" => Ok(Self::Dashed),
            "dotDash" => Ok(Self::DotDash),
            "dotDotDash" => Ok(Self::DotDotDash),
            "triple" => Ok(Self::Triple),
            "thinThickSmallGap" => Ok(Self::ThinThickSmallGap),
            "thickThinSmallGap" => Ok(Self::ThickThinSmallGap),
            "thinThickMediumGap" => Ok(Self::ThinThickMediumGap),
            "thickThinMediumGap" => Ok(Self::ThickThinMediumGap),
            "thinThickLargeGap" => Ok(Self::ThinThickLargeGap),
            "thickThinLargeGap" => Ok(Self::ThickThinLargeGap),
            "wave" => Ok(Self::Wave),
            "doubleWave" => Ok(Self::DoubleWave),
            "threeDEmboss" => Ok(Self::ThreeDEmboss),
            "threeDEngrave" => Ok(Self::ThreeDEngrave),
            "outset" => Ok(Self::Outset),
            "inset" => Ok(Self::Inset),
            _ => Err(OxmlError::InvalidValue(format!("invalid ST_Border: {s}"))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Single => "single",
            Self::Thick => "thick",
            Self::Double => "double",
            Self::Dotted => "dotted",
            Self::Dashed => "dashed",
            Self::DotDash => "dotDash",
            Self::DotDotDash => "dotDotDash",
            Self::Triple => "triple",
            Self::ThinThickSmallGap => "thinThickSmallGap",
            Self::ThickThinSmallGap => "thickThinSmallGap",
            Self::ThinThickMediumGap => "thinThickMediumGap",
            Self::ThickThinMediumGap => "thickThinMediumGap",
            Self::ThinThickLargeGap => "thinThickLargeGap",
            Self::ThickThinLargeGap => "thickThinLargeGap",
            Self::Wave => "wave",
            Self::DoubleWave => "doubleWave",
            Self::ThreeDEmboss => "threeDEmboss",
            Self::ThreeDEngrave => "threeDEngrave",
            Self::Outset => "outset",
            Self::Inset => "inset",
        }
    }
}

/// `ST_TabJc` — Tab stop alignment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_TabJc {
    Left,
    Center,
    Right,
    Decimal,
    Bar,
    Clear,
    Num,
}

impl ST_TabJc {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "left" | "start" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" | "end" => Ok(Self::Right),
            "decimal" => Ok(Self::Decimal),
            "bar" => Ok(Self::Bar),
            "clear" => Ok(Self::Clear),
            "num" => Ok(Self::Num),
            _ => Err(OxmlError::InvalidValue(format!("invalid ST_TabJc: {s}"))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Decimal => "decimal",
            Self::Bar => "bar",
            Self::Clear => "clear",
            Self::Num => "num",
        }
    }
}

/// `ST_TabTlc` — Tab leader character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_TabLeader {
    None,
    Dot,
    Hyphen,
    Underscore,
    Heavy,
    MiddleDot,
}

impl ST_TabLeader {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "none" => Ok(Self::None),
            "dot" => Ok(Self::Dot),
            "hyphen" => Ok(Self::Hyphen),
            "underscore" => Ok(Self::Underscore),
            "heavy" => Ok(Self::Heavy),
            "middleDot" => Ok(Self::MiddleDot),
            _ => Err(OxmlError::InvalidValue(format!(
                "invalid ST_TabLeader: {s}"
            ))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Dot => "dot",
            Self::Hyphen => "hyphen",
            Self::Underscore => "underscore",
            Self::Heavy => "heavy",
            Self::MiddleDot => "middleDot",
        }
    }
}

/// `ST_SectionType` — Section break type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_SectionType {
    NextPage,
    Continuous,
    EvenPage,
    OddPage,
    NextColumn,
}

impl ST_SectionType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "nextPage" => Ok(Self::NextPage),
            "continuous" => Ok(Self::Continuous),
            "evenPage" => Ok(Self::EvenPage),
            "oddPage" => Ok(Self::OddPage),
            "nextColumn" => Ok(Self::NextColumn),
            _ => Err(OxmlError::InvalidValue(format!(
                "invalid ST_SectionType: {s}"
            ))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::NextPage => "nextPage",
            Self::Continuous => "continuous",
            Self::EvenPage => "evenPage",
            Self::OddPage => "oddPage",
            Self::NextColumn => "nextColumn",
        }
    }
}

/// `ST_PageOrientation` — Page orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_PageOrientation {
    Portrait,
    Landscape,
}

impl ST_PageOrientation {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "portrait" => Ok(Self::Portrait),
            "landscape" => Ok(Self::Landscape),
            _ => Err(OxmlError::InvalidValue(format!(
                "invalid ST_PageOrientation: {s}"
            ))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Portrait => "portrait",
            Self::Landscape => "landscape",
        }
    }
}

/// `ST_HighlightColor` — Highlight colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_HighlightColor {
    Black,
    Blue,
    Cyan,
    DarkBlue,
    DarkCyan,
    DarkGray,
    DarkGreen,
    DarkMagenta,
    DarkRed,
    DarkYellow,
    Green,
    LightGray,
    Magenta,
    None,
    Red,
    White,
    Yellow,
}

impl ST_HighlightColor {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "black" => Ok(Self::Black),
            "blue" => Ok(Self::Blue),
            "cyan" => Ok(Self::Cyan),
            "darkBlue" => Ok(Self::DarkBlue),
            "darkCyan" => Ok(Self::DarkCyan),
            "darkGray" => Ok(Self::DarkGray),
            "darkGreen" => Ok(Self::DarkGreen),
            "darkMagenta" => Ok(Self::DarkMagenta),
            "darkRed" => Ok(Self::DarkRed),
            "darkYellow" => Ok(Self::DarkYellow),
            "green" => Ok(Self::Green),
            "lightGray" => Ok(Self::LightGray),
            "magenta" => Ok(Self::Magenta),
            "none" => Ok(Self::None),
            "red" => Ok(Self::Red),
            "white" => Ok(Self::White),
            "yellow" => Ok(Self::Yellow),
            _ => Err(OxmlError::InvalidValue(format!(
                "invalid ST_HighlightColor: {s}"
            ))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::Blue => "blue",
            Self::Cyan => "cyan",
            Self::DarkBlue => "darkBlue",
            Self::DarkCyan => "darkCyan",
            Self::DarkGray => "darkGray",
            Self::DarkGreen => "darkGreen",
            Self::DarkMagenta => "darkMagenta",
            Self::DarkRed => "darkRed",
            Self::DarkYellow => "darkYellow",
            Self::Green => "green",
            Self::LightGray => "lightGray",
            Self::Magenta => "magenta",
            Self::None => "none",
            Self::Red => "red",
            Self::White => "white",
            Self::Yellow => "yellow",
        }
    }
}

#[cfg(test)]
mod tests {

    // F-X018, an unmodelled enumerated value must not fail a document open.

    #[test]
    fn the_parsers_still_reject_an_unknown_value() {
        // Tolerance lives at the call site, not in the type. A caller that
        // wants strictness keeps it.
        assert!(ST_Jc::from_str("nonsense").is_err());
        assert!(ST_Underline::from_str("nonsense").is_err());
        assert!(ST_Border::from_str("nonsense").is_err());
        assert!(ST_TabJc::from_str("nonsense").is_err());
        assert!(ST_TabLeader::from_str("nonsense").is_err());
        assert!(ST_SectionType::from_str("nonsense").is_err());
        assert!(ST_PageOrientation::from_str("nonsense").is_err());
        assert!(ST_HighlightColor::from_str("nonsense").is_err());
    }
    use super::*;

    // F-X014, kashida justification values.

    #[test]
    fn kashida_justification_maps_to_both() {
        for value in ["lowKashida", "mediumKashida", "highKashida"] {
            assert_eq!(
                ST_Jc::from_str(value).unwrap(),
                ST_Jc::Both,
                "{value} should justify"
            );
        }
    }

    #[test]
    fn an_unknown_justification_is_still_rejected() {
        // The story widens the accepted set. It does not remove the check.
        assert!(ST_Jc::from_str("sideways").is_err());
        assert!(ST_Jc::from_str("").is_err());
    }

    #[test]
    fn a_document_using_kashida_justification_still_opens() {
        use crate::document::CT_Document;

        for value in ["lowKashida", "mediumKashida", "highKashida"] {
            let xml = format!(
                r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:pPr><w:jc w:val="{value}"/><w:keepNext/></w:pPr>
  <w:r><w:t>Arabic justified text.</w:t></w:r></w:p></w:body></w:document>"#
            );

            let document = CT_Document::from_xml(xml.as_bytes())
                .unwrap_or_else(|e| panic!("a document using {value} must open, got {e}"));

            let crate::document::BodyContent::Paragraph(paragraph) = &document.body.content[0]
            else {
                panic!("expected a paragraph");
            };
            let properties = paragraph.properties.as_ref().expect("properties survive");
            assert_eq!(properties.jc, Some(ST_Jc::Both), "{value} justifies");
            assert_eq!(
                properties.keep_next,
                Some(true),
                "{value} must not cost the paragraph its sibling properties"
            );
            assert_eq!(paragraph.text(), "Arabic justified text.");
        }
    }
}
