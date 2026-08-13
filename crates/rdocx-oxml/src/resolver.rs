//! Source-aware Word formatting resolution.
//!
//! This module deliberately consumes unmerged document defaults, numbering,
//! typed style chains, and direct properties. `CT_PPr::merge_from` and
//! `CT_RPr::merge_from` are structural conveniences; they are not the Word
//! cascade engine because merging first erases the provenance needed to
//! interpret toggle properties, numbering removal, and style-associated
//! numbering levels.

use std::collections::HashSet;

use crate::numbering::{CT_Numbering, EffectiveNumberingLevel};
use crate::properties::{CT_PPr, CT_RPr};
use crate::styles::{CT_Style, CT_Styles, StyleType};

/// Effective paragraph properties plus the numbering level selected while
/// the sources were still distinct.
#[derive(Debug, Clone)]
pub struct ResolvedParagraph<'a> {
    pub properties: CT_PPr,
    pub numbering_level: Option<EffectiveNumberingLevel<'a>>,
}

/// Resolves Word formatting while retaining source provenance through every
/// phase of the cascade.
#[derive(Debug, Clone, Copy)]
pub struct FormattingResolver<'a> {
    styles: &'a CT_Styles,
    numbering: Option<&'a CT_Numbering>,
}

impl<'a> FormattingResolver<'a> {
    pub fn new(styles: &'a CT_Styles, numbering: Option<&'a CT_Numbering>) -> Self {
        Self { styles, numbering }
    }

    /// Resolve docDefaults, numbering context, the typed paragraph style
    /// chain, and direct formatting in explicit phases.
    pub fn resolve_paragraph(&self, direct: Option<&CT_PPr>) -> ResolvedParagraph<'a> {
        let defaults = self
            .styles
            .doc_defaults
            .as_ref()
            .and_then(|defaults| defaults.ppr.as_ref());
        let style_id = direct
            .and_then(|properties| properties.style_id.as_deref())
            .or_else(|| {
                self.styles
                    .get_default(StyleType::Paragraph)
                    .map(|style| style.style_id.as_str())
            });
        let style_properties = self.resolve_paragraph_style_values(style_id);

        // numId=0 is a removal operation. Select the winning source before
        // applying any properties so it cannot be mistaken for a real list.
        let selected_num_id = direct
            .and_then(|properties| properties.num_id)
            .or(style_properties.num_id)
            .or_else(|| defaults.and_then(|properties| properties.num_id));
        let active_num_id = selected_num_id.filter(|num_id| *num_id != 0);

        // A direct ilvl is absolute. Otherwise a numbering level associated
        // with the applied paragraph style wins over style-level ilvl.
        let direct_level = direct.and_then(|properties| properties.num_ilvl);
        let style_level = style_properties
            .num_ilvl
            .or_else(|| defaults.and_then(|properties| properties.num_ilvl));
        let numbering_level = active_num_id.and_then(|num_id| {
            let numbering = self.numbering?;
            if let Some(level) = direct_level {
                return numbering.get_effective_level(num_id, level);
            }
            style_id
                .and_then(|style_id| numbering.get_effective_level_for_style(num_id, style_id))
                .or_else(|| numbering.get_effective_level(num_id, style_level.unwrap_or(0)))
        });

        let mut effective = CT_PPr::default();
        if let Some(defaults) = defaults {
            apply_paragraph_values(&mut effective, defaults);
        }
        if let Some(level_properties) = numbering_level.and_then(|level| level.level.ppr.as_ref()) {
            apply_paragraph_values(&mut effective, level_properties);
        }
        apply_paragraph_values(&mut effective, &style_properties);
        if let Some(direct) = direct {
            apply_paragraph_values(&mut effective, direct);
        }

        match selected_num_id {
            Some(0) => {
                effective.num_id = Some(0);
                effective.num_ilvl = direct_level;
            }
            Some(num_id) => {
                effective.num_id = Some(num_id);
                effective.num_ilvl = numbering_level
                    .map(|level| level.level.ilvl)
                    .or(direct_level)
                    .or(style_level);
            }
            None => {
                effective.num_id = None;
                effective.num_ilvl = None;
            }
        }

