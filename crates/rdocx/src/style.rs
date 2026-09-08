//! Style access and manipulation for documents.

use std::collections::{HashMap, HashSet};

use rdocx_oxml::properties::{CT_PPr, CT_RPr};
use rdocx_oxml::styles::{CT_Style, CT_Styles, CT_TblStylePr, StyleType};
use rdocx_oxml::table::{CT_TblPr, CT_TcPr};

use crate::{Error, Result};

/// An immutable reference to a style definition.
pub struct Style<'a> {
    pub(crate) inner: &'a CT_Style,
}

impl<'a> Style<'a> {
    /// The style ID (used to reference this style).
    pub fn style_id(&self) -> &str {
        &self.inner.style_id
    }

    /// The display name of the style.
    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    /// The style ID this style is based on.
    pub fn based_on(&self) -> Option<&str> {
        self.inner.based_on.as_deref()
    }

    /// The kind of content this style formats.
    pub fn style_type(&self) -> StyleType {
        self.inner.style_type
    }

    /// The reciprocal paragraph or character style linked to this style.
    pub fn linked_style(&self) -> Option<&str> {
        self.inner.linked_style.as_deref()
    }

    /// The style selected for the paragraph following this paragraph style.
    pub fn next_style(&self) -> Option<&str> {
        self.inner.next_style.as_deref()
    }

    /// The user-interface ordering priority for this style.
    pub fn priority(&self) -> Option<u32> {
        self.inner.ui_priority
    }

    /// Whether Word may redefine the style from direct formatting.
    pub fn auto_redefine(&self) -> Option<bool> {
        self.inner.auto_redefine
    }

    /// Whether the style is hidden from all user-interface lists.
    pub fn hidden(&self) -> Option<bool> {
        self.inner.hidden
    }

    /// Whether the style is hidden from the main style gallery.
    pub fn semi_hidden(&self) -> Option<bool> {
        self.inner.semi_hidden
    }

    /// Whether applying the style makes it visible in the user interface.
    pub fn unhide_when_used(&self) -> Option<bool> {
        self.inner.unhide_when_used
    }

    /// Whether the style appears in the quick style gallery.
    pub fn quick_format(&self) -> Option<bool> {
        self.inner.quick_format
    }

    /// Whether the style is locked against application in the user interface.
    pub fn locked(&self) -> Option<bool> {
        self.inner.locked
    }

    /// The style's paragraph properties.
    pub fn paragraph_properties(&self) -> Option<&CT_PPr> {
        self.inner.ppr.as_ref()
    }

    /// The style's run properties.
    pub fn run_properties(&self) -> Option<&CT_RPr> {
        self.inner.rpr.as_ref()
    }

    /// The style's base table properties.
    pub fn table_properties(&self) -> Option<&CT_TblPr> {
        self.inner.table_properties.as_ref()
    }

    /// Conditional table regions in source order.
    pub fn conditional_table_styles(&self) -> &[CT_TblStylePr] {
        &self.inner.conditional_table_styles
    }

    /// Whether this is the default style for its type.
    pub fn is_default(&self) -> bool {
        self.inner.is_default
    }
}

/// Builder for creating a new paragraph style.
pub struct StyleBuilder {
    style: CT_Style,
    cleared: u16,
}

pub(crate) const CLEAR_BASED_ON: u16 = 1 << 0;
pub(crate) const CLEAR_NEXT_STYLE: u16 = 1 << 1;
pub(crate) const CLEAR_LINKED_STYLE: u16 = 1 << 2;
pub(crate) const CLEAR_PRIORITY: u16 = 1 << 3;
pub(crate) const CLEAR_AUTO_REDEFINE: u16 = 1 << 4;
pub(crate) const CLEAR_HIDDEN: u16 = 1 << 5;
pub(crate) const CLEAR_SEMI_HIDDEN: u16 = 1 << 6;
pub(crate) const CLEAR_UNHIDE_WHEN_USED: u16 = 1 << 7;
pub(crate) const CLEAR_QUICK_FORMAT: u16 = 1 << 8;
pub(crate) const CLEAR_LOCKED: u16 = 1 << 9;
pub(crate) const CLEAR_PARAGRAPH_PROPERTIES: u16 = 1 << 10;
pub(crate) const CLEAR_RUN_PROPERTIES: u16 = 1 << 11;
pub(crate) const CLEAR_TABLE_PROPERTIES: u16 = 1 << 12;
pub(crate) const CLEAR_CONDITIONAL_TABLE_STYLES: u16 = 1 << 13;

