//! Style resolution: cascade styles and generate numbering markers.
//!
//! Ports the logic from `crates/rdocx/src/style.rs` since rdocx-layout
//! depends on rdocx-oxml directly (not rdocx).

use std::collections::HashMap;

use rdocx_oxml::numbering::{CT_Numbering, EffectiveNumberingLevel, ST_NumberFormat};
use rdocx_oxml::properties::{CT_PPr, CT_RPr};
use rdocx_oxml::resolver::FormattingResolver;
use rdocx_oxml::styles::CT_Styles;
#[cfg(test)]
use rdocx_oxml::styles::{CT_Style, StyleType};

/// A fully resolved paragraph with merged properties and numbering info.
#[derive(Debug, Clone)]
pub struct ResolvedParagraph {
    /// Merged paragraph properties (style chain + direct formatting).
    pub ppr: CT_PPr,
    /// Resolved runs with merged run properties.
    pub runs: Vec<ResolvedRun>,
    /// Numbering marker info (if paragraph is part of a list).
    pub numbering: Option<ResolvedNumbering>,
}

/// A run with fully resolved properties.
#[derive(Debug, Clone)]
pub struct ResolvedRun {
    /// Merged run properties.
    pub rpr: CT_RPr,
    /// Run content items.
    pub content: Vec<rdocx_oxml::text::RunContent>,
}

/// Resolved numbering marker for a list paragraph.
#[derive(Debug, Clone)]
pub struct ResolvedNumbering {
    /// The text of the marker (e.g., "1.", "a)", bullet char).
    pub marker_text: String,
    /// Run properties for the marker.
    pub marker_rpr: CT_RPr,
}

/// Tracks numbering counters across paragraphs.
pub struct NumberingState {
    /// (numId, ilvl) → current count
    counters: HashMap<(u32, u32), u32>,
}

impl Default for NumberingState {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberingState {
    pub fn new() -> Self {
        NumberingState {
            counters: HashMap::new(),
        }
    }

    /// Advance the counter for the given numId/ilvl and return the new value.
    /// Also resets any deeper levels.
    pub fn advance(&mut self, num_id: u32, ilvl: u32, start: u32) -> u32 {
        let key = (num_id, ilvl);
        let counter = self.counters.entry(key).or_insert(start - 1);
        *counter += 1;
        let value = *counter;

        // Reset deeper levels
        for deeper in (ilvl + 1)..=8 {
            self.counters.remove(&(num_id, deeper));
        }

        value
    }

