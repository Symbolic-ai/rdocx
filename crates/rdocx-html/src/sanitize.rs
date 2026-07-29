//! Sanitisers for values that originate in the DOCX and end up in markup.
//!
//! Everything here treats the source document as untrusted. A DOCX can carry
//! arbitrary strings in places that look innocuous — a font name, a shading
//! colour, a hyperlink target — and those strings are interpolated into HTML
//! attributes and CSS declarations. Without validation a crafted document can
//! close the attribute it sits in and inject markup, so each value is checked
//! against the grammar it is supposed to follow and dropped when it does not
//! match.

/// Escape the HTML special characters in text content.
pub(crate) fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(ch),
        }
    }
    result
}

/// Escape a value being placed inside a double-quoted HTML attribute.
pub(crate) fn escape_html_attr(text: &str) -> String {
    escape_html(text)
}

/// Validate a WordprocessingML colour (`RRGGBB`, sometimes `RGB` or `RRGGBBAA`)
/// and render it as a CSS colour.
///
/// Returns `None` for `auto`, for anything that is not pure hex, and for any
/// other length — including CSS colour keywords, which are not valid here and
/// would otherwise let arbitrary text through.
pub(crate) fn hex_color(value: &str) -> Option<String> {
    if value.eq_ignore_ascii_case("auto") {
        return None;
    }
    let digits = value.strip_prefix('#').unwrap_or(value);
    if !matches!(digits.len(), 3 | 6 | 8) {
        return None;
    }
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{digits}"))
}

/// Render a font name as a single-quoted CSS `font-family` value.
///
/// Font names are free-form text in the DOCX, so anything that could terminate
/// the quoted string, the declaration, or the surrounding attribute disqualifies
/// the name and it is dropped in favour of the inherited font.
pub(crate) fn font_family(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    let safe = trimmed.chars().all(|c| {
        !c.is_control()
            && !matches!(
                c,
                '\'' | '"' | ';' | '{' | '}' | '(' | ')' | '<' | '>' | '&' | '\\' | '/'
            )
    });
    if !safe {
        return None;
    }
    Some(format!("'{trimmed}'"))
}

/// Schemes allowed in a generated `href`.
const SAFE_URL_SCHEMES: &[&str] = &["http", "https", "mailto", "tel", "ftp", "ftps"];

/// Return the URL if it is safe to emit as a link target, otherwise `None`.
///
/// Relative and fragment URLs are allowed. Absolute URLs must use a scheme from
/// [`SAFE_URL_SCHEMES`]; this is what keeps `javascript:` and `data:text/html`
/// targets in a hostile document from becoming script execution in whatever
/// renders the output.
pub(crate) fn safe_url(url: &str) -> Option<&str> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }

    // A scheme runs up to the first ':' and may only contain these characters;
    // anything else (a '/', '?', '#' or the end of input) means it is relative.
    let Some(scheme_end) = trimmed.find(':') else {
        // No colon at all: relative or fragment URL, nothing to gate.
        return Some(trimmed);
    };
    let scheme = &trimmed[..scheme_end];
    let looks_like_scheme = !scheme.is_empty()
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));

    if !looks_like_scheme {
        // e.g. "page.html?a:b" — no scheme, so it is relative and harmless.
        return Some(trimmed);
    }

    SAFE_URL_SCHEMES
        .iter()
        .any(|s| scheme.eq_ignore_ascii_case(s))
        .then_some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_must_be_hex() {
        assert_eq!(hex_color("FF0000").as_deref(), Some("#FF0000"));
        assert_eq!(hex_color("#abc").as_deref(), Some("#abc"));
        assert_eq!(hex_color("11223344").as_deref(), Some("#11223344"));
        assert_eq!(hex_color("auto"), None);
        assert_eq!(hex_color("red"), None);
        assert_eq!(hex_color("FF0000;} body{display:none"), None);
        assert_eq!(hex_color("\" onload=\"alert(1)"), None);
    }

    #[test]
    fn font_names_cannot_break_out() {
        assert_eq!(
            font_family("Times New Roman").as_deref(),
            Some("'Times New Roman'")
        );
        assert_eq!(font_family("Arial\" onmouseover=\"alert(1)"), None);
        assert_eq!(font_family("Arial'; background:url(x); '"), None);
        assert_eq!(font_family("  "), None);
    }

    #[test]
    fn only_safe_url_schemes_pass() {
        assert_eq!(safe_url("https://example.com"), Some("https://example.com"));
        assert_eq!(safe_url("mailto:a@b.c"), Some("mailto:a@b.c"));
        assert_eq!(safe_url("#anchor"), Some("#anchor"));
        assert_eq!(safe_url("page.html"), Some("page.html"));
        assert_eq!(safe_url("javascript:alert(1)"), None);
        assert_eq!(safe_url("JaVaScRiPt:alert(1)"), None);
        assert_eq!(safe_url("data:text/html;base64,PHNjcmlwdD4="), None);
        assert_eq!(safe_url("  javascript:alert(1)  "), None);
        assert_eq!(safe_url(""), None);
    }

    #[test]
    fn escaping_covers_quote_characters() {
        assert_eq!(
            escape_html("<a href=\"x\">&'"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;"
        );
    }
}