impl StyleBuilder {
    /// Create a new paragraph style builder.
    pub fn paragraph(style_id: &str, name: &str) -> Self {
        StyleBuilder {
            style: CT_Style {
                style_id: style_id.to_string(),
                style_type: StyleType::Paragraph,
                name: Some(name.to_string()),
                based_on: None,
                next_style: None,
                linked_style: None,
                auto_redefine: None,
                hidden: None,
                ui_priority: None,
                semi_hidden: None,
                unhide_when_used: None,
                quick_format: None,
                locked: None,
                is_default: false,
                ppr: None,
                rpr: None,
                table_properties: None,
                table_properties_original: None,
                table_properties_xml: None,
                conditional_table_styles: Vec::new(),
                extra_attributes: Vec::new(),
                modeled_xml: Vec::new(),
                extra_xml: Vec::new(),
            },
            cleared: 0,
        }
    }

    /// Create a new character style builder.
    pub fn character(style_id: &str, name: &str) -> Self {
        StyleBuilder {
            style: CT_Style {
                style_id: style_id.to_string(),
                style_type: StyleType::Character,
                name: Some(name.to_string()),
                based_on: None,
                next_style: None,
                linked_style: None,
                auto_redefine: None,
                hidden: None,
                ui_priority: None,
                semi_hidden: None,
                unhide_when_used: None,
                quick_format: None,
                locked: None,
                is_default: false,
                ppr: None,
                rpr: None,
                table_properties: None,
                table_properties_original: None,
                table_properties_xml: None,
                conditional_table_styles: Vec::new(),
                extra_attributes: Vec::new(),
                modeled_xml: Vec::new(),
                extra_xml: Vec::new(),
            },
            cleared: 0,
        }
    }

    /// Create a new table style builder.
    pub fn table(style_id: &str, name: &str) -> Self {
        let mut builder = Self::paragraph(style_id, name);
        builder.style.style_type = StyleType::Table;
        builder
    }

    /// Set the parent style this one inherits from.
    pub fn based_on(mut self, style_id: &str) -> Self {
        self.style.based_on = Some(style_id.to_string());
        self.cleared &= !CLEAR_BASED_ON;
        self
    }

    /// Remove the parent style during an update.
    pub fn clear_based_on(mut self) -> Self {
        self.style.based_on = None;
        self.cleared |= CLEAR_BASED_ON;
        self
    }

    /// Set the next style (applied to the following paragraph after pressing Enter).
    pub fn next_style(mut self, style_id: &str) -> Self {
        self.style.next_style = Some(style_id.to_string());
        self.cleared &= !CLEAR_NEXT_STYLE;
        self
    }

    /// Remove the following-paragraph style during an update.
    pub fn clear_next_style(mut self) -> Self {
        self.style.next_style = None;
        self.cleared |= CLEAR_NEXT_STYLE;
        self
    }

    /// Link this paragraph style to a character style, or the reverse.
    pub fn linked_style(mut self, style_id: &str) -> Self {
        self.style.linked_style = Some(style_id.to_string());
        self.cleared &= !CLEAR_LINKED_STYLE;
        self
    }

    /// Remove the reciprocal paragraph or character link during an update.
    pub fn clear_linked_style(mut self) -> Self {
        self.style.linked_style = None;
        self.cleared |= CLEAR_LINKED_STYLE;
        self
    }

    /// Set the style's user-interface ordering priority.
    pub fn priority(mut self, priority: u32) -> Self {
        self.style.ui_priority = Some(priority);
        self.cleared &= !CLEAR_PRIORITY;
        self
    }

    /// Remove the user-interface ordering priority during an update.
    pub fn clear_priority(mut self) -> Self {
        self.style.ui_priority = None;
        self.cleared |= CLEAR_PRIORITY;
        self
    }

