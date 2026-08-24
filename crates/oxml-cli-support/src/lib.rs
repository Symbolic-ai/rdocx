//! Shared command-line contracts for OOXML tools.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
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

/// Fails before publication when any requested output already exists.
pub fn ensure_output_paths_available(paths: &[PathBuf]) -> io::Result<()> {
    let mut unique = BTreeSet::new();
    for path in paths {
        if !unique.insert(path.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate output path {}", path.display()),
            ));
        }
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("output already exists: {}", path.display()),
            ));
        }
    }
    Ok(())
}

/// Stages output files beside their final destinations and publishes them as a set.
pub struct StagedOutputSet {
    staged: Vec<StagedOutput>,
}

struct StagedOutput {
    final_path: PathBuf,
    temp_path: PathBuf,
}

impl StagedOutputSet {
    pub fn new() -> Self {
        Self { staged: Vec::new() }
    }

    pub fn stage_bytes(&mut self, final_path: &Path, bytes: &[u8]) -> io::Result<()> {
        if self
            .staged
            .iter()
            .any(|staged| staged.final_path == final_path)
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate output path {}", final_path.display()),
            ));
        }
        if final_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("output already exists: {}", final_path.display()),
            ));
        }
        let (temp_path, mut file) = create_temp_file(final_path)?;
        if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }
        self.staged.push(StagedOutput {
            final_path: final_path.to_path_buf(),
            temp_path,
        });
        Ok(())
    }

    pub fn publish(mut self) -> io::Result<()> {
        let mut published = Vec::new();
        for staged in &self.staged {
            match publish_staged_output(staged) {
                Ok(()) => {
                    published.push(staged.final_path.clone());
                }
                Err(error) => {
                    for path in published {
                        let _ = fs::remove_file(path);
                    }
                    for staged in &self.staged {
                        let _ = fs::remove_file(&staged.temp_path);
                    }
                    self.staged.clear();
                    return Err(error);
                }
            }
        }
        for staged in &self.staged {
            let _ = fs::remove_file(&staged.temp_path);
        }
        self.staged.clear();
        Ok(())
    }
}

impl Default for StagedOutputSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StagedOutputSet {
    fn drop(&mut self) {
        for staged in &self.staged {
            let _ = fs::remove_file(&staged.temp_path);
        }
    }
}

fn create_temp_file(final_path: &Path) -> io::Result<(PathBuf, File)> {
    let parent = final_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    for attempt in 0..1000_u32 {
        let temp_path = parent.join(format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            attempt
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "could not create a unique temporary file beside {}",
            final_path.display()
        ),
    ))
}

fn publish_staged_output(staged: &StagedOutput) -> io::Result<()> {
    let mut input = File::open(&staged.temp_path)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged.final_path)?;
    if let Err(error) = io::copy(&mut input, &mut output)
        .and_then(|_| output.sync_all())
        .map(|_| ())
    {
        let _ = fs::remove_file(&staged.final_path);
        return Err(error);
    }
    fs::remove_file(&staged.temp_path)
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
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let temp = std::env::temp_dir().join(format!("oxml-cli-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir(&temp).unwrap();
        temp
    }

    fn temp_entries(path: &Path) -> Vec<PathBuf> {
        fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".tmp"))
            })
            .collect()
    }

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

    #[test]
    fn staged_outputs_roll_back_published_files_when_later_publication_fails() {
        let temp = temp_dir("staged-rollback");
        let first = temp.join("first.png");
        let second = temp.join("second.png");
        let mut staged = StagedOutputSet::new();
        staged.stage_bytes(&first, b"first").unwrap();
        staged.stage_bytes(&second, b"second").unwrap();
        fs::create_dir(&second).unwrap();

        let error = staged.publish().unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert!(!first.exists());
        assert!(second.is_dir());
        assert!(temp_entries(&temp).is_empty());
        fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn staged_outputs_leave_no_temps_on_success_and_reject_duplicate_targets() {
        let temp = temp_dir("staged-success");
        let first = temp.join("first.png");
        let second = temp.join("second.png");
        ensure_output_paths_available(&[first.clone(), second.clone()]).unwrap();
        assert_eq!(
            ensure_output_paths_available(&[first.clone(), first.clone()])
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AlreadyExists
        );

        let mut staged = StagedOutputSet::new();
        staged.stage_bytes(&first, b"first").unwrap();
        assert_eq!(
            staged.stage_bytes(&first, b"duplicate").unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );
        staged.stage_bytes(&second, b"second").unwrap();
        staged.publish().unwrap();

        assert_eq!(fs::read(first).unwrap(), b"first");
        assert_eq!(fs::read(second).unwrap(), b"second");
        assert!(temp_entries(&temp).is_empty());
        fs::remove_dir_all(&temp).unwrap();
    }
}
