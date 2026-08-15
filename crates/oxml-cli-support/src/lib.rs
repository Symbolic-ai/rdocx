//! Shared command-line contracts for OOXML tools.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

const MAX_RANGE_VALUES: usize = 100_000;

/// An invalid shared command-line value.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    /// A range expression or one of its components is invalid.
    #[error("invalid one-based range component: {0:?}")]
    InvalidRange(String),
    /// A range expression would materialize too many values.
    #[error("range selection exceeds the maximum of {limit} values")]
    RangeTooLarge { limit: usize },
    /// A JSON envelope payload was not an object.
    #[error("JSON payload must be an object")]
    JsonPayloadNotObject,
    /// A JSON envelope payload tried to define the reserved schema field.
    #[error("JSON payload must not define reserved field \"schema\"")]
    ReservedSchemaField,
}

/// Parses positive one-based values and inclusive ranges.
///
/// Components are comma-separated. Whitespace around components and range
/// endpoints is ignored. The result is sorted and deduplicated. At most
/// 100,000 values may be requested across all components before
/// deduplication.
pub fn parse_range(input: &str) -> Result<Vec<usize>, Error> {
    let mut values = BTreeSet::new();
    let mut expansion_work = 0;

    if input.trim().is_empty() {
        return Err(Error::InvalidRange(input.to_owned()));
    }

    for raw_component in input.split(',') {
        let component = raw_component.trim();
        if component.is_empty() {
            return Err(Error::InvalidRange(raw_component.to_owned()));
        }

        let hyphen_count = component.bytes().filter(|byte| *byte == b'-').count();
        match hyphen_count {
            0 => {
                let value = parse_positive(component)?;
                charge_expansion_work(&mut expansion_work, 1)?;
                values.insert(value);
            }
            1 => {
                let (start, end) = component
                    .split_once('-')
                    .expect("one counted hyphen must split");
                let start = parse_positive(start.trim())?;
                let end = parse_positive(end.trim())?;
                if start > end {
                    return Err(Error::InvalidRange(component.to_owned()));
                }
                let cardinality = end
                    .checked_sub(start)
                    .and_then(|width| width.checked_add(1))
                    .ok_or(Error::RangeTooLarge {
                        limit: MAX_RANGE_VALUES,
                    })?;
                charge_expansion_work(&mut expansion_work, cardinality)?;
                values.extend(start..=end);
            }
            _ => return Err(Error::InvalidRange(component.to_owned())),
        }
    }

    Ok(values.into_iter().collect())
}

fn charge_expansion_work(total: &mut usize, additional: usize) -> Result<(), Error> {
    let charged = total.checked_add(additional).ok_or(Error::RangeTooLarge {
        limit: MAX_RANGE_VALUES,
    })?;
    if charged > MAX_RANGE_VALUES {
        return Err(Error::RangeTooLarge {
            limit: MAX_RANGE_VALUES,
        });
    }
    *total = charged;
    Ok(())
}

fn parse_positive(value: &str) -> Result<usize, Error> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| Error::InvalidRange(value.to_owned()))?;
    if parsed == 0 {
        return Err(Error::InvalidRange(value.to_owned()));
    }
    Ok(parsed)
}

/// Replaces or adds the requested extension while preserving the input path.
pub fn default_output_path(input: &Path, extension: &str) -> PathBuf {
    let mut output = input.to_path_buf();
    output.set_extension(extension.trim_start_matches('.'));
    output
}

/// Adds the versioned CLI schema field to an object payload.
pub fn json_envelope(mut payload: Value) -> Result<Value, Error> {
    let Value::Object(object) = &mut payload else {
        return Err(Error::JsonPayloadNotObject);
    };
    if object.contains_key("schema") {
        return Err(Error::ReservedSchemaField);
    }
    object.insert("schema".to_owned(), Value::from(1));
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;

    #[test]
    fn range_2_4_through_6_is_the_expected_set() {
        assert_eq!(parse_range("2,4-6").unwrap(), [2, 4, 5, 6]);
    }

    #[test]
    fn invalid_ranges_are_rejected_and_duplicates_are_normalized() {
        assert_eq!(parse_range("6, 2,4-6,2").unwrap(), [2, 4, 5, 6]);
        for invalid in ["", "0", "1,,2", "6-4", "2-", "-2", "two", "1-2-3"] {
            assert!(parse_range(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn ranges_too_large_to_materialize_are_rejected() {
        let expected = Err(Error::RangeTooLarge { limit: 100_000 });
        assert_eq!(parse_range("1-100001"), expected);
        assert_eq!(parse_range(&format!("1-{}", usize::MAX)), expected);
    }

    #[test]
    fn exactly_one_hundred_thousand_requested_values_are_accepted() {
        let values = parse_range("1-100000").unwrap();
        assert_eq!(values.len(), 100_000);
        assert_eq!(values.first(), Some(&1));
        assert_eq!(values.last(), Some(&100_000));
    }

    #[test]
    fn overlapping_ranges_cannot_amplify_expansion_work() {
        assert_eq!(
            parse_range("1-50001,1-50001"),
            Err(Error::RangeTooLarge { limit: 100_000 })
        );
    }

    #[test]
    fn json_envelope_has_schema_one_and_preserves_payload_fields() {
        let value = json_envelope(json!({"slides": 3, "metadata": {"title": "Deck"}}))
            .expect("object payload");
        assert_eq!(value["schema"], 1);
        assert_eq!(value["slides"], 3);
        assert_eq!(value["metadata"]["title"], "Deck");
        assert!(json_envelope(json!({"schema": 9})).is_err());
        assert!(json_envelope(json!([1, 2, 3])).is_err());
    }

    #[test]
    fn output_paths_replace_or_add_only_the_extension() {
        assert_eq!(
            default_output_path(Path::new("relative/report.docx"), "pdf"),
            Path::new("relative/report.pdf")
        );
        assert_eq!(
            default_output_path(Path::new("relative/report"), ".html"),
            Path::new("relative/report.html")
        );
        assert_eq!(
            default_output_path(Path::new("relative/report.final.docx"), "md"),
            Path::new("relative/report.final.md")
        );
    }
}