    /// Set whether Word may redefine the style from direct formatting.
    pub fn auto_redefine(mut self, value: bool) -> Self {
        self.style.auto_redefine = Some(value);
        self.cleared &= !CLEAR_AUTO_REDEFINE;
        self
    }

    /// Remove the automatic-redefinition setting during an update.
    pub fn clear_auto_redefine(mut self) -> Self {
        self.style.auto_redefine = None;
        self.cleared |= CLEAR_AUTO_REDEFINE;
        self
    }

    /// Set whether the style is hidden from all user-interface lists.
    pub fn hidden(mut self, value: bool) -> Self {
        self.style.hidden = Some(value);
        self.cleared &= !CLEAR_HIDDEN;
        self
    }

    /// Remove the hidden setting during an update.
    pub fn clear_hidden(mut self) -> Self {
        self.style.hidden = None;
        self.cleared |= CLEAR_HIDDEN;
        self
    }

    /// Set whether the style is hidden from the main style gallery.
    pub fn semi_hidden(mut self, value: bool) -> Self {
        self.style.semi_hidden = Some(value);
        self.cleared &= !CLEAR_SEMI_HIDDEN;
        self
    }

    /// Remove the semi-hidden setting during an update.
    pub fn clear_semi_hidden(mut self) -> Self {
        self.style.semi_hidden = None;
        self.cleared |= CLEAR_SEMI_HIDDEN;
        self
    }

    /// Set whether applying the style makes it visible in the user interface.
    pub fn unhide_when_used(mut self, value: bool) -> Self {
        self.style.unhide_when_used = Some(value);
        self.cleared &= !CLEAR_UNHIDE_WHEN_USED;
        self
    }

    /// Remove the unhide-when-used setting during an update.
    pub fn clear_unhide_when_used(mut self) -> Self {
        self.style.unhide_when_used = None;
        self.cleared |= CLEAR_UNHIDE_WHEN_USED;
        self
    }

    /// Set whether the style appears in the quick style gallery.
    pub fn quick_format(mut self, value: bool) -> Self {
        self.style.quick_format = Some(value);
        self.cleared &= !CLEAR_QUICK_FORMAT;
        self
    }

    /// Remove the quick-format setting during an update.
    pub fn clear_quick_format(mut self) -> Self {
        self.style.quick_format = None;
        self.cleared |= CLEAR_QUICK_FORMAT;
        self
    }

    /// Set whether the style is locked against application in the user interface.
    pub fn locked(mut self, value: bool) -> Self {
        self.style.locked = Some(value);
        self.cleared &= !CLEAR_LOCKED;
        self
    }

    /// Remove the locked setting during an update.
    pub fn clear_locked(mut self) -> Self {
        self.style.locked = None;
        self.cleared |= CLEAR_LOCKED;
        self
    }

    /// Set paragraph properties for this style.
    pub fn paragraph_properties(mut self, ppr: CT_PPr) -> Self {
        self.style.ppr = Some(ppr);
        self.cleared &= !CLEAR_PARAGRAPH_PROPERTIES;
        self
    }

    /// Remove all paragraph properties during an update.
    pub fn clear_paragraph_properties(mut self) -> Self {
        self.style.ppr = None;
        self.cleared |= CLEAR_PARAGRAPH_PROPERTIES;
        self
    }

    /// Set run properties for this style.
    pub fn run_properties(mut self, rpr: CT_RPr) -> Self {
        self.style.rpr = Some(rpr);
        self.cleared &= !CLEAR_RUN_PROPERTIES;
        self
    }

    /// Remove all run properties during an update.
    pub fn clear_run_properties(mut self) -> Self {
        self.style.rpr = None;
        self.cleared |= CLEAR_RUN_PROPERTIES;
        self
    }

    /// Set base table properties for a table style.
    pub fn table_properties(mut self, properties: CT_TblPr) -> Self {
        self.style.table_properties = Some(properties);
        self.cleared &= !CLEAR_TABLE_PROPERTIES;
        self
    }

    /// Remove all base table properties during an update.
    pub fn clear_table_properties(mut self) -> Self {
        self.style.table_properties = None;
        self.cleared |= CLEAR_TABLE_PROPERTIES;
        self
    }