        ResolvedParagraph {
            properties: effective,
            numbering_level,
        }
    }

    /// Resolve run defaults, the typed paragraph-style chain, the typed
    /// character-style chain, and direct run formatting.
    pub fn resolve_run(
        &self,
        paragraph_direct: Option<&CT_PPr>,
        run_direct: Option<&CT_RPr>,
    ) -> CT_RPr {
        let paragraph_style_id = paragraph_direct
            .and_then(|properties| properties.style_id.as_deref())
            .or_else(|| {
                self.styles
                    .get_default(StyleType::Paragraph)
                    .map(|style| style.style_id.as_str())
            });
        let character_style_id = run_direct.and_then(|properties| properties.style_id.as_deref());
        self.resolve_run_sources(paragraph_style_id, character_style_id, run_direct)
    }

    /// Resolve styles without requiring concrete paragraph/run nodes.
    pub fn resolve_run_sources(
        &self,
        paragraph_style_id: Option<&str>,
        character_style_id: Option<&str>,
        direct: Option<&CT_RPr>,
    ) -> CT_RPr {
        let mut effective = CT_RPr::default();
        if let Some(defaults) = self
            .styles
            .doc_defaults
            .as_ref()
            .and_then(|defaults| defaults.rpr.as_ref())
        {
            apply_direct_run_values(&mut effective, defaults);
        }

        let paragraph_style_id = paragraph_style_id.or_else(|| {
            self.styles
                .get_default(StyleType::Paragraph)
                .map(|style| style.style_id.as_str())
        });
        for style in self.typed_style_chain(paragraph_style_id, StyleType::Paragraph) {
            if let Some(properties) = &style.rpr {
                apply_style_run_values(&mut effective, properties);
            }
        }
        for style in self.typed_style_chain(character_style_id, StyleType::Character) {
            if let Some(properties) = &style.rpr {
                apply_style_run_values(&mut effective, properties);
            }
        }
        if let Some(direct) = direct {
            apply_direct_run_values(&mut effective, direct);
        }
        effective
    }

    /// Resolve paragraph defaults plus a typed paragraph style chain, without
    /// numbering or direct formatting.
    pub fn resolve_paragraph_style(&self, style_id: Option<&str>) -> CT_PPr {
        let direct = style_id.map(|style_id| CT_PPr {
            style_id: Some(style_id.to_string()),
            ..Default::default()
        });
        self.resolve_paragraph(direct.as_ref()).properties
    }

    fn resolve_paragraph_style_values(&self, style_id: Option<&str>) -> CT_PPr {
        let mut properties = CT_PPr::default();
        for style in self.typed_style_chain(style_id, StyleType::Paragraph) {
            if let Some(style_properties) = &style.ppr {
                apply_paragraph_values(&mut properties, style_properties);
            }
        }
        properties
    }

    fn typed_style_chain(
        &self,
        style_id: Option<&str>,
        expected_type: StyleType,
    ) -> Vec<&'a CT_Style> {
        let mut chain = Vec::new();
        let mut current_id = style_id.map(str::to_owned);
        let mut seen = HashSet::new();
        while let Some(style_id) = current_id {
            if !seen.insert(style_id.clone()) {
                break;
            }
            let Some(style) = self.styles.get_by_id(&style_id) else {
                break;
            };
            if style.style_type != expected_type {
                break;
            }
            chain.push(style);
            current_id = style.based_on.clone();
        }
        chain.reverse();
        chain
    }
}

fn apply_paragraph_values(target: &mut CT_PPr, source: &CT_PPr) {
    macro_rules! apply {
        ($($field:ident),+ $(,)?) => {
            $(if source.$field.is_some() {
                target.$field = source.$field.clone();
            })+
        };
    }
    apply!(
        style_id,
        jc,
        space_before,
        space_after,
        line_spacing,
        line_rule,
        before_autospacing,
        after_autospacing,
        ind_left,
        ind_right,
        ind_first_line,
        ind_hanging,
        keep_next,
        keep_lines,
        page_break_before,
        widow_control,
        suppress_auto_hyphens,
        outline_lvl,
        borders,
        tabs,
        shading,
        rpr,
        num_ilvl,
        num_id,
        sect_pr,
    );
}

fn apply_direct_run_values(target: &mut CT_RPr, source: &CT_RPr) {
    macro_rules! apply {
        ($($field:ident),+ $(,)?) => {
            $(if source.$field.is_some() {
                target.$field = source.$field.clone();
            })+
        };
    }
    apply!(
        style_id,
        font_ascii,
        font_hansi,
        font_east_asia,
        font_cs,
        font_ascii_theme,
        font_hansi_theme,
        bold,
        bold_cs,
        italic,
        italic_cs,
        underline,
        strike,
        dstrike,
        sz,
        sz_cs,
        color,
        color_theme,
        highlight,
        caps,
        small_caps,
        vert_align,
        spacing,
        width_scale,
        position,
        shading,
        vanish,
    );
}

fn apply_style_run_values(target: &mut CT_RPr, source: &CT_RPr) {
    let prior = target.clone();
    apply_direct_run_values(target, source);
    target.bold = apply_style_toggle(prior.bold, source.bold);
    target.bold_cs = apply_style_toggle(prior.bold_cs, source.bold_cs);
    target.italic = apply_style_toggle(prior.italic, source.italic);
    target.italic_cs = apply_style_toggle(prior.italic_cs, source.italic_cs);
    target.strike = apply_style_toggle(prior.strike, source.strike);
    target.dstrike = apply_style_toggle(prior.dstrike, source.dstrike);
    target.caps = apply_style_toggle(prior.caps, source.caps);
    target.small_caps = apply_style_toggle(prior.small_caps, source.small_caps);
    target.vanish = apply_style_toggle(prior.vanish, source.vanish);
}