    /// Get the current count for a level (without advancing).
    pub fn current(&self, num_id: u32, ilvl: u32) -> u32 {
        self.counters.get(&(num_id, ilvl)).copied().unwrap_or(0)
    }
}

/// Resolve paragraph properties by walking the style inheritance chain.
pub fn resolve_paragraph_properties(style_id: Option<&str>, styles: &CT_Styles) -> CT_PPr {
    FormattingResolver::new(styles, None).resolve_paragraph_style(style_id)
}

/// Resolve run properties by walking paragraph and character style chains.
pub fn resolve_run_properties(
    para_style_id: Option<&str>,
    run_style_id: Option<&str>,
    styles: &CT_Styles,
) -> CT_RPr {
    FormattingResolver::new(styles, None).resolve_run_sources(para_style_id, run_style_id, None)
}

/// The paragraph properties a numbering level carries, mainly its indentation.
///
/// In the property chain these are applied before paragraph styles and direct
/// formatting, so either later source can deliberately replace the list
/// level's indentation.
pub fn level_paragraph_properties(
    num_id: u32,
    ilvl: u32,
    numbering: &CT_Numbering,
) -> Option<&CT_PPr> {
    numbering
        .get_effective_level(num_id, ilvl)?
        .level
        .ppr
        .as_ref()
}

/// Generate the marker text for a numbered/bulleted list item.
pub fn generate_marker(
    num_id: u32,
    ilvl: u32,
    numbering: &CT_Numbering,
    state: &mut NumberingState,
) -> Option<ResolvedNumbering> {
    generate_marker_inner(num_id, ilvl, numbering, None, state)
}

/// Generate a marker while resolving numbering-style indirection.
pub fn generate_marker_with_styles(
    num_id: u32,
    ilvl: u32,
    numbering: &CT_Numbering,
    styles: &CT_Styles,
    state: &mut NumberingState,
) -> Option<ResolvedNumbering> {
    generate_marker_inner(num_id, ilvl, numbering, Some(styles), state)
}

fn generate_marker_inner(
    num_id: u32,
    ilvl: u32,
    numbering: &CT_Numbering,
    styles: Option<&CT_Styles>,
    state: &mut NumberingState,
) -> Option<ResolvedNumbering> {
    let abs = numbering.get_abstract_num_for(num_id)?;
    let effective_level = effective_level(numbering, styles, num_id, ilvl)?;
    let lvl = effective_level.level;

    // Counters belong to the abstract definition, not the numbering instance.
    // Writers such as Pandoc and LibreOffice emit a separate w:num per list
    // block while pointing them all at one w:abstractNum, and readers are
    // expected to carry the count across them. Keying on num_id restarted the
    // sequence at every block, so a two item list rendered as "1." twice.
    let counter_id = abs.abstract_num_id;

    let num_fmt = lvl.num_fmt.clone().unwrap_or(ST_NumberFormat::Decimal);
    let start = effective_level.start;
    let lvl_text = lvl.lvl_text.as_deref().unwrap_or("%1.");

    let marker_text = if num_fmt == ST_NumberFormat::Bullet {
        lvl_text.to_string()
    } else {
        let count = state.advance(counter_id, ilvl, start);
        format_lvl_text(
            lvl_text, num_id, counter_id, ilvl, count, numbering, styles, state,
        )
    };

    let marker_rpr = lvl.rpr.clone().unwrap_or_default();

    Some(ResolvedNumbering {
        marker_text,
        marker_rpr,
    })
}

/// Format level text by substituting %1, %2, etc. with formatted counters.
fn format_lvl_text(
    template: &str,
    num_id: u32,
    counter_id: u32,
    current_ilvl: u32,
    current_count: u32,
    numbering: &CT_Numbering,
    styles: Option<&CT_Styles>,
    state: &NumberingState,
) -> String {
    let mut result = template.to_string();
    for lvl_idx in 0..=8u32 {
        let placeholder = format!("%{}", lvl_idx + 1);
        if result.contains(&placeholder) {
            let count = if lvl_idx == current_ilvl {
                current_count
            } else {
                state.current(counter_id, lvl_idx)
            };
            let fmt = effective_level(numbering, styles, num_id, lvl_idx)
                .and_then(|level| level.level.num_fmt.clone())
                .unwrap_or(ST_NumberFormat::Decimal);
            let formatted = format_number(count, fmt);
            result = result.replace(&placeholder, &formatted);
        }
    }
    result
}

fn effective_level<'a>(
    numbering: &'a CT_Numbering,
    styles: Option<&CT_Styles>,
    num_id: u32,
    ilvl: u32,
) -> Option<EffectiveNumberingLevel<'a>> {
    match styles {
        Some(styles) => numbering.get_effective_level_with_styles(num_id, ilvl, styles),
        None => numbering.get_effective_level(num_id, ilvl),
    }
}

/// Format a number according to ST_NumberFormat.
fn format_number(n: u32, fmt: ST_NumberFormat) -> String {
    match fmt {
        ST_NumberFormat::Decimal => n.to_string(),
        ST_NumberFormat::UpperRoman => to_roman(n, true),
        ST_NumberFormat::LowerRoman => to_roman(n, false),
        ST_NumberFormat::UpperLetter => to_letter(n, true),
        ST_NumberFormat::LowerLetter => to_letter(n, false),
        ST_NumberFormat::Ordinal => format!("{n}"),
        ST_NumberFormat::Bullet | ST_NumberFormat::None => String::new(),
        ST_NumberFormat::Other(_) => n.to_string(),
    }
}