    /// Remove all conditional table regions during an update.
    pub fn clear_conditional_table_styles(mut self) -> Self {
        self.style.conditional_table_styles.clear();
        self.cleared |= CLEAR_CONDITIONAL_TABLE_STYLES;
        self
    }

    /// Add one conditional table style region in source order.
    pub fn conditional_table_style(
        mut self,
        region: &str,
        paragraph_properties: Option<CT_PPr>,
        table_properties: Option<CT_TblPr>,
        cell_properties: Option<CT_TcPr>,
    ) -> Self {
        self.style.conditional_table_styles.push(CT_TblStylePr {
            region: region.to_string(),
            paragraph_properties,
            table_properties,
            cell_properties,
            extra_attributes: Vec::new(),
            raw_xml: Vec::new(),
        });
        self
    }

    /// Build the style (consumed by Document::add_style).
    pub(crate) fn build(self) -> (CT_Style, u16) {
        (self.style, self.cleared)
    }
}

pub(crate) fn validate_style_graph(styles: &CT_Styles) -> Result<()> {
    let mut by_id = HashMap::new();
    for style in &styles.styles {
        if style.style_id.is_empty() {
            return Err(style_graph_error("style IDs cannot be empty"));
        }
        if by_id.insert(style.style_id.as_str(), style).is_some() {
            return Err(style_graph_error(format!(
                "duplicate style ID '{}'",
                style.style_id
            )));
        }
    }

    for style_type in [
        StyleType::Paragraph,
        StyleType::Character,
        StyleType::Table,
        StyleType::Numbering,
    ] {
        let defaults = styles
            .styles
            .iter()
            .filter(|style| style.style_type == style_type && style.is_default)
            .count();
        if defaults > 1 {
            return Err(style_graph_error(format!(
                "style type '{}' has more than one default",
                style_type.to_str()
            )));
        }
    }

    for style in &styles.styles {
        if style.style_type == StyleType::Character && style.ppr.is_some() {
            return Err(style_graph_error(format!(
                "character style '{}' cannot contain paragraph properties",
                style.style_id
            )));
        }
        if style.style_type != StyleType::Table
            && (style.table_properties.is_some() || !style.conditional_table_styles.is_empty())
        {
            return Err(style_graph_error(format!(
                "{} style '{}' cannot contain table properties",
                style.style_type.to_str(),
                style.style_id
            )));
        }
        let mut conditional_regions = HashSet::new();
        for conditional in &style.conditional_table_styles {
            if !matches!(
                conditional.region.as_str(),
                "wholeTable"
                    | "firstRow"
                    | "lastRow"
                    | "firstCol"
                    | "lastCol"
                    | "band1Vert"
                    | "band2Vert"
                    | "band1Horz"
                    | "band2Horz"
                    | "neCell"
                    | "nwCell"
                    | "seCell"
                    | "swCell"
            ) {
                return Err(style_graph_error(format!(
                    "table style '{}' has invalid conditional region '{}'",
                    style.style_id, conditional.region
                )));
            }
            if !conditional_regions.insert(conditional.region.as_str()) {
                return Err(style_graph_error(format!(
                    "table style '{}' repeats conditional region '{}'",
                    style.style_id, conditional.region
                )));
            }
        }

        if let Some(parent_id) = style.based_on.as_deref() {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                style_graph_error(format!(
                    "style '{}' is based on missing style '{parent_id}'",
                    style.style_id
                ))
            })?;
            if parent.style_type != style.style_type {
                return Err(style_graph_error(format!(
                    "style '{}' cannot be based on {} style '{parent_id}'",
                    style.style_id,
                    parent.style_type.to_str()
                )));
            }
        }

        if let Some(next_id) = style.next_style.as_deref() {
            if style.style_type != StyleType::Paragraph {
                return Err(style_graph_error(format!(
                    "{} style '{}' cannot declare a next style",
                    style.style_type.to_str(),
                    style.style_id
                )));
            }
            let next = by_id.get(next_id).ok_or_else(|| {
                style_graph_error(format!(
                    "style '{}' names missing next style '{next_id}'",
                    style.style_id
                ))
            })?;
            if next.style_type != StyleType::Paragraph {
                return Err(style_graph_error(format!(
                    "paragraph style '{}' has non-paragraph next style '{next_id}'",
                    style.style_id
                )));
            }
        }

        if let Some(linked_id) = style.linked_style.as_deref() {
            let linked = by_id.get(linked_id).ok_or_else(|| {
                style_graph_error(format!(
                    "style '{}' links to missing style '{linked_id}'",
                    style.style_id
                ))
            })?;
            let legal_types = matches!(
                (style.style_type, linked.style_type),
                (StyleType::Paragraph, StyleType::Character)
                    | (StyleType::Character, StyleType::Paragraph)
            );
            if !legal_types {
                return Err(style_graph_error(format!(
                    "{} style '{}' cannot link to {} style '{linked_id}'",
                    style.style_type.to_str(),
                    style.style_id,
                    linked.style_type.to_str()
                )));
            }
            if linked.linked_style.as_deref() != Some(style.style_id.as_str()) {
                return Err(style_graph_error(format!(
                    "linked styles '{}' and '{linked_id}' are not reciprocal",
                    style.style_id
                )));
            }
        }
    }

    for style in &styles.styles {
        let mut seen = HashSet::new();
        let mut current = Some(style.style_id.as_str());
        while let Some(style_id) = current {
            if !seen.insert(style_id) {
                return Err(style_graph_error(format!(
                    "based-on cycle contains style '{style_id}'"
                )));
            }
            current = by_id
                .get(style_id)
                .and_then(|current_style| current_style.based_on.as_deref());
        }
    }

    Ok(())
}