fn apply_style_toggle(inherited: Option<bool>, style: Option<bool>) -> Option<bool> {
    match style {
        Some(true) => Some(!inherited.unwrap_or(false)),
        Some(false) | None => inherited,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numbering::{CT_AbstractNum, CT_Lvl, CT_Num};
    use crate::styles::CT_DocDefaults;
    use crate::units::Twips;

    fn style(style_id: &str, style_type: StyleType) -> CT_Style {
        CT_Style {
            style_id: style_id.to_string(),
            style_type,
            name: None,
            based_on: None,
            next_style: None,
            is_default: false,
            ppr: None,
            rpr: None,
        }
    }

    #[test]
    fn direct_toggle_is_absolute_after_style_toggle() {
        let mut paragraph_style = style("Paragraph", StyleType::Paragraph);
        paragraph_style.rpr = Some(CT_RPr {
            bold: Some(true),
            ..Default::default()
        });
        let styles = CT_Styles {
            doc_defaults: Some(CT_DocDefaults {
                rpr: Some(CT_RPr {
                    bold: Some(true),
                    ..Default::default()
                }),
                ppr: None,
            }),
            styles: vec![paragraph_style],
        };
        let direct = CT_RPr {
            bold: Some(false),
            ..Default::default()
        };
        let resolved = FormattingResolver::new(&styles, None).resolve_run_sources(
            Some("Paragraph"),
            None,
            Some(&direct),
        );
        assert_eq!(resolved.bold, Some(false));
    }

    #[test]
    fn false_style_toggle_keeps_the_inherited_value() {
        let mut base = style("Base", StyleType::Paragraph);
        base.rpr = Some(CT_RPr {
            italic: Some(true),
            ..Default::default()
        });
        let mut derived = style("Derived", StyleType::Paragraph);
        derived.based_on = Some("Base".to_string());
        derived.rpr = Some(CT_RPr {
            italic: Some(false),
            ..Default::default()
        });
        let styles = CT_Styles {
            doc_defaults: None,
            styles: vec![base, derived],
        };

        let resolved =
            FormattingResolver::new(&styles, None).resolve_run_sources(Some("Derived"), None, None);
        assert_eq!(resolved.italic, Some(true));
    }

    #[test]
    fn based_on_does_not_cross_style_types() {
        let mut paragraph = style("Paragraph", StyleType::Paragraph);
        paragraph.based_on = Some("Character".to_string());
        let mut character = style("Character", StyleType::Character);
        character.rpr = Some(CT_RPr {
            italic: Some(true),
            ..Default::default()
        });
        let styles = CT_Styles {
            doc_defaults: None,
            styles: vec![paragraph, character],
        };
        let resolved = FormattingResolver::new(&styles, None).resolve_run_sources(
            Some("Paragraph"),
            None,
            None,
        );
        assert_eq!(resolved.italic, None);
    }

    #[test]
    fn direct_num_id_zero_removes_inherited_numbering() {
        let mut paragraph = style("List", StyleType::Paragraph);
        paragraph.ppr = Some(CT_PPr {
            num_id: Some(7),
            num_ilvl: Some(2),
            ..Default::default()
        });
        let styles = CT_Styles {
            doc_defaults: None,
            styles: vec![paragraph],
        };
        let direct = CT_PPr {
            style_id: Some("List".to_string()),
            num_id: Some(0),
            ..Default::default()
        };
        let resolved = FormattingResolver::new(&styles, None).resolve_paragraph(Some(&direct));
        assert_eq!(resolved.properties.num_id, Some(0));
        assert_eq!(resolved.properties.num_ilvl, None);
        assert!(resolved.numbering_level.is_none());
    }

    #[test]
    fn numbering_p_style_beats_style_level_and_style_beats_numbering_values() {
        let mut paragraph = style("Associated", StyleType::Paragraph);
        paragraph.ppr = Some(CT_PPr {
            num_id: Some(7),
            num_ilvl: Some(1),
            ind_left: Some(Twips(900)),
            ..Default::default()
        });
        let styles = CT_Styles {
            doc_defaults: None,
            styles: vec![paragraph],
        };
        let mut level = CT_Lvl::new(3);
        level.p_style = Some("Associated".to_string());
        level.ppr = Some(CT_PPr {
            ind_left: Some(Twips(300)),
            ..Default::default()
        });
        let mut abstract_num = CT_AbstractNum::new(2);
        abstract_num.levels.push(level);
        let numbering = CT_Numbering {
            abstract_nums: vec![abstract_num],
            nums: vec![CT_Num {
                num_id: 7,
                abstract_num_id: 2,
                level_overrides: Vec::new(),
            }],
        };
        let direct = CT_PPr {
            style_id: Some("Associated".to_string()),
            ..Default::default()
        };
        let resolved =
            FormattingResolver::new(&styles, Some(&numbering)).resolve_paragraph(Some(&direct));
        assert_eq!(resolved.properties.num_ilvl, Some(3));
        assert_eq!(resolved.properties.ind_left, Some(Twips(900)));
    }
}