fn to_roman(mut n: u32, upper: bool) -> String {
    let vals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for &(value, numeral) in &vals {
        while n >= value {
            result.push_str(numeral);
            n -= value;
        }
    }
    if upper { result } else { result.to_lowercase() }
}

fn to_letter(n: u32, upper: bool) -> String {
    if n == 0 {
        return String::new();
    }
    let base = if upper { b'A' } else { b'a' };
    let idx = ((n - 1) % 26) as u8;
    String::from(char::from(base + idx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdocx_oxml::units::{HalfPoint, Twips};

    fn test_styles() -> CT_Styles {
        let mut styles = CT_Styles::new_default();
        styles.styles.push(CT_Style {
            style_id: "Heading2".to_string(),
            style_type: StyleType::Paragraph,
            name: Some("heading 2".to_string()),
            based_on: Some("Heading1".to_string()),
            next_style: Some("Normal".to_string()),
            is_default: false,
            ppr: Some(CT_PPr {
                space_before: Some(Twips(40)),
                ..Default::default()
            }),
            rpr: Some(CT_RPr {
                sz: Some(HalfPoint(26)),
                color: Some("2E74B5".to_string()),
                ..Default::default()
            }),
        });
        styles
    }

    #[test]
    fn resolve_normal_paragraph() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Normal"), &styles);
        assert_eq!(ppr.space_after, Some(Twips(160)));
    }

    #[test]
    fn resolve_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Heading1"), &styles);
        assert_eq!(ppr.keep_next, Some(true));
        assert_eq!(ppr.space_before, Some(Twips(240)));
        assert_eq!(ppr.space_after, Some(Twips(0)));
    }

    #[test]
    fn resolve_heading2_inherits_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Heading2"), &styles);
        assert_eq!(ppr.keep_next, Some(true));
        assert_eq!(ppr.space_before, Some(Twips(40)));
    }

    #[test]
    fn resolve_heading2_rpr() {
        let styles = test_styles();
        let rpr = resolve_run_properties(Some("Heading2"), None, &styles);
        assert_eq!(rpr.font_ascii, Some("Calibri".to_string()));
        assert_eq!(rpr.sz, Some(HalfPoint(26)));
        assert_eq!(rpr.bold, Some(true));
        assert_eq!(rpr.color, Some("2E74B5".to_string()));
    }

    #[test]
    fn numbering_decimal_marker() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_numbered_list();

        let mut state = NumberingState::new();
        let marker1 = generate_marker(num_id, 0, &numbering, &mut state).unwrap();
        assert_eq!(marker1.marker_text, "1.");
        let marker2 = generate_marker(num_id, 0, &numbering, &mut state).unwrap();
        assert_eq!(marker2.marker_text, "2.");
    }

    #[test]
    fn numbering_bullet_marker() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_bullet_list();

        let mut state = NumberingState::new();
        let marker = generate_marker(num_id, 0, &numbering, &mut state).unwrap();
        assert_eq!(marker.marker_text, "\u{2022}");
    }

    #[test]
    fn numbering_marker_uses_full_level_and_start_overrides() {
        use rdocx_oxml::numbering::{CT_Lvl, CT_LvlOverride};

        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_numbered_list();
        let instance = numbering
            .nums
            .iter_mut()
            .find(|num| num.num_id == num_id)
            .unwrap();
        instance.level_overrides.push(CT_LvlOverride {
            ilvl: 0,
            start_override: Some(4),
            level: Some(CT_Lvl {
                num_fmt: Some(ST_NumberFormat::UpperLetter),
                lvl_text: Some("%1)".to_string()),
                ..CT_Lvl::new(0)
            }),
        });

        let marker = generate_marker(num_id, 0, &numbering, &mut NumberingState::new()).unwrap();
        assert_eq!(marker.marker_text, "D)");
    }

    #[test]
    fn numbering_sub_level_reset() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_numbered_list();

        let mut state = NumberingState::new();
        // Level 0: 1, 2
        generate_marker(num_id, 0, &numbering, &mut state);
        generate_marker(num_id, 0, &numbering, &mut state);
        // Level 1: a
        let sub = generate_marker(num_id, 1, &numbering, &mut state).unwrap();
        assert_eq!(sub.marker_text, "a.");
        // Back to level 0: 3 — this should reset level 1
        generate_marker(num_id, 0, &numbering, &mut state);
        let sub2 = generate_marker(num_id, 1, &numbering, &mut state).unwrap();
        assert_eq!(sub2.marker_text, "a."); // reset
    }

    #[test]
    fn roman_numeral_formatting() {
        assert_eq!(to_roman(1, true), "I");
        assert_eq!(to_roman(4, true), "IV");
        assert_eq!(to_roman(9, true), "IX");
        assert_eq!(to_roman(14, false), "xiv");
    }

    #[test]
    fn letter_formatting() {
        assert_eq!(to_letter(1, false), "a");
        assert_eq!(to_letter(26, false), "z");
        assert_eq!(to_letter(27, false), "a"); // wraps
        assert_eq!(to_letter(1, true), "A");
    }

    /// Two numbering instances that share one abstract definition are one
    /// list and must keep counting.
    ///
    /// Pandoc and LibreOffice both emit a separate `w:num` per list block
    /// pointing at the same `w:abstractNum`. Keying counters on num_id made
    /// every block restart, so a two item list rendered as "1." twice.
    #[test]
    fn shared_abstract_definition_continues_the_count() {
        let mut numbering = CT_Numbering::new();
        let first = numbering.add_numbered_list();
        let abstract_id = numbering
            .nums
            .iter()
            .find(|n| n.num_id == first)
            .unwrap()
            .abstract_num_id;

        // A second instance pointing at the same abstract definition.
        let second = numbering.nums.iter().map(|n| n.num_id).max().unwrap() + 1;
        numbering.nums.push(rdocx_oxml::numbering::CT_Num {
            num_id: second,
            abstract_num_id: abstract_id,
            level_overrides: Vec::new(),
        });

        let mut state = NumberingState::new();
        let a = generate_marker(first, 0, &numbering, &mut state).unwrap();
        let b = generate_marker(second, 0, &numbering, &mut state).unwrap();
        assert_eq!(a.marker_text, "1.");
        assert_eq!(b.marker_text, "2.", "the count must carry across instances");
    }

    /// Separate abstract definitions are separate lists and each restarts.
    #[test]
    fn separate_abstract_definitions_count_independently() {
        let mut numbering = CT_Numbering::new();
        let first = numbering.add_numbered_list();
        let second = numbering.add_numbered_list();

        let mut state = NumberingState::new();
        let a = generate_marker(first, 0, &numbering, &mut state).unwrap();
        let b = generate_marker(second, 0, &numbering, &mut state).unwrap();
        assert_eq!(a.marker_text, "1.");
        assert_eq!(b.marker_text, "1.");
    }

    /// Each level carries its own indent, and deeper levels step further in.
    #[test]
    fn level_paragraph_properties_expose_per_level_indent() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_numbered_list();

        let lvl0 = level_paragraph_properties(num_id, 0, &numbering)
            .expect("level 0 should carry paragraph properties");
        let lvl1 = level_paragraph_properties(num_id, 1, &numbering)
            .expect("level 1 should carry paragraph properties");

        let left0 = lvl0.ind_left.expect("level 0 indent").0;
        let left1 = lvl1.ind_left.expect("level 1 indent").0;
        assert!(
            left1 > left0,
            "level 1 must indent further than level 0, got {left0} then {left1}"
        );
    }
}