fn style_graph_error(message: impl Into<String>) -> Error {
    Error::Other(format!("invalid style graph: {}", message.into()))
}

/// Resolve the effective paragraph properties by walking the style inheritance chain.
pub fn resolve_paragraph_properties(style_id: Option<&str>, styles: &CT_Styles) -> CT_PPr {
    let mut effective = CT_PPr::default();

    // Start from docDefaults
    if let Some(ref defaults) = styles.doc_defaults
        && let Some(ref ppr) = defaults.ppr
    {
        effective.merge_from(ppr);
    }

    // Walk the selected style's basedOn chain.
    let selected_style_id = style_id.or_else(|| {
        styles
            .get_default(StyleType::Paragraph)
            .map(|style| style.style_id.as_str())
    });
    if let Some(sid) = selected_style_id {
        let chain = collect_style_chain(sid, styles);
        // Apply from most-base to most-derived
        for style in chain.iter().rev() {
            if let Some(ref ppr) = style.ppr {
                effective.merge_from(ppr);
            }
        }
    }

    effective
}

/// Resolve the effective run properties by walking the style inheritance chain.
pub fn resolve_run_properties(
    para_style_id: Option<&str>,
    run_style_id: Option<&str>,
    styles: &CT_Styles,
) -> CT_RPr {
    let mut effective = CT_RPr::default();

    // Start from docDefaults
    if let Some(ref defaults) = styles.doc_defaults
        && let Some(ref rpr) = defaults.rpr
    {
        effective.merge_from(rpr);
    }

    // Apply paragraph style's rpr
    let para_sid = para_style_id.or_else(|| {
        styles
            .get_default(StyleType::Paragraph)
            .map(|s| s.style_id.as_str())
    });
    if let Some(sid) = para_sid {
        let chain = collect_style_chain(sid, styles);
        for style in chain.iter().rev() {
            if let Some(ref rpr) = style.rpr {
                effective.merge_from(rpr);
            }
        }
    }

    // Apply character style's rpr
    if let Some(sid) = run_style_id.and_then(|style_id| character_style_id(style_id, styles)) {
        let chain = collect_style_chain(sid, styles);
        for style in chain.iter().rev() {
            if let Some(ref rpr) = style.rpr {
                effective.merge_from(rpr);
            }
        }
    }

    effective
}

fn character_style_id<'a>(style_id: &'a str, styles: &'a CT_Styles) -> Option<&'a str> {
    let style = styles.get_by_id(style_id)?;
    match style.style_type {
        StyleType::Character => Some(style.style_id.as_str()),
        StyleType::Paragraph => style.linked_style.as_deref().filter(|linked_id| {
            styles
                .get_by_id(linked_id)
                .is_some_and(|linked| linked.style_type == StyleType::Character)
        }),
        StyleType::Table | StyleType::Numbering => None,
    }
}

/// Collect the chain of styles from the given style up through basedOn ancestors.
fn collect_style_chain<'a>(style_id: &str, styles: &'a CT_Styles) -> Vec<&'a CT_Style> {
    let mut chain = Vec::new();
    let mut current_id = Some(style_id.to_string());
    let mut seen = std::collections::HashSet::new();

    while let Some(ref sid) = current_id {
        if !seen.insert(sid.clone()) {
            break; // Prevent cycles
        }
        if let Some(style) = styles.get_by_id(sid) {
            chain.push(style);
            current_id = style.based_on.clone();
        } else {
            break;
        }
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdocx_oxml::units::{HalfPoint, Twips};

    fn test_styles() -> CT_Styles {
        let mut styles = CT_Styles::new_default();

        // Add a Heading2 based on Heading1
        styles.styles.push(CT_Style {
            style_id: "Heading2".to_string(),
            style_type: StyleType::Paragraph,
            name: Some("heading 2".to_string()),
            based_on: Some("Heading1".to_string()),
            next_style: Some("Normal".to_string()),
            linked_style: None,
            auto_redefine: None,
            hidden: None,
            ui_priority: None,
            semi_hidden: None,
            unhide_when_used: None,
            quick_format: None,
            locked: None,
            is_default: false,
            ppr: Some(CT_PPr {
                space_before: Some(Twips(40)), // Override Heading1's 240
                ..Default::default()
            }),
            rpr: Some(CT_RPr {
                sz: Some(HalfPoint(26)), // Override Heading1's 32
                color: Some("2E74B5".to_string()),
                ..Default::default()
            }),
            table_properties: None,
            table_properties_original: None,
            table_properties_xml: None,
            conditional_table_styles: Vec::new(),
            extra_attributes: Vec::new(),
            modeled_xml: Vec::new(),
            extra_xml: Vec::new(),
        });

        styles
    }

    #[test]
    fn resolve_normal_paragraph() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Normal"), &styles);
        // Should have docDefaults' spacing
        assert_eq!(ppr.space_after, Some(Twips(160)));
    }

    #[test]
    fn resolve_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Heading1"), &styles);
        // keepNext from Heading1
        assert_eq!(ppr.keep_next, Some(true));
        // spaceBefore from Heading1 (overrides docDefaults which has none)
        assert_eq!(ppr.space_before, Some(Twips(240)));
        // spaceAfter from Heading1 overrides docDefaults
        assert_eq!(ppr.space_after, Some(Twips(0)));
    }

    #[test]
    fn resolve_heading2_inherits_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Heading2"), &styles);
        // keepNext inherited from Heading1
        assert_eq!(ppr.keep_next, Some(true));
        // spaceBefore overridden by Heading2
        assert_eq!(ppr.space_before, Some(Twips(40)));
    }

    #[test]
    fn resolve_heading2_rpr() {
        let styles = test_styles();
        let rpr = resolve_run_properties(Some("Heading2"), None, &styles);
        // Font from docDefaults
        assert_eq!(rpr.font_ascii, Some("Calibri".to_string()));
        // Size overridden by Heading2 (not Heading1's 32)
        assert_eq!(rpr.sz, Some(HalfPoint(26)));
        // Bold inherited from Heading1
        assert_eq!(rpr.bold, Some(true));
        // Color from Heading2
        assert_eq!(rpr.color, Some("2E74B5".to_string()));
    }

    #[test]
    fn resolve_default_when_no_style() {
        let mut styles = test_styles();
        for style in &mut styles.styles {
            style.is_default = style.style_id == "Heading2";
        }
        let ppr = resolve_paragraph_properties(None, &styles);
        assert_eq!(ppr.keep_next, Some(true));
        assert_eq!(ppr.space_before, Some(Twips(40)));
        assert_eq!(ppr.space_after, Some(Twips(0)));
    }
}
