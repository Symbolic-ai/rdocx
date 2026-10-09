//! Bibliography source identity, ordered XML and native citation formatting.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ops::Range;

use oxml_opc::OpcPackage;
use quick_xml::XmlVersion;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::reader::NsReader;

use crate::{Document, Error, Result};

const BIB_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/bibliography";
const CUSTOM_XML_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
const CUSTOM_XML_PROPS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps";
const CUSTOM_XML_PROPS_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.customXmlProperties+xml";

/// The seventeen standard bibliography source classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BibliographySourceKind {
    ArticleInAPeriodical,
    Book,
    BookSection,
    JournalArticle,
    ConferenceProceedings,
    Report,
    SoundRecording,
    Performance,
    Art,
    DocumentFromInternetSite,
    InternetSite,
    Film,
    Interview,
    Patent,
    ElectronicSource,
    Case,
    Misc,
}

impl BibliographySourceKind {
    fn xml_name(self) -> &'static str {
        match self {
            Self::ArticleInAPeriodical => "ArticleInAPeriodical",
            Self::Book => "Book",
            Self::BookSection => "BookSection",
            Self::JournalArticle => "JournalArticle",
            Self::ConferenceProceedings => "ConferenceProceedings",
            Self::Report => "Report",
            Self::SoundRecording => "SoundRecording",
            Self::Performance => "Performance",
            Self::Art => "Art",
            Self::DocumentFromInternetSite => "DocumentFromInternetSite",
            Self::InternetSite => "InternetSite",
            Self::Film => "Film",
            Self::Interview => "Interview",
            Self::Patent => "Patent",
            Self::ElectronicSource => "ElectronicSource",
            Self::Case => "Case",
            Self::Misc => "Misc",
        }
    }

    fn from_xml_name(name: &str) -> Option<Self> {
        match name {
            "ArticleInAPeriodical" => Some(Self::ArticleInAPeriodical),
            "Book" => Some(Self::Book),
            "BookSection" => Some(Self::BookSection),
            "JournalArticle" => Some(Self::JournalArticle),
            "ConferenceProceedings" => Some(Self::ConferenceProceedings),
            "Report" => Some(Self::Report),
            "SoundRecording" => Some(Self::SoundRecording),
            "Performance" => Some(Self::Performance),
            "Art" => Some(Self::Art),
            "DocumentFromInternetSite" => Some(Self::DocumentFromInternetSite),
            "InternetSite" => Some(Self::InternetSite),
            "Film" => Some(Self::Film),
            "Interview" => Some(Self::Interview),
            "Patent" => Some(Self::Patent),
            "ElectronicSource" => Some(Self::ElectronicSource),
            "Case" => Some(Self::Case),
            "Misc" => Some(Self::Misc),
            _ => None,
        }
    }
}

/// A standard contributor role in the ordered author list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BibliographyContributorRole {
    Artist,
    Author,
    BookAuthor,
    Compiler,
    Composer,
    Conductor,
    Counsel,
    Director,
    Editor,
    Interviewee,
    Interviewer,
    Inventor,
    Performer,
    ProducerName,
    Translator,
    Writer,
}

impl BibliographyContributorRole {
    fn xml_name(self) -> &'static str {
        match self {
            Self::Artist => "Artist",
            Self::Author => "Author",
            Self::BookAuthor => "BookAuthor",
            Self::Compiler => "Compiler",
            Self::Composer => "Composer",
            Self::Conductor => "Conductor",
            Self::Counsel => "Counsel",
            Self::Director => "Director",
            Self::Editor => "Editor",
            Self::Interviewee => "Interviewee",
            Self::Interviewer => "Interviewer",
            Self::Inventor => "Inventor",
            Self::Performer => "Performer",
            Self::ProducerName => "ProducerName",
            Self::Translator => "Translator",
            Self::Writer => "Writer",
        }
    }

    fn from_xml_name(name: &str) -> Option<Self> {
        match name {
            "Artist" => Some(Self::Artist),
            "Author" => Some(Self::Author),
            "BookAuthor" => Some(Self::BookAuthor),
            "Compiler" => Some(Self::Compiler),
            "Composer" => Some(Self::Composer),
            "Conductor" => Some(Self::Conductor),
            "Counsel" => Some(Self::Counsel),
            "Director" => Some(Self::Director),
            "Editor" => Some(Self::Editor),
            "Interviewee" => Some(Self::Interviewee),
            "Interviewer" => Some(Self::Interviewer),
            "Inventor" => Some(Self::Inventor),
            "Performer" => Some(Self::Performer),
            "ProducerName" => Some(Self::ProducerName),
            "Translator" => Some(Self::Translator),
            "Writer" => Some(Self::Writer),
            _ => None,
        }
    }
}

/// A standard textual source property, including repeated occurrences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BibliographySourceField {
    AbbreviatedCaseNumber,
    AlbumTitle,
    BookTitle,
    Broadcaster,
    BroadcastTitle,
    CaseNumber,
    ChapterNumber,
    City,
    Comments,
    ConferenceName,
    CountryRegion,
    Court,
    Day,
    DayAccessed,
    Department,
    Distributor,
    Edition,
    Institution,
    InternetSiteTitle,
    Issue,
    JournalName,
    Medium,
    Month,
    MonthAccessed,
    NumberVolumes,
    Pages,
    PatentNumber,
    PeriodicalTitle,
    ProductionCompany,
    PublicationTitle,
    Publisher,
    RecordingNumber,
    ReferenceOrder,
    Reporter,
    ShortTitle,
    StandardNumber,
    StateProvince,
    Station,
    Theater,
    ThesisType,
    Title,
    PatentType,
    Url,
    Version,
    Volume,
    Year,
    YearAccessed,
}

impl BibliographySourceField {
    fn xml_name(self) -> &'static str {
        match self {
            Self::AbbreviatedCaseNumber => "AbbreviatedCaseNumber",
            Self::AlbumTitle => "AlbumTitle",
            Self::BookTitle => "BookTitle",
            Self::Broadcaster => "Broadcaster",
            Self::BroadcastTitle => "BroadcastTitle",
            Self::CaseNumber => "CaseNumber",
            Self::ChapterNumber => "ChapterNumber",
            Self::City => "City",
            Self::Comments => "Comments",
            Self::ConferenceName => "ConferenceName",
            Self::CountryRegion => "CountryRegion",
            Self::Court => "Court",
            Self::Day => "Day",
            Self::DayAccessed => "DayAccessed",
            Self::Department => "Department",
            Self::Distributor => "Distributor",
            Self::Edition => "Edition",
            Self::Institution => "Institution",
            Self::InternetSiteTitle => "InternetSiteTitle",
            Self::Issue => "Issue",
            Self::JournalName => "JournalName",
            Self::Medium => "Medium",
            Self::Month => "Month",
            Self::MonthAccessed => "MonthAccessed",
            Self::NumberVolumes => "NumberVolumes",
            Self::Pages => "Pages",
            Self::PatentNumber => "PatentNumber",
            Self::PeriodicalTitle => "PeriodicalTitle",
            Self::ProductionCompany => "ProductionCompany",
            Self::PublicationTitle => "PublicationTitle",
            Self::Publisher => "Publisher",
            Self::RecordingNumber => "RecordingNumber",
            Self::ReferenceOrder => "RefOrder",
            Self::Reporter => "Reporter",
            Self::ShortTitle => "ShortTitle",
            Self::StandardNumber => "StandardNumber",
            Self::StateProvince => "StateProvince",
            Self::Station => "Station",
            Self::Theater => "Theater",
            Self::ThesisType => "ThesisType",
            Self::Title => "Title",
            Self::PatentType => "Type",
            Self::Url => "URL",
            Self::Version => "Version",
            Self::Volume => "Volume",
            Self::Year => "Year",
            Self::YearAccessed => "YearAccessed",
        }
    }

    fn from_xml_name(name: &str) -> Option<Self> {
        match name {
            "AbbreviatedCaseNumber" => Some(Self::AbbreviatedCaseNumber),
            "AlbumTitle" => Some(Self::AlbumTitle),
            "BookTitle" => Some(Self::BookTitle),
            "Broadcaster" => Some(Self::Broadcaster),
            "BroadcastTitle" => Some(Self::BroadcastTitle),
            "CaseNumber" => Some(Self::CaseNumber),
            "ChapterNumber" => Some(Self::ChapterNumber),
            "City" => Some(Self::City),
            "Comments" => Some(Self::Comments),
            "ConferenceName" => Some(Self::ConferenceName),
            "CountryRegion" => Some(Self::CountryRegion),
            "Court" => Some(Self::Court),
            "Day" => Some(Self::Day),
            "DayAccessed" => Some(Self::DayAccessed),
            "Department" => Some(Self::Department),
            "Distributor" => Some(Self::Distributor),
            "Edition" => Some(Self::Edition),
            "Institution" => Some(Self::Institution),
            "InternetSiteTitle" => Some(Self::InternetSiteTitle),
            "Issue" => Some(Self::Issue),
            "JournalName" => Some(Self::JournalName),
            "Medium" => Some(Self::Medium),
            "Month" => Some(Self::Month),
            "MonthAccessed" => Some(Self::MonthAccessed),
            "NumberVolumes" => Some(Self::NumberVolumes),
            "Pages" => Some(Self::Pages),
            "PatentNumber" => Some(Self::PatentNumber),
            "PeriodicalTitle" => Some(Self::PeriodicalTitle),
            "ProductionCompany" => Some(Self::ProductionCompany),
            "PublicationTitle" => Some(Self::PublicationTitle),
            "Publisher" => Some(Self::Publisher),
            "RecordingNumber" => Some(Self::RecordingNumber),
            "RefOrder" => Some(Self::ReferenceOrder),
            "Reporter" => Some(Self::Reporter),
            "ShortTitle" => Some(Self::ShortTitle),
            "StandardNumber" => Some(Self::StandardNumber),
            "StateProvince" => Some(Self::StateProvince),
            "Station" => Some(Self::Station),
            "Theater" => Some(Self::Theater),
            "ThesisType" => Some(Self::ThesisType),
            "Title" => Some(Self::Title),
            "Type" => Some(Self::PatentType),
            "URL" => Some(Self::Url),
            "Version" => Some(Self::Version),
            "Volume" => Some(Self::Volume),
            "Year" => Some(Self::Year),
            "YearAccessed" => Some(Self::YearAccessed),
            _ => None,
        }
    }
}

/// A pinned installed Word bibliography style edition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BibliographyStyle {
    ApaSixthEdition,
    Chicago,
    Gb7714,
    GostName,
    GostTitle,
    HarvardAnglia,
    Ieee,
    Iso690AuthorDate,
    Iso690Numeric,
    MlaSeventhEdition,
    Sist02,
    Turabian,
}

/// Ordered components of a person's bibliography name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographyPerson {
    pub first: Vec<String>,
    pub middle: Vec<String>,
    pub last: Vec<String>,
}

/// Personal or corporate authorship. Corporate is legal only for Author and Performer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BibliographyAuthor {
    People(Vec<BibliographyPerson>),
    Corporate(String),
}

/// One contributor occurrence in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographyContributor {
    pub role: BibliographyContributorRole,
    pub value: BibliographyAuthor,
}

/// One textual property occurrence in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographyProperty {
    pub field: BibliographySourceField,
    pub value: String,
}

/// Editable standard source data. Unmodelled imported XML remains package-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographySource {
    pub tag: String,
    pub guid: String,
    pub kind: BibliographySourceKind,
    /// Numeric source locale. Zero contributes no formatting override.
    /// Imported lexical spellings remain unchanged by inspection or a no-op replacement.
    pub locale: Option<u32>,
    pub contributors: Vec<BibliographyContributor>,
    pub properties: Vec<BibliographyProperty>,
}

/// Per-source citation switches, in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationSourceOptions {
    pub tag: String,
    pub pages: Option<String>,
    pub volume: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub suppress_author: bool,
    pub suppress_year: bool,
    pub suppress_title: bool,
}

/// Citation members and the field-wide formatting locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationOptions {
    pub sources: Vec<CitationSourceOptions>,
    /// Field-wide formatting locale. Sources with an explicit nonzero locale override it.
    /// An omitted or zero locale needs caller-provided application context when no source overrides it.
    pub locale: Option<u32>,
}

/// Formatting style, formatting locale and independent source selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographyOptions {
    pub style: BibliographyStyle,
    pub locale: Option<u32>,
    pub locale_filter: Option<u32>,
    pub tags: Vec<String>,
}

/// Preserved collection metadata and its recognized style, when present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographyStyleInfo {
    pub style_key: Option<String>,
    pub style_path: Option<String>,
    /// Standard Sources style metadata has no formatting locale, so this is absent.
    /// Formatting locale selectors belong to individual field instructions.
    pub locale: Option<u32>,
    pub supported_style: Option<BibliographyStyle>,
}

/// Source identity and the editable supported projection, when unambiguous.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographySourceInfo {
    pub tag: String,
    pub guid: Option<String>,
    pub source_type: Option<String>,
    /// Editable data when identity and modeled members have an unambiguous projection.
    pub supported: Option<BibliographySource>,
    /// Inspection limitations, including unknown or ambiguous imported locale identities.
    /// Absence of a numeric locale alone does not distinguish omitted data from a projection limit.
    pub diagnostics: Vec<String>,
}

/// Explicit update results. Unsupported producers retain their complete caches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibliographyUpdateReport {
    pub updated_citations: usize,
    pub rebuilt_bibliographies: usize,
    pub diagnostics: Vec<String>,
}

// Exact ranges retain repeated members, unknown attributes and producer prefixes.
#[derive(Debug)]
struct SourceXmlNode {
    namespace: String,
    local: String,
    name: String,
    full: Range<usize>,
    content: Range<usize>,
    attributes: Vec<(String, String, String)>,
    children: Vec<usize>,
    text: String,
    opaque_scalar: bool,
    empty: bool,
}

#[derive(Debug)]
struct SourceXml {
    nodes: Vec<SourceXmlNode>,
}

fn bibliography_error(message: impl Into<String>) -> Error {
    Error::Other(format!("bibliography: {}", message.into()))
}

fn namespace_value(value: ResolveResult<'_>) -> Result<String> {
    match value {
        ResolveResult::Bound(value) => String::from_utf8(value.as_ref().to_vec())
            .map_err(|_| bibliography_error("namespace is not UTF-8")),
        ResolveResult::Unbound => Ok(String::new()),
        ResolveResult::Unknown(prefix) => Err(bibliography_error(format!(
            "unbound namespace prefix {}",
            String::from_utf8_lossy(&prefix)
        ))),
    }
}

impl SourceXml {
    fn parse(xml: &[u8]) -> Result<Self> {
        oxml_core::xml::validate_strict_xml_1_0(xml)
            .map_err(|error| bibliography_error(format!("invalid source XML: {error:?}")))?;
        let mut reader = NsReader::from_reader(xml);
        let mut buffer = Vec::new();
        let mut nodes: Vec<SourceXmlNode> = Vec::new();
        let mut stack: Vec<usize> = Vec::new();
        let mut roots = 0;
        loop {
            let start = reader.buffer_position() as usize;
            let (namespace, event) = reader
                .read_resolved_event_into(&mut buffer)
                .map_err(|error| bibliography_error(format!("invalid source XML: {error}")))?;
            let namespace = namespace_value(namespace)?;
            let event = event.into_owned();
            let end = reader.buffer_position() as usize;
            match event {
                Event::Start(element) | Event::Empty(element) => {
                    let empty = xml.get(end.saturating_sub(2)..end) == Some(b"/>");
                    let name = std::str::from_utf8(element.name().as_ref())
                        .map_err(|_| bibliography_error("element name is not UTF-8"))?
                        .to_owned();
                    let local = std::str::from_utf8(element.local_name().as_ref())
                        .map_err(|_| bibliography_error("element local name is not UTF-8"))?
                        .to_owned();
                    let mut attributes = Vec::new();
                    for attribute in element.attributes() {
                        let attribute =
                            attribute.map_err(|error| bibliography_error(error.to_string()))?;
                        if attribute.key.as_ref() == b"xmlns"
                            || attribute.key.as_ref().starts_with(b"xmlns:")
                        {
                            continue;
                        }
                        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
                        let namespace = namespace_value(namespace)?;
                        let local = std::str::from_utf8(local.as_ref())
                            .map_err(|_| bibliography_error("attribute name is not UTF-8"))?
                            .to_owned();
                        let value = attribute
                            .decoded_and_normalized_value(
                                XmlVersion::Implicit1_0,
                                element.decoder(),
                            )
                            .map_err(|error| bibliography_error(error.to_string()))?
                            .into_owned();
                        if attributes
                            .iter()
                            .any(|(ns, key, _)| ns == &namespace && key == &local)
                        {
                            return Err(bibliography_error("duplicate expanded attribute"));
                        }
                        attributes.push((namespace, local, value));
                    }
                    let index = nodes.len();
                    if let Some(&parent) = stack.last() {
                        nodes[parent].children.push(index);
                    } else {
                        roots += 1;
                        if roots != 1 {
                            return Err(bibliography_error("multiple XML roots"));
                        }
                    }
                    nodes.push(SourceXmlNode {
                        namespace,
                        local,
                        name,
                        full: start..end,
                        content: end..end,
                        attributes,
                        children: Vec::new(),
                        text: String::new(),
                        opaque_scalar: false,
                        empty,
                    });
                    if !empty {
                        stack.push(index);
                    }
                }
                Event::End(_) => {
                    let index = stack
                        .pop()
                        .ok_or_else(|| bibliography_error("unmatched XML end"))?;
                    nodes[index].content.end = start;
                    nodes[index].full.end = end;
                }
                Event::Text(text) => {
                    let value = text
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(|error| bibliography_error(error.to_string()))?;
                    if let Some(&index) = stack.last() {
                        nodes[index].text.push_str(&value);
                    } else if !value.trim().is_empty() {
                        return Err(bibliography_error("text outside XML root"));
                    }
                }
                Event::CData(text) => {
                    let index = *stack
                        .last()
                        .ok_or_else(|| bibliography_error("CDATA outside XML root"))?;
                    nodes[index].opaque_scalar = true;
                    nodes[index].text.push_str(
                        &text
                            .decode()
                            .map_err(|error| bibliography_error(error.to_string()))?,
                    );
                }
                Event::GeneralRef(reference) => {
                    let index = *stack
                        .last()
                        .ok_or_else(|| bibliography_error("reference outside XML root"))?;
                    let value = reference
                        .decode()
                        .map_err(|error| bibliography_error(error.to_string()))?;
                    let spelling = format!("&{value};");
                    let decoded = quick_xml::escape::unescape(&spelling)
                        .map_err(|error| bibliography_error(error.to_string()))?;
                    nodes[index].text.push_str(&decoded);
                }
                Event::Comment(_) | Event::PI(_) => {
                    if let Some(&index) = stack.last() {
                        nodes[index].opaque_scalar = true;
                    }
                }
                Event::DocType(_) => return Err(bibliography_error("DOCTYPE is unsupported")),
                Event::Decl(_) if start == 0 && roots == 0 => {}
                Event::Decl(_) => return Err(bibliography_error("misplaced XML declaration")),
                Event::Eof => break,
            }
            buffer.clear();
        }
        if roots != 1 || !stack.is_empty() {
            return Err(bibliography_error("incomplete XML root"));
        }
        Ok(Self { nodes })
    }

    fn bibliography_children(&self, index: usize, name: &str) -> Vec<usize> {
        self.nodes[index]
            .children
            .iter()
            .copied()
            .filter(|&child| {
                self.nodes[child].namespace == BIB_NS && self.nodes[child].local == name
            })
            .collect()
    }

    fn scalar(&self, index: usize) -> Result<&str> {
        let node = &self.nodes[index];
        if !node.children.is_empty() {
            return Err(bibliography_error("mixed source scalar content"));
        }
        Ok(&node.text)
    }

    fn optional_scalar(&self, index: usize, name: &str) -> Result<Option<String>> {
        let children = self.bibliography_children(index, name);
        match children.as_slice() {
            [] => Ok(None),
            [child] => Ok(Some(self.scalar(*child)?.to_owned())),
            _ => Err(bibliography_error(format!("ambiguous repeated {name}"))),
        }
    }

    fn source_info(&self, index: usize) -> Result<BibliographySourceInfo> {
        let mut diagnostics = Vec::new();
        let mut identity_scalar = |name| match self.optional_scalar(index, name) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error.to_string());
                None
            }
        };
        let tag = identity_scalar("Tag").unwrap_or_default();
        let guid = identity_scalar("Guid");
        let source_type = identity_scalar("SourceType");
        let supported = if diagnostics.is_empty() {
            match self.supported_source(index, &tag, guid.as_deref(), source_type.as_deref(), true)
            {
                Ok(source) => source,
                Err(error) => {
                    diagnostics.push(error.to_string());
                    None
                }
            }
        } else {
            None
        };
        Ok(BibliographySourceInfo {
            tag,
            guid,
            source_type,
            supported,
            diagnostics,
        })
    }

    fn supported_source(
        &self,
        index: usize,
        tag: &str,
        guid: Option<&str>,
        kind: Option<&str>,
        project_locale: bool,
    ) -> Result<Option<BibliographySource>> {
        let Some(kind) = kind.and_then(BibliographySourceKind::from_xml_name) else {
            return Ok(None);
        };
        let Some(guid) = guid.filter(|value| !value.trim().is_empty()) else {
            return Ok(None);
        };
        if tag.trim().is_empty() {
            return Ok(None);
        }
        let locale = if project_locale {
            self.optional_scalar(index, "LCID")?
                .map(|value| source_locale_number(&value))
                .transpose()?
        } else {
            None
        };
        let mut contributors = Vec::new();
        let mut properties = Vec::new();
        for &child in &self.nodes[index].children {
            let node = &self.nodes[child];
            if node.namespace != BIB_NS {
                continue;
            }
            if let Some(field) = BibliographySourceField::from_xml_name(&node.local) {
                properties.push(BibliographyProperty {
                    field,
                    value: self.scalar(child)?.to_owned(),
                });
            } else if node.local == "Author" {
                for &role_node in &node.children {
                    let role = &self.nodes[role_node];
                    if role.namespace != BIB_NS {
                        continue;
                    }
                    let Some(role_kind) = BibliographyContributorRole::from_xml_name(&role.local)
                    else {
                        continue;
                    };
                    // Empty optional Author/Performer containers remain raw, not invented empty People.
                    let known = role
                        .children
                        .iter()
                        .copied()
                        .filter(|&child| self.nodes[child].namespace == BIB_NS)
                        .collect::<Vec<_>>();
                    if known.is_empty()
                        && matches!(
                            role_kind,
                            BibliographyContributorRole::Author
                                | BibliographyContributorRole::Performer
                        )
                    {
                        continue;
                    }
                    let [value_node] = known.as_slice() else {
                        return Err(bibliography_error("invalid contributor choice"));
                    };
                    let value_node = *value_node;
                    let value = match self.nodes[value_node].local.as_str() {
                        "Corporate"
                            if matches!(
                                role_kind,
                                BibliographyContributorRole::Author
                                    | BibliographyContributorRole::Performer
                            ) =>
                        {
                            BibliographyAuthor::Corporate(self.scalar(value_node)?.to_owned())
                        }
                        "NameList" => {
                            let people = self.bibliography_children(value_node, "Person");
                            if people.is_empty() {
                                return Err(bibliography_error("NameList requires Person"));
                            }
                            let mut result = Vec::new();
                            for person in people {
                                let mut result_person = BibliographyPerson {
                                    first: Vec::new(),
                                    middle: Vec::new(),
                                    last: Vec::new(),
                                };
                                let mut particle = 0;
                                for &component in &self.nodes[person].children {
                                    let node = &self.nodes[component];
                                    if node.namespace != BIB_NS {
                                        continue;
                                    }
                                    let next = match node.local.as_str() {
                                        "Last" => 0,
                                        "First" => 1,
                                        "Middle" => 2,
                                        _ => {
                                            return Err(bibliography_error(
                                                "unknown standard Person member",
                                            ));
                                        }
                                    };
                                    if next < particle {
                                        return Err(bibliography_error("Person child order"));
                                    }
                                    particle = next;
                                    let destination = match next {
                                        0 => &mut result_person.last,
                                        1 => &mut result_person.first,
                                        _ => &mut result_person.middle,
                                    };
                                    destination.push(self.scalar(component)?.to_owned());
                                }
                                result.push(result_person);
                            }
                            BibliographyAuthor::People(result)
                        }
                        _ => return Err(bibliography_error("invalid contributor value")),
                    };
                    contributors.push(BibliographyContributor {
                        role: role_kind,
                        value,
                    });
                }
            }
        }
        Ok(Some(BibliographySource {
            tag: tag.to_owned(),
            guid: guid.to_owned(),
            kind,
            locale,
            contributors,
            properties,
        }))
    }
}

pub(crate) fn bibliography_paragraph_property_ranges(
    xml: &[u8],
) -> Result<BTreeMap<usize, Range<usize>>> {
    const WORD: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let parsed = SourceXml::parse(xml)?;
    let mut ranges = BTreeMap::new();
    for paragraph in parsed
        .nodes
        .iter()
        .filter(|node| node.namespace == WORD && node.local == "p")
    {
        let properties = paragraph
            .children
            .iter()
            .map(|&child| &parsed.nodes[child])
            .filter(|node| node.namespace == WORD && node.local == "pPr")
            .collect::<Vec<_>>();
        match properties.as_slice() {
            [] => {}
            [properties] => {
                ranges.insert(paragraph.full.start, properties.full.clone());
            }
            _ => return Err(bibliography_error("ambiguous paragraph property ownership")),
        }
    }
    Ok(ranges)
}

pub(crate) fn bibliography_end_paragraph_properties(
    xml: &[u8],
    bindings: &BTreeMap<String, String>,
) -> Result<Vec<u8>> {
    const WORD: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let mut prefix = String::from("<rdocx-bibliography-properties");
    for (name, value) in bindings {
        if name == "xml" {
            continue;
        }
        let attribute = if name.is_empty() {
            "xmlns".into()
        } else {
            format!("xmlns:{name}")
        };
        prefix.push_str(&format!(
            " {attribute}=\"{}\"",
            quick_xml::escape::escape(value)
        ));
    }
    prefix.push('>');
    let mut scoped = prefix.as_bytes().to_vec();
    scoped.extend_from_slice(xml);
    scoped.extend_from_slice(b"</rdocx-bibliography-properties>");
    let parsed = SourceXml::parse(&scoped)?;
    let [root] = parsed.nodes[0].children.as_slice() else {
        return Err(bibliography_error(
            "ambiguous preserved paragraph properties",
        ));
    };
    let properties = &parsed.nodes[*root];
    if properties.namespace != WORD || properties.local != "pPr" {
        return Err(bibliography_error(
            "unsupported paragraph property namespace",
        ));
    }
    let styles = properties
        .children
        .iter()
        .map(|&child| &parsed.nodes[child])
        .filter(|node| node.namespace == WORD && node.local == "pStyle")
        .collect::<Vec<_>>();
    if styles.len() > 1 {
        return Err(bibliography_error("ambiguous preserved paragraph style"));
    }
    if let [style] = styles.as_slice()
        && style.empty
        && style.children.is_empty()
        && !style.opaque_scalar
        && style.attributes.as_slice() == [(WORD.into(), "val".into(), "Normal".into())]
    {
        let range = style
            .full
            .start
            .checked_sub(prefix.len())
            .zip(style.full.end.checked_sub(prefix.len()))
            .filter(|(start, end)| start <= end && *end <= xml.len())
            .ok_or_else(|| {
                bibliography_error("preserved style escaped its paragraph properties")
            })?;
        return apply_source_edits(xml, vec![(range.0..range.1, Vec::new())]);
    }
    Ok(xml.to_vec())
}

// Reset only recognized formatting particles. Namespace-qualified opaque XML remains raw.
pub(crate) fn bibliography_first_paragraph_properties(
    original: &[u8],
    generated: &[u8],
    bindings: &BTreeMap<String, String>,
) -> Result<Vec<u8>> {
    const WORD: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    const P: &[&str] = &[
        "pStyle",
        "keepNext",
        "keepLines",
        "pageBreakBefore",
        "framePr",
        "widowControl",
        "numPr",
        "suppressLineNumbers",
        "pBdr",
        "shd",
        "tabs",
        "suppressAutoHyphens",
        "kinsoku",
        "wordWrap",
        "overflowPunct",
        "topLinePunct",
        "autoSpaceDE",
        "autoSpaceDN",
        "bidi",
        "adjustRightInd",
        "snapToGrid",
        "spacing",
        "ind",
        "contextualSpacing",
        "mirrorIndents",
        "suppressOverlap",
        "jc",
        "textDirection",
        "textAlignment",
        "textboxTightWrap",
        "outlineLvl",
        "divId",
        "cnfStyle",
        "rPr",
        "sectPr",
        "pPrChange",
    ];
    const R: &[&str] = &[
        "rStyle",
        "rFonts",
        "b",
        "bCs",
        "i",
        "iCs",
        "caps",
        "smallCaps",
        "strike",
        "dstrike",
        "outline",
        "shadow",
        "emboss",
        "imprint",
        "noProof",
        "snapToGrid",
        "vanish",
        "webHidden",
        "color",
        "spacing",
        "w",
        "kern",
        "position",
        "sz",
        "szCs",
        "highlight",
        "u",
        "effect",
        "bdr",
        "shd",
        "fitText",
        "vertAlign",
        "rtl",
        "cs",
        "em",
        "lang",
        "eastAsianLayout",
        "specVanish",
        "oMath",
        "ins",
        "del",
        "rPrChange",
    ];
    let mut prefix = String::from("<rdocx-bibliography-properties");
    for (name, value) in bindings {
        if name == "xml" {
            continue;
        }
        let attribute = if name.is_empty() {
            "xmlns".into()
        } else {
            format!("xmlns:{name}")
        };
        prefix.push_str(&format!(
            " {attribute}=\"{}\"",
            quick_xml::escape::escape(value)
        ));
    }
    prefix.push('>');
    let mut scoped = prefix.as_bytes().to_vec();
    scoped.extend_from_slice(original);
    scoped.extend_from_slice(generated);
    scoped.extend_from_slice(b"</rdocx-bibliography-properties>");
    let parsed = SourceXml::parse(&scoped)?;
    let [old, new] = parsed.nodes[0].children.as_slice() else {
        return Err(bibliography_error("ambiguous first paragraph properties"));
    };
    if [&parsed.nodes[*old], &parsed.nodes[*new]]
        .iter()
        .any(|node| node.namespace != WORD || node.local != "pPr")
    {
        return Err(bibliography_error(
            "unsupported first paragraph property namespace",
        ));
    }
    fn owned(node: &SourceXmlNode, order: &[&str]) -> bool {
        node.namespace == WORD
            && order.contains(&node.local.as_str())
            && !matches!(
                node.local.as_str(),
                "sectPr" | "pPrChange" | "ins" | "del" | "rPrChange"
            )
    }
    fn merge(
        parsed: &SourceXml,
        xml: &[u8],
        old: usize,
        new: usize,
        order: &[&str],
    ) -> Result<Vec<u8>> {
        let old = &parsed.nodes[old];
        let new = &parsed.nodes[new];
        // History and section boundaries are caller metadata rather than formatting.
        fn validate(parsed: &SourceXml, old: &SourceXmlNode, order: &[&str]) -> Result<()> {
            let mut seen = BTreeSet::new();
            let mut previous = 0;
            for &child in &old.children {
                let node = &parsed.nodes[child];
                if owned(node, order) {
                    let rank = order.iter().position(|name| *name == node.local).unwrap();
                    if !seen.insert(node.local.as_str()) || rank < previous {
                        return Err(bibliography_error(
                            "ambiguous first paragraph formatting particles",
                        ));
                    }
                    previous = rank;
                    if node.local != "rPr"
                        && (node.attributes.iter().any(|(ns, local, _)| {
                            let attributes: &[&str] = match node.local.as_str() {
                                "numPr" | "pBdr" | "tabs" => &[],
                                "ind" => &[
                                    "left",
                                    "leftChars",
                                    "right",
                                    "rightChars",
                                    "start",
                                    "startChars",
                                    "end",
                                    "endChars",
                                    "hanging",
                                    "hangingChars",
                                    "firstLine",
                                    "firstLineChars",
                                ],
                                "spacing" if order == P => &[
                                    "before",
                                    "beforeLines",
                                    "beforeAutospacing",
                                    "after",
                                    "afterLines",
                                    "afterAutospacing",
                                    "line",
                                    "lineRule",
                                ],
                                "rFonts" => &[
                                    "ascii",
                                    "hAnsi",
                                    "eastAsia",
                                    "cs",
                                    "asciiTheme",
                                    "hAnsiTheme",
                                    "eastAsiaTheme",
                                    "cstheme",
                                    "hint",
                                ],
                                "color" => &["val", "themeColor", "themeTint", "themeShade"],
                                "lang" => &["val", "eastAsia", "bidi"],
                                "u" => &["val", "color", "themeColor", "themeTint", "themeShade"],
                                "shd" => &[
                                    "val",
                                    "color",
                                    "fill",
                                    "themeColor",
                                    "themeTint",
                                    "themeShade",
                                    "themeFill",
                                    "themeFillTint",
                                    "themeFillShade",
                                ],
                                "bdr" => &[
                                    "val",
                                    "color",
                                    "themeColor",
                                    "themeTint",
                                    "themeShade",
                                    "sz",
                                    "space",
                                    "shadow",
                                    "frame",
                                ],
                                "fitText" => &["val", "id"],
                                "eastAsianLayout" => {
                                    &["id", "combine", "combineBrackets", "vert", "vertCompress"]
                                }
                                "framePr" => &[
                                    "dropCap",
                                    "lines",
                                    "w",
                                    "h",
                                    "vSpace",
                                    "hSpace",
                                    "wrap",
                                    "hAnchor",
                                    "vAnchor",
                                    "x",
                                    "xAlign",
                                    "y",
                                    "yAlign",
                                    "hRule",
                                    "anchorLock",
                                ],
                                "cnfStyle" => &[
                                    "val",
                                    "firstRow",
                                    "lastRow",
                                    "firstColumn",
                                    "lastColumn",
                                    "oddVBand",
                                    "evenVBand",
                                    "oddHBand",
                                    "evenHBand",
                                    "firstRowFirstColumn",
                                    "firstRowLastColumn",
                                    "lastRowFirstColumn",
                                    "lastRowLastColumn",
                                ],
                                _ => &["val"],
                            };
                            ns != WORD || !attributes.contains(&local.as_str())
                        }) || !node.children.is_empty()
                            || node.opaque_scalar
                            || !node.text.trim().is_empty())
                    {
                        return Err(bibliography_error(
                            "opaque metadata in replaced formatting particle",
                        ));
                    }
                    if node.local == "rPr" {
                        validate(parsed, node, R)?;
                    }
                }
            }
            Ok(())
        }
        validate(parsed, old, order)?;
        if old.empty {
            let mut result = xml[old.full.start..old.full.end - 2].to_vec();
            result.push(b'>');
            for &child in &new.children {
                result.extend_from_slice(&generated_particle(
                    &xml[parsed.nodes[child].full.clone()],
                )?);
            }
            result.extend_from_slice(format!("</{}>", old.name).as_bytes());
            return Ok(result);
        }
        let mut edits = Vec::new();
        let mut inserted = BTreeSet::new();
        for &child in &old.children {
            let node = &parsed.nodes[child];
            if !owned(node, order) {
                continue;
            }
            let counterpart = new.children.iter().copied().find(|&n| {
                parsed.nodes[n].namespace == WORD && parsed.nodes[n].local == node.local
            });
            let replacement = if let Some(n) = counterpart {
                inserted.insert(n);
                if node.local == "rPr" {
                    merge(parsed, xml, child, n, R)?
                } else {
                    generated_particle(&xml[parsed.nodes[n].full.clone()])?
                }
            } else {
                if node.local == "rPr"
                    && (!node.attributes.is_empty()
                        || node.opaque_scalar
                        || !node.text.trim().is_empty()
                        || node.children.iter().any(|&child| {
                            let child = &parsed.nodes[child];
                            child.namespace != WORD
                                || !R.contains(&child.local.as_str())
                                || matches!(child.local.as_str(), "ins" | "del" | "rPrChange")
                        }))
                {
                    return Err(bibliography_error(
                        "removal would discard paragraph run metadata",
                    ));
                }
                Vec::new()
            };
            edits.push((
                node.full.start - old.full.start..node.full.end - old.full.start,
                replacement,
            ));
        }
        for &child in &new.children {
            if inserted.contains(&child) {
                continue;
            }
            let node = &parsed.nodes[child];
            let rank = order
                .iter()
                .position(|name| *name == node.local)
                .ok_or_else(|| bibliography_error("unexpected generated formatting particle"))?;
            let position = old
                .children
                .iter()
                .map(|&n| &parsed.nodes[n])
                .find(|n| {
                    n.namespace == WORD
                        && order
                            .iter()
                            .position(|name| *name == n.local)
                            .is_some_and(|r| r > rank)
                })
                .map_or(old.content.end, |n| n.full.start);
            edits.push((
                position - old.full.start..position - old.full.start,
                generated_particle(&xml[node.full.clone()])?,
            ));
        }
        apply_source_edits(&xml[old.full.clone()], edits)
    }
    fn generated_particle(xml: &[u8]) -> Result<Vec<u8>> {
        crate::field::xml_fragment_with_namespaces(
            xml,
            &BTreeMap::from([("w".into(), WORD.into())]),
            "generated bibliography formatting",
        )
    }
    merge(&parsed, &scoped, *old, *new, P)
}

struct BibliographyCollection {
    part: String,
    xml: Vec<u8>,
    parsed: SourceXml,
    sources: Vec<(usize, BibliographySourceInfo)>,
}

fn internal_target(
    package: &OpcPackage,
    owner: &str,
    relationship: &oxml_opc::Relationship,
) -> Result<String> {
    if relationship
        .target_mode
        .as_deref()
        .is_some_and(|mode| mode != "Internal")
        || !crate::embedded::relationship_target_is_normalized_pack_uri(&relationship.target)
    {
        return Err(bibliography_error("invalid internal custom XML edge"));
    }
    let target = OpcPackage::resolve_rel_target(owner, &relationship.target);
    if !package.contains_part(&target) {
        return Err(bibliography_error("dangling custom XML edge"));
    }
    Ok(target)
}

fn bibliography_collection(
    package: &OpcPackage,
    owner: &str,
) -> Result<Option<BibliographyCollection>> {
    let Some(relationships) = package.get_part_rels(owner) else {
        return Ok(None);
    };
    let mut result = None;
    let mut seen_parts = HashSet::new();
    let mut seen_properties = HashSet::new();
    let mut seen_store_ids = HashSet::new();
    for relationship in relationships
        .items
        .iter()
        .filter(|edge| edge.rel_type == CUSTOM_XML_REL)
    {
        let part = internal_target(package, owner, relationship)?;
        if !seen_parts.insert(part.to_ascii_lowercase()) {
            return Err(bibliography_error("duplicate custom XML collection edge"));
        }
        let xml = package
            .get_part(&part)
            .ok_or_else(|| bibliography_error("missing custom XML part"))?;
        let parsed = SourceXml::parse(xml)?;
        let root = &parsed.nodes[0];
        let is_bibliography = root.namespace == BIB_NS && root.local == "Sources";
        if is_bibliography && result.is_some() {
            return Err(bibliography_error("ambiguous source collection"));
        }
        if is_bibliography
            && package.content_types.content_type_for(&part) != Some("application/xml")
        {
            return Err(bibliography_error("wrong bibliography source content type"));
        }
        let properties = package
            .get_part_rels(&part)
            .map(|rels| {
                rels.items
                    .iter()
                    .filter(|edge| edge.rel_type == CUSTOM_XML_PROPS_REL)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if properties.len() > 1 {
            return Err(bibliography_error("ambiguous source properties edges"));
        }
        if let Some(properties) = properties.first() {
            let properties_part = internal_target(package, &part, properties)?;
            if !seen_properties.insert(properties_part.to_ascii_lowercase()) {
                return Err(bibliography_error("shared custom XML properties owner"));
            }
            if package.content_types.content_type_for(&properties_part)
                != Some(CUSTOM_XML_PROPS_TYPE)
            {
                return Err(bibliography_error(
                    "wrong custom XML properties content type",
                ));
            }
            let properties_xml = package
                .get_part(&properties_part)
                .ok_or_else(|| bibliography_error("missing properties part"))?;
            SourceXml::parse(properties_xml)?;
            let store_id = crate::content_control::parse_store_item_id(properties_xml)?
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| bibliography_error("missing custom XML store identity"))?;
            if !seen_store_ids.insert(crate::content_control::normalize_store_item_id(&store_id)) {
                return Err(bibliography_error(
                    "conflicting custom XML store identities",
                ));
            }
        }
        if !is_bibliography {
            continue;
        }
        let mut sources = Vec::new();
        for index in parsed.bibliography_children(0, "Source") {
            sources.push((index, parsed.source_info(index)?));
        }
        result = Some(BibliographyCollection {
            part,
            xml: xml.to_vec(),
            parsed,
            sources,
        });
    }
    Ok(result)
}

impl Document {
    /// Inspect the qualified source collection without changing producer XML.
    pub fn bibliography_sources(&self) -> Result<Vec<BibliographySourceInfo>> {
        Ok(bibliography_collection(&self.package, &self.doc_part_name)?
            .map(|collection| {
                collection
                    .sources
                    .into_iter()
                    .map(|(_, source)| source)
                    .collect()
            })
            .unwrap_or_default())
    }
}

fn validate_source(source: &BibliographySource) -> Result<()> {
    if let Some(locale) = source.locale {
        validate_locale_number(locale)?;
    }
    if source.tag.trim().is_empty() {
        return Err(bibliography_error("empty source tag"));
    }
    let guid = source
        .guid
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or(&source.guid);
    if guid.len() != 36
        || !guid.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
    {
        return Err(bibliography_error("invalid source GUID"));
    }
    for value in [&source.tag, &source.guid]
        .into_iter()
        .chain(source.properties.iter().map(|property| &property.value))
    {
        oxml_core::xml::reject_non_xml_characters("bibliography source", value)
            .map_err(|error| bibliography_error(error.to_string()))?;
    }
    for contributor in &source.contributors {
        match &contributor.value {
            BibliographyAuthor::Corporate(value) => {
                if !matches!(
                    contributor.role,
                    BibliographyContributorRole::Author | BibliographyContributorRole::Performer
                ) {
                    return Err(bibliography_error(
                        "corporate value is illegal for contributor role",
                    ));
                }
                oxml_core::xml::reject_non_xml_characters("corporate contributor", value)
                    .map_err(|error| bibliography_error(error.to_string()))?;
            }
            BibliographyAuthor::People(people) => {
                if people.is_empty() {
                    return Err(bibliography_error("NameList requires at least one Person"));
                }
                for person in people {
                    for value in person
                        .last
                        .iter()
                        .chain(&person.first)
                        .chain(&person.middle)
                    {
                        oxml_core::xml::reject_non_xml_characters("bibliography name", value)
                            .map_err(|error| bibliography_error(error.to_string()))?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn source_text_xml(value: &str) -> String {
    quick_xml::escape::escape(value).replace('\r', "&#13;")
}

fn source_element(name: &str, text: &str) -> String {
    format!(
        "<b:{name} xmlns:b=\"{BIB_NS}\">{}</b:{name}>",
        source_text_xml(text)
    )
}

fn contributor_value_xml(value: &BibliographyAuthor) -> String {
    match value {
        BibliographyAuthor::Corporate(value) => source_element("Corporate", value),
        BibliographyAuthor::People(people) => {
            let mut xml = format!("<b:NameList xmlns:b=\"{BIB_NS}\">");
            for person in people {
                xml.push_str("<b:Person>");
                for (name, values) in [
                    ("Last", &person.last),
                    ("First", &person.first),
                    ("Middle", &person.middle),
                ] {
                    for value in values {
                        xml.push_str(&source_element(name, value));
                    }
                }
                xml.push_str("</b:Person>");
            }
            xml.push_str("</b:NameList>");
            xml
        }
    }
}

fn contributor_xml(contributor: &BibliographyContributor) -> String {
    let name = contributor.role.xml_name();
    format!(
        "<b:{name} xmlns:b=\"{BIB_NS}\">{}</b:{name}>",
        contributor_value_xml(&contributor.value)
    )
}

fn authored_source_xml(source: &BibliographySource) -> String {
    let mut xml = format!("<b:Source xmlns:b=\"{BIB_NS}\">");
    xml.push_str(&source_element("Tag", &source.tag));
    xml.push_str(&source_element("SourceType", source.kind.xml_name()));
    xml.push_str(&source_element("Guid", &source.guid));
    if let Some(locale) = source.locale {
        xml.push_str(&source_element("LCID", &locale.to_string()));
    }
    if !source.contributors.is_empty() {
        xml.push_str("<b:Author>");
        for contributor in &source.contributors {
            xml.push_str(&contributor_xml(contributor));
        }
        xml.push_str("</b:Author>");
    }
    for property in &source.properties {
        xml.push_str(&source_element(property.field.xml_name(), &property.value));
    }
    xml.push_str("</b:Source>");
    xml
}

fn unique_collection_identities(collection: &BibliographyCollection) -> Result<()> {
    let mut tags = HashSet::new();
    let mut guids = HashSet::new();
    for (index, source) in &collection.sources {
        for name in ["Tag", "Guid", "SourceType", "LCID"] {
            if collection.parsed.bibliography_children(*index, name).len() > 1 {
                return Err(bibliography_error(format!(
                    "ambiguous repeated source identity {name}"
                )));
            }
        }
        if !source.tag.is_empty() && !tags.insert(source.tag.as_str()) {
            return Err(bibliography_error("duplicate source tag"));
        }
        if let Some(guid) = &source.guid
            && !guids.insert(crate::content_control::normalize_store_item_id(guid))
        {
            return Err(bibliography_error("duplicate source GUID"));
        }
    }
    Ok(())
}

fn apply_source_edits(xml: &[u8], mut edits: Vec<(Range<usize>, Vec<u8>)>) -> Result<Vec<u8>> {
    edits.sort_by_key(|(range, _)| (range.start, range.end));
    let mut output = Vec::new();
    let mut cursor = 0;
    for (range, replacement) in edits {
        if range.start < cursor || range.end < range.start || range.end > xml.len() {
            return Err(bibliography_error("overlapping or stale source edits"));
        }
        output.extend_from_slice(&xml[cursor..range.start]);
        output.extend_from_slice(&replacement);
        cursor = range.end;
    }
    output.extend_from_slice(&xml[cursor..]);
    Ok(output)
}

fn append_to_source_node(
    xml: &[u8],
    node: &SourceXmlNode,
    content: &[u8],
) -> Result<(Range<usize>, Vec<u8>)> {
    if node.empty {
        let mut replacement = xml[node.full.start..node.full.end - 2].to_vec();
        replacement.push(b'>');
        replacement.extend_from_slice(content);
        replacement.extend_from_slice(format!("</{}>", node.name).as_bytes());
        Ok((node.full.clone(), replacement))
    } else {
        Ok((node.content.end..node.content.end, content.to_vec()))
    }
}

fn source_scalar_edit(
    xml: &[u8],
    parsed: &SourceXml,
    index: usize,
    value: Option<&str>,
) -> Result<Option<(Range<usize>, Vec<u8>)>> {
    let node = &parsed.nodes[index];
    if value == Some(parsed.scalar(index)?) {
        return Ok(None);
    }
    if node.opaque_scalar || !node.children.is_empty() {
        return Err(bibliography_error(
            "replacement would discard producer scalar metadata",
        ));
    }
    if let Some(value) = value {
        if node.empty {
            return append_to_source_node(xml, node, source_text_xml(value).as_bytes()).map(Some);
        }
        Ok(Some((
            node.content.clone(),
            source_text_xml(value).as_bytes().to_vec(),
        )))
    } else {
        if !node.attributes.is_empty() {
            return Err(bibliography_error(
                "removal would discard producer attributes",
            ));
        }
        Ok(Some((node.full.clone(), Vec::new())))
    }
}

fn source_name_edits(node: &SourceXmlNode, local: &str) -> Vec<(Range<usize>, Vec<u8>)> {
    let name = node
        .name
        .rsplit_once(':')
        .map(|(prefix, _)| format!("{prefix}:{local}"))
        .unwrap_or_else(|| local.to_owned())
        .into_bytes();
    let mut edits = vec![(
        node.full.start + 1..node.full.start + 1 + node.name.len(),
        name.clone(),
    )];
    if !node.empty {
        edits.push((
            node.content.end + 2..node.content.end + 2 + node.name.len(),
            name,
        ));
    }
    edits
}

fn replace_source_xml(
    collection: &BibliographyCollection,
    index: usize,
    source: &BibliographySource,
) -> Result<Vec<u8>> {
    let parsed = &collection.parsed;
    let previous = parsed
        .source_info(index)?
        .supported
        .ok_or_else(|| bibliography_error("source has unsupported or ambiguous modeled data"))?;
    if previous == *source {
        return Ok(collection.xml.clone());
    }
    let mut edits = Vec::new();
    let mut appended = String::new();
    for (name, desired) in [
        ("SourceType", Some(source.kind.xml_name().to_owned())),
        ("LCID", source.locale.map(|value| value.to_string())),
    ] {
        let children = parsed.bibliography_children(index, name);
        match children.as_slice() {
            [] => {
                if let Some(value) = desired {
                    appended.push_str(&source_element(name, &value));
                }
            }
            [child] => {
                if let Some(edit) =
                    source_scalar_edit(&collection.xml, parsed, *child, desired.as_deref())?
                {
                    edits.push(edit);
                }
            }
            _ => return Err(bibliography_error("ambiguous source identity")),
        }
    }
    let mut property_index = 0;
    for &child in &parsed.nodes[index].children {
        let node = &parsed.nodes[child];
        if node.namespace != BIB_NS {
            continue;
        }
        let Some(field) = BibliographySourceField::from_xml_name(&node.local) else {
            continue;
        };
        let replacement = source.properties.get(property_index);
        property_index += 1;
        if let Some(property) = replacement
            && property.field != field
        {
            if !node.attributes.is_empty() || node.opaque_scalar || !node.children.is_empty() {
                return Err(bibliography_error(
                    "property identity change would discard producer metadata",
                ));
            }
            if node.empty {
                let mut updated = node
                    .name
                    .rsplit_once(':')
                    .map(|(prefix, _)| format!("{prefix}:{}", property.field.xml_name()))
                    .unwrap_or_else(|| property.field.xml_name().to_owned());
                updated = format!(
                    "<{updated}{}>{}</{updated}>",
                    std::str::from_utf8(
                        &collection.xml[node.full.start + node.name.len() + 1..node.full.end - 2]
                    )
                    .map_err(|_| bibliography_error("invalid property XML"))?,
                    source_text_xml(&property.value)
                );
                edits.push((node.full.clone(), updated.into_bytes()));
                continue;
            }
            edits.extend(source_name_edits(node, property.field.xml_name()));
        }
        if let Some(edit) = source_scalar_edit(
            &collection.xml,
            parsed,
            child,
            replacement.map(|property| property.value.as_str()),
        )? {
            edits.push(edit);
        }
    }
    for property in source.properties.iter().skip(property_index) {
        appended.push_str(&source_element(property.field.xml_name(), &property.value));
    }
    if source.contributors != previous.contributors {
        // Contributor containers can be interleaved with scalar properties.
        // Patch an owned role in place and retain each surrounding raw Author.
        let mut previous_index = 0;
        let mut replacement_index = 0;
        for author in parsed.bibliography_children(index, "Author") {
            for &role in &parsed.nodes[author].children {
                let node = &parsed.nodes[role];
                if node.namespace != BIB_NS {
                    continue;
                }
                let Some(role_kind) = BibliographyContributorRole::from_xml_name(&node.local)
                else {
                    continue;
                };
                if node.children.is_empty()
                    && matches!(
                        role_kind,
                        BibliographyContributorRole::Author
                            | BibliographyContributorRole::Performer
                    )
                {
                    continue;
                }
                let old = previous
                    .contributors
                    .get(previous_index)
                    .ok_or_else(|| bibliography_error("stale contributor inventory"))?;
                previous_index += 1;
                let new = source.contributors.get(replacement_index);
                replacement_index += usize::from(new.is_some());
                if new == Some(old) {
                    continue;
                }
                let mut pending = vec![role];
                while let Some(index) = pending.pop() {
                    let node = &parsed.nodes[index];
                    if node.namespace != BIB_NS || !node.attributes.is_empty() || node.opaque_scalar
                    {
                        return Err(bibliography_error(
                            "replacement would discard contributor producer metadata",
                        ));
                    }
                    pending.extend(node.children.iter().copied());
                }
                if let Some(new) = new {
                    if new.role != role_kind {
                        edits.extend(source_name_edits(node, new.role.xml_name()));
                    }
                    edits.push((
                        node.content.clone(),
                        contributor_value_xml(&new.value).into_bytes(),
                    ));
                } else {
                    edits.push((node.full.clone(), Vec::new()));
                }
            }
        }
        if replacement_index < source.contributors.len() {
            appended.push_str(&format!("<b:Author xmlns:b=\"{BIB_NS}\">"));
            for contributor in &source.contributors[replacement_index..] {
                appended.push_str(&contributor_xml(contributor));
            }
            appended.push_str("</b:Author>");
        }
    }
    if !appended.is_empty() {
        edits.push(append_to_source_node(
            &collection.xml,
            &parsed.nodes[index],
            appended.as_bytes(),
        )?);
    }
    apply_source_edits(&collection.xml, edits)
}

fn create_collection(candidate: &mut Document) -> Result<String> {
    candidate
        .identifiers
        .observe_package_graph(&candidate.package)?;
    let part = candidate
        .identifiers
        .reserve_preferred_part_name("/customXml/item1.xml")?;
    candidate
        .package
        .content_types
        .add_override(&part, "application/xml");
    let owner = candidate.doc_part_name.clone();
    candidate.add_relative_internal_relationship_checked(&owner, CUSTOM_XML_REL, &part)?;
    let properties = candidate
        .identifiers
        .reserve_preferred_part_name("/customXml/itemProps1.xml")?;
    let mut occupied = HashSet::new();
    for (name, relationships) in &candidate.package.part_rels {
        for relationship in relationships
            .items
            .iter()
            .filter(|edge| edge.rel_type == CUSTOM_XML_PROPS_REL)
        {
            let target = internal_target(&candidate.package, name, relationship)?;
            let xml = candidate
                .package
                .get_part(&target)
                .ok_or_else(|| bibliography_error("missing custom XML properties"))?;
            if let Some(identity) = crate::content_control::parse_store_item_id(xml)? {
                occupied.insert(crate::content_control::normalize_store_item_id(&identity));
            }
        }
    }
    let identity = (0u64..=0xffffffffffff)
        .map(|index| format!("{{00000000-0000-4000-8000-{index:012X}}}"))
        .find(|value| !occupied.contains(&crate::content_control::normalize_store_item_id(value)))
        .ok_or_else(|| bibliography_error("custom XML store identity space exhausted"))?;
    candidate.package.set_part(&properties, format!("<ds:datastoreItem xmlns:ds=\"http://schemas.openxmlformats.org/officeDocument/2006/customXml\" ds:itemID=\"{identity}\"><ds:schemaRefs/></ds:datastoreItem>").into_bytes());
    candidate
        .package
        .content_types
        .add_override(&properties, CUSTOM_XML_PROPS_TYPE);
    candidate.add_relative_internal_relationship_checked(
        &part,
        CUSTOM_XML_PROPS_REL,
        &properties,
    )?;
    candidate.package.set_part(
        &part,
        format!("<b:Sources xmlns:b=\"{BIB_NS}\"></b:Sources>").into_bytes(),
    );
    Ok(part)
}

fn publish_source_collection(document: &mut Document, candidate: Document) -> Result<()> {
    let collection = bibliography_collection(&candidate.package, &candidate.doc_part_name)?
        .ok_or_else(|| bibliography_error("staged collection disappeared"))?;
    unique_collection_identities(&collection)?;
    let mut candidate = candidate.reopen_prepared_staged()?;
    candidate.invalidate_layout();
    document.commit_staged_mutation(candidate);
    Ok(())
}

impl Document {
    /// Append one validated source while retaining all existing producer records.
    pub fn add_bibliography_source(&mut self, source: &BibliographySource) -> Result<()> {
        validate_source(source)?;
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        if bibliography_collection(&candidate.package, &candidate.doc_part_name)?.is_none() {
            create_collection(&mut candidate)?;
        }
        let collection = bibliography_collection(&candidate.package, &candidate.doc_part_name)?
            .ok_or_else(|| bibliography_error("source collection creation failed"))?;
        unique_collection_identities(&collection)?;
        if collection.sources.iter().any(|(_, existing)| {
            existing.tag == source.tag
                || existing.guid.as_ref().is_some_and(|guid| {
                    crate::content_control::normalize_store_item_id(guid)
                        == crate::content_control::normalize_store_item_id(&source.guid)
                })
        }) {
            return Err(bibliography_error("source identity already exists"));
        }
        let edit = append_to_source_node(
            &collection.xml,
            &collection.parsed.nodes[0],
            authored_source_xml(source).as_bytes(),
        )?;
        candidate.package.set_part(
            &collection.part,
            apply_source_edits(&collection.xml, vec![edit])?,
        );
        publish_source_collection(self, candidate)
    }

    /// Replace modeled members of one source, preserving its GUID and raw metadata.
    pub fn replace_bibliography_source(&mut self, source: &BibliographySource) -> Result<()> {
        validate_source(source)?;
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        let collection = bibliography_collection(&candidate.package, &candidate.doc_part_name)?
            .ok_or_else(|| bibliography_error("source collection does not exist"))?;
        unique_collection_identities(&collection)?;
        let (index, existing) = collection
            .sources
            .iter()
            .find(|(_, existing)| existing.tag == source.tag)
            .ok_or_else(|| bibliography_error("source tag does not exist"))?;
        if existing.guid.as_deref() != Some(source.guid.as_str()) {
            return Err(bibliography_error("source GUID cannot change"));
        }
        let xml = replace_source_xml(&collection, *index, source)?;
        if xml == collection.xml {
            return Ok(());
        }
        candidate.package.set_part(&collection.part, xml);
        publish_source_collection(self, candidate)
    }

    /// Remove an unreferenced source without deleting its collection or properties.
    pub fn remove_bibliography_source(&mut self, tag: &str) -> Result<()> {
        if tag.trim().is_empty() {
            return Err(bibliography_error("empty source tag"));
        }
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        let collection = bibliography_collection(&candidate.package, &candidate.doc_part_name)?
            .ok_or_else(|| bibliography_error("source collection does not exist"))?;
        unique_collection_identities(&collection)?;
        let (index, _) = collection
            .sources
            .iter()
            .find(|(_, source)| source.tag == tag)
            .ok_or_else(|| bibliography_error("source tag does not exist"))?;
        for instruction in crate::field::bibliography_reference_instructions(&candidate)? {
            if !instruction.quotes_are_balanced() {
                return Err(bibliography_error("ambiguous citation reference"));
            }
            let mut tags = Vec::new();
            for argument in &instruction.arguments {
                let rdocx_oxml::text::FieldArgument::Text(value) = argument else {
                    return Err(bibliography_error("nested citation reference"));
                };
                tags.push(value.as_str());
            }
            for switch in instruction
                .switches
                .iter()
                .filter(|switch| switch.name == "m")
            {
                let Some(rdocx_oxml::text::FieldArgument::Text(value)) = &switch.argument else {
                    return Err(bibliography_error("ambiguous citation member"));
                };
                tags.push(value.as_str());
            }
            if tags.is_empty() {
                return Err(bibliography_error("citation has no source identity"));
            }
            if tags.contains(&tag) {
                return Err(bibliography_error("source is referenced by a citation"));
            }
        }
        candidate.package.set_part(
            &collection.part,
            apply_source_edits(
                &collection.xml,
                vec![(collection.parsed.nodes[*index].full.clone(), Vec::new())],
            )?,
        );
        publish_source_collection(self, candidate)
    }
}

// MS-OE376 2.1.732, Part 4 7.6.2.39. These are lexical LCID encodings,
// not an observed accepted-locale manifest or a formatter fallback table.
const SOURCE_LOCALE_ENCODINGS: &[(u32, &str)] = &[
    (1025, "ar-SA"),
    (1026, "bg-BG"),
    (1027, "ca-ES"),
    (1028, "zh-TW"),
    (1029, "cs-CZ"),
    (1030, "da-DK"),
    (1031, "de-DE"),
    (1032, "el-GR"),
    (1033, "en-US"),
    (1034, "es-ES"),
    (1035, "fi-FI"),
    (1036, "fr-FR"),
    (1037, "he-IL"),
    (1038, "hu-HU"),
    (1039, "is-IS"),
    (1040, "it-IT"),
    (1041, "ja-JP"),
    (1042, "ko-KR"),
    (1043, "nl-NL"),
    (1044, "nb-NO"),
    (1045, "pl-PL"),
    (1046, "pt-BR"),
    (1047, "rm-CH"),
    (1048, "ro-RO"),
    (1049, "ru-RU"),
    (1050, "hr-HR"),
    (1051, "sk-SK"),
    (1052, "sq-AL"),
    (1053, "sv-SE"),
    (1054, "th-TH"),
    (1055, "tr-TR"),
    (1056, "ur-PK"),
    (1057, "id-ID"),
    (1058, "uk-UA"),
    (1059, "be-BY"),
    (1060, "sl-SI"),
    (1061, "et-EE"),
    (1062, "lv-LV"),
    (1063, "lt-LT"),
    (1064, "tg-Cyrl-TJ"),
    (1065, "fa-IR"),
    (1066, "vi-VN"),
    (1067, "hy-AM"),
    (1068, "az-Latn-AZ"),
    (1069, "eu-ES"),
    (1070, "wen-DE"),
    (1071, "mk-MK"),
    (1072, "st-ZA"),
    (1073, "ts-ZA"),
    (1074, "tn-ZA"),
    (1075, "ven-ZA"),
    (1076, "xh-ZA"),
    (1077, "zu-ZA"),
    (1078, "af-ZA"),
    (1079, "ka-GE"),
    (1080, "fo-FO"),
    (1081, "hi-IN"),
    (1082, "mt-MT"),
    (1083, "se-NO"),
    (1084, "gd-GB"),
    (1085, "yi"),
    (1086, "ms-MY"),
    (1087, "kk-KZ"),
    (1088, "ky-KG"),
    (1089, "sw-KE"),
    (1090, "tk-TM"),
    (1091, "uz-Latn-UZ"),
    (1092, "tt-RU"),
    (1093, "bn-IN"),
    (1094, "pa-IN"),
    (1095, "gu-IN"),
    (1096, "or-IN"),
    (1097, "ta-IN"),
    (1098, "te-IN"),
    (1099, "kn-IN"),
    (1100, "ml-IN"),
    (1101, "as-IN"),
    (1102, "mr-IN"),
    (1103, "sa-IN"),
    (1104, "mn-MN"),
    (1105, "bo-CN"),
    (1106, "cy-GB"),
    (1107, "km-KH"),
    (1108, "lo-LA"),
    (1109, "my-MM"),
    (1110, "gl-ES"),
    (1111, "kok-IN"),
    (1112, "mni"),
    (1113, "sd-IN"),
    (1114, "syr-SY"),
    (1115, "si-LK"),
    (1116, "chr-US"),
    (1117, "iu-Cans-CA"),
    (1118, "am-ET"),
    (1119, "tmz"),
    (1120, "ks-Arab-IN"),
    (1121, "ne-NP"),
    (1122, "fy-NL"),
    (1123, "ps-AF"),
    (1124, "fil-PH"),
    (1125, "dv-MV"),
    (1126, "bin-NG"),
    (1127, "fuv-NG"),
    (1128, "ha-Latn-NG"),
    (1129, "ibb-NG"),
    (1130, "yo-NG"),
    (1131, "quz-BO"),
    (1132, "nso-ZA"),
    (1136, "ig-NG"),
    (1137, "kr-NG"),
    (1138, "gaz-ET"),
    (1139, "ti-ER"),
    (1140, "gn-PY"),
    (1141, "haw-US"),
    (1142, "la"),
    (1143, "so-SO"),
    (1144, "ii-CN"),
    (1145, "pap-AN"),
    (1152, "ug-Arab-CN"),
    (1153, "mi-NZ"),
    (2049, "ar-IQ"),
    (2052, "zh-CN"),
    (2055, "de-CH"),
    (2057, "en-GB"),
    (2058, "es-MX"),
    (2060, "fr-BE"),
    (2064, "it-CH"),
    (2067, "nl-BE"),
    (2068, "nn-NO"),
    (2070, "pt-PT"),
    (2072, "ro-MD"),
    (2073, "ru-MD"),
    (2074, "sr-Latn-CS"),
    (2077, "sv-FI"),
    (2080, "ur-IN"),
    (2092, "az-Cyrl-AZ"),
    (2108, "ga-IE"),
    (2110, "ms-BN"),
    (2115, "uz-Cyrl-UZ"),
    (2117, "bn-BD"),
    (2118, "pa-PK"),
    (2128, "mn-Mong-CN"),
    (2129, "bo-BT"),
    (2137, "sd-PK"),
    (2143, "tzm-Latn-DZ"),
    (2144, "ks-Deva-IN"),
    (2145, "ne-IN"),
    (2155, "quz-EC"),
    (2163, "ti-ET"),
    (3073, "ar-EG"),
    (3076, "zh-HK"),
    (3079, "de-AT"),
    (3081, "en-AU"),
    (3082, "es-ES"),
    (3084, "fr-CA"),
    (3098, "sr-Cyrl-CS"),
    (3179, "quz-PE"),
    (4097, "ar-LY"),
    (4100, "zh-SG"),
    (4103, "de-LU"),
    (4105, "en-CA"),
    (4106, "es-GT"),
    (4108, "fr-CH"),
    (4122, "hr-BA"),
    (5121, "ar-DZ"),
    (5124, "zh-MO"),
    (5127, "de-LI"),
    (5129, "en-NZ"),
    (5130, "es-CR"),
    (5132, "fr-LU"),
    (5146, "bs-Latn-BA"),
    (6145, "ar-MO"),
    (6153, "en-IE"),
    (6154, "es-PA"),
    (6156, "fr-MC"),
    (7169, "ar-TN"),
    (7177, "en-ZA"),
    (7178, "es-DO"),
    (7180, "fr-029"),
    (8193, "ar-OM"),
    (8201, "en-JM"),
    (8202, "es-VE"),
    (8204, "fr-RE"),
    (9217, "ar-YE"),
    (9225, "en-029"),
    (9226, "es-CO"),
    (9228, "fr-CG"),
    (10241, "ar-SY"),
    (10249, "en-BZ"),
    (10250, "es-PE"),
    (10252, "fr-SN"),
    (11265, "ar-JO"),
    (11273, "en-TT"),
    (11274, "es-AR"),
    (11276, "fr-CM"),
    (12289, "ar-LB"),
    (12297, "en-ZW"),
    (12298, "es-EC"),
    (12300, "fr-CI"),
    (13313, "ar-KW"),
    (13321, "en-PH"),
    (13322, "es-CL"),
    (13324, "fr-ML"),
    (14337, "ar-AE"),
    (14345, "en-ID"),
    (14346, "es-UY"),
    (14348, "fr-MA"),
    (15361, "ar-BH"),
    (15369, "en-HK"),
    (15370, "es-PY"),
    (15372, "fr-HT"),
    (16385, "ar-QA"),
    (16393, "en-IN"),
    (16394, "es-BO"),
    (17417, "en-MY"),
    (17418, "es-SV"),
    (18441, "en-SG"),
    (18442, "es-HN"),
    (19466, "es-NI"),
    (20490, "es-PR"),
    (21514, "es-US"),
    (58378, "es-419"),
    (58380, "fr-015"),
];

fn source_locale_number(value: &str) -> Result<u32> {
    // Native1034 normalizes to this distinct traditional producer spelling.
    if value.eq_ignore_ascii_case("es-ES_tradnl") {
        return Ok(1034);
    }
    // Distinct native filter membership establishes the modern operational identity.
    if value.eq_ignore_ascii_case("es-ES") {
        return Ok(3082);
    }
    if let Ok(number) = value.parse::<u32>() {
        validate_locale_number(number)?;
        return Ok(number);
    }
    let mut matches = SOURCE_LOCALE_ENCODINGS
        .iter()
        .filter(|(_, name)| name.eq_ignore_ascii_case(value));
    let Some(&(number, _)) = matches.next() else {
        return Err(bibliography_error("unknown source LCID encoding"));
    };
    if matches.next().is_some() {
        return Err(bibliography_error(format!(
            "non-lossless locale projection: {value} identifies multiple documented LCIDs"
        )));
    }
    Ok(number)
}

fn validate_locale_number(number: u32) -> Result<()> {
    if number == 0
        || SOURCE_LOCALE_ENCODINGS
            .iter()
            .any(|&(lcid, _)| lcid == number)
    {
        Ok(())
    } else {
        Err(bibliography_error(format!(
            "undefined bibliography LCID {number}"
        )))
    }
}

impl BibliographyStyle {
    fn metadata(self) -> (&'static str, &'static str) {
        match self {
            Self::ApaSixthEdition => ("APASixthEditionOfficeOnline.xsl", "APA"),
            Self::Chicago => ("CHICAGO.XSL", "Chicago"),
            Self::Gb7714 => ("GB.XSL", "GB7714"),
            Self::GostName => ("GostName.XSL", "GOST - Name Sort"),
            Self::GostTitle => ("GostTitle.XSL", "GOST - Title Sort"),
            Self::HarvardAnglia => ("HarvardAnglia2008OfficeOnline.xsl", "Harvard - Anglia"),
            Self::Ieee => ("IEEE2006OfficeOnline.xsl", "IEEE"),
            Self::Iso690AuthorDate => ("ISO690.XSL", "ISO 690 - First Element and Date"),
            Self::Iso690Numeric => ("ISO690Nmerical.XSL", "ISO 690 - Numerical Reference"),
            Self::MlaSeventhEdition => ("MLASeventhEditionOfficeOnline.xsl", "MLA"),
            Self::Sist02 => ("SIST02.XSL", "SIST02"),
            Self::Turabian => ("TURABIAN.XSL", "Turabian"),
        }
    }

    fn from_metadata(path: Option<&str>, key: Option<&str>) -> Option<Self> {
        let styles = [
            Self::ApaSixthEdition,
            Self::Chicago,
            Self::Gb7714,
            Self::GostName,
            Self::GostTitle,
            Self::HarvardAnglia,
            Self::Ieee,
            Self::Iso690AuthorDate,
            Self::Iso690Numeric,
            Self::MlaSeventhEdition,
            Self::Sist02,
            Self::Turabian,
        ];
        styles.into_iter().find(|style| {
            let (filename, name) = style.metadata();
            path.is_none_or(|path| {
                path.trim_start_matches(['\\', '/'])
                    .eq_ignore_ascii_case(filename)
            }) && key.is_none_or(|key| key == name)
        })
    }
}

fn style_info(collection: &BibliographyCollection) -> Option<BibliographyStyleInfo> {
    let attributes = &collection.parsed.nodes[0].attributes;
    let get = |name| {
        attributes
            .iter()
            .find(|(namespace, local, _)| namespace.is_empty() && local == name)
            .map(|(_, _, value)| value.clone())
    };
    let style_path = get("SelectedStyle");
    let style_key = get("StyleName");
    if style_path.is_none() && style_key.is_none() {
        return None;
    }
    let supported_style =
        BibliographyStyle::from_metadata(style_path.as_deref(), style_key.as_deref());
    Some(BibliographyStyleInfo {
        style_key,
        style_path,
        locale: None,
        supported_style,
    })
}

fn attach_style_metadata(candidate: &mut Document, style: BibliographyStyle) -> Result<()> {
    if bibliography_collection(&candidate.package, &candidate.doc_part_name)?.is_none() {
        create_collection(candidate)?;
    }
    let collection = bibliography_collection(&candidate.package, &candidate.doc_part_name)?
        .ok_or_else(|| bibliography_error("source collection creation failed"))?;
    unique_collection_identities(&collection)?;
    let root = &collection.parsed.nodes[0];
    let start_end = if root.empty {
        root.full.end
    } else {
        root.content.start
    };
    let start = &collection.xml[root.full.start..start_end];
    let (filename, key) = style.metadata();
    let path = format!("\\{filename}");
    let mut edits = Vec::new();
    let mut appended = String::new();
    for (name, value) in [("SelectedStyle", path.as_str()), ("StyleName", key)] {
        if let Some((_, _, previous)) = root
            .attributes
            .iter()
            .find(|(namespace, local, _)| namespace.is_empty() && local == name)
        {
            if previous == value {
                continue;
            }
            let (begin, end) = crate::field::attribute_value_span(start, name.as_bytes())
                .ok_or_else(|| bibliography_error("style attribute source range disappeared"))?;
            edits.push((
                root.full.start + begin..root.full.start + end,
                quick_xml::escape::escape(value).as_bytes().to_vec(),
            ));
        } else {
            appended.push_str(&format!(" {name}=\"{}\"", quick_xml::escape::escape(value)));
        }
    }
    if !appended.is_empty() {
        let insertion = start_end - if root.empty { 2 } else { 1 };
        edits.push((insertion..insertion, appended.into_bytes()));
    }
    if !edits.is_empty() {
        candidate.package.set_part(
            &collection.part,
            apply_source_edits(&collection.xml, edits)?,
        );
    }
    Ok(())
}

fn option_switch(name: &str, value: Option<String>) -> rdocx_oxml::text::FieldSwitch {
    rdocx_oxml::text::FieldSwitch {
        name: name.to_owned(),
        argument: value.map(rdocx_oxml::text::FieldArgument::Text),
    }
}

fn bibliography_option_switches(
    options: &BibliographyOptions,
) -> Result<Vec<rdocx_oxml::text::FieldSwitch>> {
    for locale in [options.locale, options.locale_filter]
        .into_iter()
        .flatten()
    {
        validate_locale_number(locale)?;
    }
    let mut switches = Vec::new();
    if let Some(locale) = options.locale {
        switches.push(option_switch("l", Some(locale.to_string())));
    }
    if let Some(locale) = options.locale_filter {
        switches.push(option_switch("f", Some(locale.to_string())));
    }
    for tag in &options.tags {
        if tag.trim().is_empty() {
            return Err(bibliography_error("empty selected source tag"));
        }
        switches.push(option_switch("m", Some(tag.clone())));
    }
    rdocx_oxml::text::FieldInstruction::new("BIBLIOGRAPHY", Vec::new(), switches.clone())?;
    Ok(switches)
}

impl Document {
    /// Inspect saved standard style metadata without inventing a root locale.
    pub fn bibliography_style(&self) -> Result<Option<BibliographyStyleInfo>> {
        Ok(bibliography_collection(&self.package, &self.doc_part_name)?
            .as_ref()
            .and_then(style_info))
    }

    /// Atomically set document style and existing bibliography selectors.
    /// Citation instructions and every stored result cache retain their bytes.
    pub fn set_bibliography_options(&mut self, options: &BibliographyOptions) -> Result<()> {
        let switches = bibliography_option_switches(options)?;
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        let before = candidate.package.clone();
        attach_style_metadata(&mut candidate, options.style)?;
        crate::field::patch_bibliography_instruction_options(&mut candidate, &switches)?;
        if candidate.package.parts == before.parts
            && candidate.package.part_rels.len() == before.part_rels.len()
            && candidate
                .package
                .part_rels
                .iter()
                .all(|(part, relationships)| {
                    before
                        .part_rels
                        .get(part)
                        .is_some_and(|previous| previous.items == relationships.items)
                })
            && candidate.package.content_types == before.content_types
        {
            return Ok(());
        }
        publish_source_collection(self, candidate)
    }

    /// Insert a complex citation at one checked paragraph run boundary.
    pub fn insert_citation(
        &mut self,
        position: &crate::StoryRunPosition,
        options: &CitationOptions,
    ) -> Result<()> {
        if let Some(locale) = options.locale {
            validate_locale_number(locale)?;
        }
        let Some(first) = options.sources.first() else {
            return Err(bibliography_error("citation requires at least one source"));
        };
        let mut switches = Vec::new();
        if let Some(locale) = options.locale {
            switches.push(option_switch("l", Some(locale.to_string())));
        }
        for (index, source) in options.sources.iter().enumerate() {
            if source.tag.trim().is_empty() {
                return Err(bibliography_error("empty citation source tag"));
            }
            if index != 0 {
                switches.push(option_switch("m", Some(source.tag.clone())));
            }
            for (name, value) in [
                ("p", &source.pages),
                ("v", &source.volume),
                ("f", &source.prefix),
                ("s", &source.suffix),
            ] {
                if let Some(value) = value {
                    switches.push(option_switch(name, Some(value.clone())));
                }
            }
            for (name, enabled) in [
                ("n", source.suppress_author),
                ("y", source.suppress_year),
                ("t", source.suppress_title),
            ] {
                if enabled {
                    switches.push(option_switch(name, None));
                }
            }
        }
        let instruction = rdocx_oxml::text::FieldInstruction::new(
            "CITATION",
            vec![rdocx_oxml::text::FieldArgument::Text(first.tag.clone())],
            switches,
        )?;
        self.insert_checked_story_field(position, instruction)
    }

    /// Insert one bibliography owner with its own selectors and document style.
    /// Existing bibliography instructions and caches remain unchanged.
    pub fn insert_bibliography(
        &mut self,
        position: &crate::ContentLocation,
        options: &BibliographyOptions,
    ) -> Result<()> {
        let switches = bibliography_option_switches(options)?;
        let instruction =
            rdocx_oxml::text::FieldInstruction::new("BIBLIOGRAPHY", Vec::new(), switches)?;
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        attach_style_metadata(&mut candidate, options.style)?;
        let mut paragraph = rdocx_oxml::text::CT_P::new();
        let mut run = rdocx_oxml::text::CT_R::new("");
        run.content = vec![rdocx_oxml::text::RunContent::Field(
            rdocx_oxml::text::Field::from_instruction(
                instruction,
                rdocx_oxml::text::FieldForm::Complex,
                Vec::new(),
            )?,
        )];
        paragraph.runs.push(run);
        candidate.insert_content(position, crate::ContentFragment::paragraph(paragraph)?)?;
        candidate.prepare_staged_package()?;
        publish_source_collection(self, candidate)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BibliographyFormattingLocale {
    Numeric(u32),
    Lexical(String),
}

fn formatting_locale(
    source: Option<&str>,
    field: Option<&str>,
    default: Option<u32>,
) -> Result<BibliographyFormattingLocale> {
    let numeric = |number| {
        if number == 0 {
            default
                .map(BibliographyFormattingLocale::Numeric)
                .ok_or_else(|| {
                    bibliography_error("bibliography field requires application default locale")
                })
        } else {
            validate_locale_number(number)?;
            Ok(BibliographyFormattingLocale::Numeric(number))
        }
    };
    let resolve = |value: &str| {
        if let Ok(number) = value.parse::<u32>() {
            return numeric(number);
        }
        if value.eq_ignore_ascii_case("es-ES_tradnl") {
            return numeric(1034);
        }
        if value.eq_ignore_ascii_case("es-ES") {
            return numeric(3082);
        }
        let matches = SOURCE_LOCALE_ENCODINGS
            .iter()
            .filter(|(_, name)| name.eq_ignore_ascii_case(value))
            .map(|&(number, _)| number)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Err(bibliography_error("unknown bibliography LCID encoding")),
            [number] => numeric(*number),
            _ => Ok(BibliographyFormattingLocale::Lexical(value.to_owned())),
        }
    };
    // Keep raw source0 distinct in XML. It supplies no formatting override.
    if let Some(source) = source.filter(|value| value.parse::<u32>() != Ok(0)) {
        return resolve(source);
    }
    match field {
        Some(value) => resolve(value),
        None => numeric(0),
    }
}

// Measured numeric CITATION operand/member boundary. Generic formatting has its own phase.
pub(crate) const NUMERIC_CITATION_SWITCH_LIMIT: usize = 10;

pub(crate) struct BibliographyUpdateState {
    collection: Option<BibliographyCollection>,
    style: Option<BibliographyStyle>,
    default_locale: Option<u32>,
    reference_numbers: BTreeMap<String, u32>,
    label_metrics: Option<(oxml_layout::FontManager, oxml_layout::FontId, f64)>,
}

impl BibliographyUpdateState {
    fn new(document: &Document, default_locale: Option<u32>) -> Result<Self> {
        if let Some(locale) = default_locale {
            if locale == 0 {
                return Err(bibliography_error(
                    "application default locale must be nonzero",
                ));
            }
            validate_locale_number(locale)?;
        }
        let collection = bibliography_collection(&document.package, &document.doc_part_name)?;
        if let Some(collection) = &collection {
            unique_collection_identities(collection)?;
        }
        let style = collection
            .as_ref()
            .and_then(style_info)
            .and_then(|info| info.supported_style);
        let mut reference_numbers = BTreeMap::new();
        if matches!(
            style,
            Some(BibliographyStyle::Ieee | BibliographyStyle::Iso690Numeric)
        ) {
            let mut tags = crate::field::bibliography_citation_encounter_tags(document)?;
            if let Some(collection) = &collection {
                tags.extend(
                    collection
                        .sources
                        .iter()
                        .map(|(_, identity)| identity.tag.clone()),
                );
            }
            for tag in tags {
                if reference_numbers.contains_key(&tag)
                    || !collection.as_ref().is_some_and(|collection| {
                        collection
                            .sources
                            .iter()
                            .any(|(_, source)| source.tag == tag)
                    })
                {
                    continue;
                }
                let number = u32::try_from(reference_numbers.len())
                    .ok()
                    .and_then(|number| number.checked_add(1))
                    .ok_or_else(|| {
                        bibliography_error("citation reference count exceeds the numeric domain")
                    })?;
                reference_numbers.insert(tag, number);
            }
        }
        let label_metrics = if style == Some(BibliographyStyle::Ieee) {
            let style_id = if document.styles.get_by_id("Bibliography").is_some() {
                "Bibliography"
            } else {
                "Normal"
            };
            let effective =
                crate::style::resolve_run_properties(Some(style_id), None, &document.styles);
            let input = document.build_layout_input();
            let family = rdocx_layout::engine::resolve_font_family(
                &effective,
                input.theme.as_ref(),
                rdocx_layout::engine::WordFontSlot::Ascii,
            );
            let mut fonts = oxml_layout::FontManager::new_deterministic()?;
            fonts.load_additional_fonts(&input.fonts);
            let font = fonts.resolve_font_for_metrics(
                family.as_deref(),
                effective.bold.unwrap_or(false),
                effective.italic.unwrap_or(false),
            )?;
            Some((fonts, font, effective.sz.map_or(11.0, |size| size.to_pt())))
        } else {
            None
        };
        Ok(Self {
            collection,
            style,
            default_locale,
            reference_numbers,
            label_metrics,
        })
    }

    pub(crate) fn bibliography_blocks(
        &self,
        instruction: &rdocx_oxml::text::FieldInstruction,
        text_width: Option<i32>,
    ) -> Result<Option<Vec<rdocx_oxml::document::BodyContent>>> {
        use rdocx_oxml::text::FieldArgument;
        let Some(collection) = &self.collection else {
            return Ok(None);
        };
        let Some(style) = self.style else {
            return Ok(None);
        };
        if !instruction.quotes_are_balanced() || !instruction.arguments.is_empty() {
            return Err(bibliography_error("ambiguous bibliography instruction"));
        }
        let mut tags = Vec::new();
        let mut field_locale = None;
        let mut locale_filter = None;
        for switch in &instruction.switches {
            match switch.name.as_str() {
                "m" | "l" => {
                    let Some(FieldArgument::Text(value)) = &switch.argument else {
                        return Err(bibliography_error(
                            "bibliography operand is missing or nested",
                        ));
                    };
                    if switch.name == "m" {
                        tags.push(value.as_str());
                    } else {
                        field_locale = Some(value.as_str());
                    }
                }
                "f" => {
                    locale_filter = Some(match &switch.argument {
                        None => 0,
                        Some(FieldArgument::Text(value)) => source_locale_number(value)?,
                        Some(_) => {
                            return Err(bibliography_error("nested bibliography locale filter"));
                        }
                    });
                }
                _ => return Ok(None),
            }
        }
        if tags.len() > 1 {
            return Err(bibliography_error(
                "catalogued bibliography selection branch is still being implemented",
            ));
        }
        let mut entries = Vec::new();
        for (index, identity) in &collection.sources {
            if !tags.is_empty() && !tags.contains(&identity.tag.as_str()) {
                continue;
            }
            let raw_locale = collection.parsed.optional_scalar(*index, "LCID")?;
            if let Some(filter) = locale_filter {
                let source_locale = raw_locale
                    .as_deref()
                    .map(source_locale_number)
                    .transpose()?
                    .unwrap_or(0);
                if source_locale != filter {
                    continue;
                }
            }
            let locale =
                formatting_locale(raw_locale.as_deref(), field_locale, self.default_locale)?;
            let Some(source) = collection.parsed.supported_source(
                *index,
                &identity.tag,
                identity.guid.as_deref(),
                identity.source_type.as_deref(),
                false,
            )?
            else {
                return Ok(None);
            };
            if (style != BibliographyStyle::ApaSixthEdition
                && locale != BibliographyFormattingLocale::Numeric(1033))
                || (style != BibliographyStyle::ApaSixthEdition
                    && source.kind != BibliographySourceKind::Book)
                || (style != BibliographyStyle::ApaSixthEdition
                    && (source.contributors.iter().any(|contributor| {
                        contributor.role != BibliographyContributorRole::Author
                    }) || source.properties.iter().any(|property| {
                        !matches!(
                            property.field,
                            BibliographySourceField::Title
                                | BibliographySourceField::ShortTitle
                                | BibliographySourceField::Year
                                | BibliographySourceField::City
                                | BibliographySourceField::Publisher
                                | BibliographySourceField::ReferenceOrder
                        )
                    })))
            {
                return Err(bibliography_error(
                    "catalogued bibliography formatter branch is still being implemented",
                ));
            }

            let reference = self.reference_numbers.get(&source.tag).copied();
            let numeric = matches!(
                style,
                BibliographyStyle::Iso690Numeric | BibliographyStyle::Ieee
            );
            let order = if numeric {
                reference.ok_or_else(|| {
                    bibliography_error("missing effective bibliography reference number")
                })?
            } else {
                0
            };
            let key = if !numeric && tags.is_empty() && collection.sources.len() > 1 {
                bibliography_sort_key(&source, style, &locale)?
            } else {
                BibliographySortKey::default()
            };
            let rtl_paragraph_mark = style == BibliographyStyle::ApaSixthEdition
                && matches!(
                    locale,
                    BibliographyFormattingLocale::Numeric(
                        1025 | 1037
                            | 1056
                            | 1065
                            | 1123
                            | 2118
                            | 2137
                            | 1085
                            | 1114
                            | 1119
                            | 1120
                            | 1125
                            | 2080
                            | 1152
                            | 2049
                            | 3073
                            | 4097
                            | 5121
                            | 6145
                            | 7169
                            | 8193
                            | 9217
                            | 10241
                            | 11265
                            | 12289
                            | 13313
                            | 14337
                            | 15361
                            | 16385
                    )
                );
            let complex_paragraph_mark = style == BibliographyStyle::ApaSixthEdition
                && matches!(
                    locale,
                    BibliographyFormattingLocale::Numeric(
                        1081 | 1093
                            | 1094
                            | 1095
                            | 1096
                            | 1097
                            | 1098
                            | 1099
                            | 1100
                            | 1101
                            | 1102
                            | 1103
                            | 2144
                    )
                );
            let east_asia_paragraph_mark = style == BibliographyStyle::ApaSixthEdition
                && matches!(
                    locale,
                    BibliographyFormattingLocale::Numeric(
                        1028 | 1041 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124
                    )
                );
            entries.push((
                order,
                key,
                source_bibliography_paragraph(&source, style, reference, locale)?,
                (
                    rtl_paragraph_mark,
                    complex_paragraph_mark,
                    east_asia_paragraph_mark,
                ),
            ));
        }
        if entries.is_empty() {
            return Err(bibliography_error(
                "catalogued empty bibliography branch is still being implemented",
            ));
        }
        entries.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
        if style == BibliographyStyle::Ieee {
            let text_width = text_width.ok_or_else(|| {
                bibliography_error(
                    "catalogued bibliography story geometry branch is still being implemented",
                )
            })?;
            let (fonts, font, size) = self
                .label_metrics
                .as_ref()
                .ok_or_else(|| bibliography_error("bibliography label font metrics unavailable"))?;
            let mut label_width = 0;
            let mut rows = Vec::new();
            for (number, _, paragraph, _) in entries {
                let advance = fonts
                    .shape_text(*font, &format!("[{number}]"), *size)?
                    .width
                    * 20.0;
                // Native bibliography-specific reserve, measured independently of generic table layout.
                // Four font/digit axes and two body-width axes bind this sans-trailing-space rule.
                let width = (advance + 75.0).ceil();
                if !width.is_finite() || width < 0.0 || width > f64::from(i32::MAX) {
                    return Err(bibliography_error("bibliography label width overflow"));
                }
                label_width = label_width.max(width as i32);
                rows.push((number, paragraph));
            }
            return Ok(Some(ieee_bibliography_table(
                rows,
                label_width,
                text_width,
            )?));
        }
        Ok(Some(
            entries
                .into_iter()
                .enumerate()
                .map(
                    |(
                        index,
                        (
                            _,
                            _,
                            mut paragraph,
                            (rtl_paragraph_mark, complex_paragraph_mark, east_asia_paragraph_mark),
                        ),
                    )| {
                        if index != 0
                            && let Some(properties) = paragraph.properties.as_mut()
                            && let Some(run_properties) = properties.rpr.as_mut()
                        {
                            run_properties.sz = None;
                            run_properties.sz_cs = None;
                            if rtl_paragraph_mark
                                || complex_paragraph_mark
                                || east_asia_paragraph_mark
                            {
                                run_properties.font_hint = Some(
                                    if east_asia_paragraph_mark {
                                        "eastAsia"
                                    } else {
                                        "cs"
                                    }
                                    .into(),
                                );
                                run_properties.rtl = rtl_paragraph_mark.then_some(true);
                                run_properties.complex_script =
                                    complex_paragraph_mark.then_some(true);
                            }
                        }
                        rdocx_oxml::document::BodyContent::Paragraph(paragraph)
                    },
                )
                .collect(),
        ))
    }

    pub(crate) fn citation_runs(
        &self,
        instruction: &rdocx_oxml::text::FieldInstruction,
    ) -> Result<Option<Vec<rdocx_oxml::text::CT_R>>> {
        use rdocx_oxml::text::FieldArgument;
        let Some(collection) = &self.collection else {
            return Ok(None);
        };
        let Some(style) = self.style else {
            return Ok(None);
        };
        if !instruction.quotes_are_balanced() {
            return Err(bibliography_error("ambiguous citation instruction"));
        }
        let full_instruction = instruction;
        let numeric_instruction = if matches!(
            style,
            BibliographyStyle::Ieee | BibliographyStyle::Iso690Numeric
        ) {
            if instruction
                .switches
                .iter()
                .skip(NUMERIC_CITATION_SWITCH_LIMIT)
                .any(|switch| {
                    switch.name == "*"
                        && !matches!(&switch.argument,
                    Some(FieldArgument::Text(value)) if value.eq_ignore_ascii_case("Upper"))
                })
            {
                return Err(bibliography_error(
                    "catalogued late numeric general-format phase is still being implemented",
                ));
            }
            let mut effective = instruction.clone();
            effective.switches.truncate(NUMERIC_CITATION_SWITCH_LIMIT);
            Some(effective)
        } else {
            None
        };
        let instruction = numeric_instruction.as_ref().unwrap_or(instruction);
        let [FieldArgument::Text(first)] = instruction.arguments.as_slice() else {
            return Err(bibliography_error(
                "citation requires one initial source tag",
            ));
        };
        let member = |tag: String| CitationSourceOptions {
            tag,
            pages: None,
            volume: None,
            prefix: None,
            suffix: None,
            suppress_author: false,
            suppress_year: false,
            suppress_title: false,
        };
        let mut members = vec![member(first.clone())];
        let mut field_locale = None;
        for switch in &instruction.switches {
            match switch.name.as_str() {
                "n" | "y" | "t" if switch.argument.is_none() => {
                    let current = members.last_mut().expect("initial citation member exists");
                    match switch.name.as_str() {
                        "n" => current.suppress_author = true,
                        "y" => current.suppress_year = true,
                        _ => current.suppress_title = true,
                    }
                }
                "l" | "m" | "p" | "v" | "f" | "s" => {
                    let Some(FieldArgument::Text(value)) = &switch.argument else {
                        return Err(bibliography_error("citation operand is missing or nested"));
                    };
                    if switch.name == "m" {
                        members.push(member(value.clone()));
                    } else if switch.name == "l" {
                        field_locale = Some(value.as_str());
                    } else {
                        let current = members.last_mut().expect("initial citation member exists");
                        let operand = match switch.name.as_str() {
                            "p" => &mut current.pages,
                            "v" => &mut current.volume,
                            "f" => &mut current.prefix,
                            _ => &mut current.suffix,
                        };
                        // Numeric styles use the first scalar operand for each source member.
                        if operand.is_none()
                            || !matches!(
                                style,
                                BibliographyStyle::Ieee | BibliographyStyle::Iso690Numeric
                            )
                        {
                            *operand = Some(value.clone());
                        }
                    }
                }
                "n" | "y" | "t" => return Err(bibliography_error("citation flag has an operand")),
                "*" if matches!(
                    style,
                    BibliographyStyle::Ieee | BibliographyStyle::Iso690Numeric
                ) => {}
                _ => return Ok(None),
            }
        }
        let mut fragments = Vec::new();
        let mut numeric_profile = None;
        for member in &members {
            if member.tag.trim().is_empty() {
                return Err(bibliography_error("empty citation source identity"));
            }
            let Some((index, identity)) = collection
                .sources
                .iter()
                .find(|(_, source)| source.tag == member.tag)
            else {
                return Ok(None);
            };
            let raw_locale = collection.parsed.optional_scalar(*index, "LCID")?;
            let locale =
                formatting_locale(raw_locale.as_deref(), field_locale, self.default_locale)?;
            let Some(source) = collection.parsed.supported_source(
                *index,
                &identity.tag,
                identity.guid.as_deref(),
                identity.source_type.as_deref(),
                false,
            )?
            else {
                return Ok(None);
            };
            if matches!(
                style,
                BibliographyStyle::Ieee | BibliographyStyle::Iso690Numeric
            ) {
                let number = self.reference_numbers.get(&member.tag).ok_or_else(|| {
                    bibliography_error("citation encounter numbering lost its source identity")
                })?;
                let profile = if style == BibliographyStyle::Ieee {
                    Some((true, 0, true))
                } else {
                    iso_numeric_run_profile(&locale)
                }
                .ok_or_else(|| {
                    bibliography_error(
                        "catalogued numeric citation locale branch is still being implemented",
                    )
                })?;
                if numeric_profile.is_some_and(|previous| previous != profile) {
                    return Err(bibliography_error(
                        "catalogued mixed-locale numeric citation branch is still being implemented",
                    ));
                }
                numeric_profile = Some(profile);
                let mut fragment = number.to_string();
                if style == BibliographyStyle::Ieee {
                    if let Some(pages) = &member.pages {
                        if locale != BibliographyFormattingLocale::Numeric(1033)
                            && locale != BibliographyFormattingLocale::Numeric(1036)
                        {
                            return Err(bibliography_error(
                                "catalogued numeric page-label locale is still being implemented",
                            ));
                        }
                        let label = if pages.contains(['-', '–']) {
                            "pp."
                        } else {
                            "p."
                        };
                        fragment.push_str(&format!(", {label} {pages}"));
                    }
                } else if member.pages.is_some()
                    || member.volume.is_some()
                    || member.prefix.is_some()
                    || member.suffix.is_some()
                {
                    if locale != BibliographyFormattingLocale::Numeric(1033)
                        && locale != BibliographyFormattingLocale::Numeric(1036)
                    {
                        return Err(bibliography_error(
                            "catalogued numeric modifier locale is still being implemented",
                        ));
                    }
                    let pages = member.pages.as_deref().filter(|value| !value.is_empty());
                    let volume = member.volume.as_deref().filter(|value| !value.is_empty());
                    match (volume, pages) {
                        (Some(volume), Some(pages)) => {
                            fragment.push_str(&format!(" {volume}: {pages}"))
                        }
                        (Some(volume), None) => fragment.push_str(&format!(" vol. {volume}")),
                        (None, Some(pages)) => {
                            let label = if pages.contains(['-', '–']) {
                                "pp."
                            } else {
                                "p."
                            };
                            fragment.push_str(&format!(" {label} {pages}"));
                        }
                        (None, None) => {}
                    }
                    fragment = format!(
                        "{}{fragment}{}",
                        member.prefix.as_deref().unwrap_or_default(),
                        member.suffix.as_deref().unwrap_or_default()
                    );
                }
                fragments.push(fragment);
                continue;
            }
            // Additional style, locale and kind branches are implemented from their bound records.
            // An unfinished catalogued branch fails the transaction instead of retaining it as unsupported.
            if style == BibliographyStyle::ApaSixthEdition
                && source.kind == BibliographySourceKind::Patent
                && members.len() == 1
                && !first_source_property(&source, BibliographySourceField::CountryRegion)
                    .is_empty()
                && !first_source_property(&source, BibliographySourceField::PatentNumber).is_empty()
                && !first_source_property(&source, BibliographySourceField::Year).is_empty()
                && member.pages.is_none()
                && member.volume.is_none()
                && member.prefix.is_none()
                && member.suffix.is_none()
                && !member.suppress_author
                && !member.suppress_year
                && !member.suppress_title
                && let Some(grammar) = apa_patent_run_grammar(&locale)
            {
                return Ok(Some(source_citation_runs(
                    &source, None, grammar.0, grammar.1,
                )?));
            }
            if style == BibliographyStyle::ApaSixthEdition
                && source.kind == BibliographySourceKind::Case
                && members.len() == 1
                && !first_source_property(&source, BibliographySourceField::ShortTitle).is_empty()
                && !first_source_property(&source, BibliographySourceField::Year).is_empty()
                && member.pages.is_none()
                && member.volume.is_none()
                && member.prefix.is_none()
                && member.suffix.is_none()
                && !member.suppress_author
                && !member.suppress_year
                && !member.suppress_title
                && let Some(grammar) = apa_case_run_grammar(&locale)
            {
                return Ok(Some(source_citation_runs(
                    &source, None, grammar.0, grammar.1,
                )?));
            }
            if style == BibliographyStyle::Chicago
                && members.len() == 1
                && member.pages.is_none()
                && member.volume.is_none()
                && member.prefix.is_none()
                && member.suffix.is_none()
                && !member.suppress_author
                && !member.suppress_year
                && !member.suppress_title
                && !first_source_property(&source, BibliographySourceField::Year).is_empty()
                && let Some(creator) = single_citation_creator(&source, style)
                && creator.last.len() == 1
                && creator.first.len() <= 1
                && !creator.last[0].is_empty()
                && let Some(grammar) = chicago_run_grammar(&locale)
            {
                if has_equivalent_creator(collection, *index, creator, style)? {
                    return Err(bibliography_error(
                        "catalogued creator-context disambiguation is still being implemented",
                    ));
                }
                return Ok(Some(source_citation_runs(
                    &source,
                    Some(creator),
                    grammar.0,
                    grammar.1,
                )?));
            }
            if style == BibliographyStyle::ApaSixthEdition
                && !matches!(
                    source.kind,
                    BibliographySourceKind::Patent | BibliographySourceKind::Case
                )
                && members.len() == 1
                && member.pages.is_none()
                && member.volume.is_none()
                && member.prefix.is_none()
                && member.suffix.is_none()
                && !member.suppress_author
                && !member.suppress_year
                && !member.suppress_title
                && !first_source_property(&source, BibliographySourceField::ShortTitle).is_empty()
                && !first_source_property(&source, BibliographySourceField::Year).is_empty()
                && let Some(creator) = single_citation_creator(&source, style)
                && creator.last.len() == 1
                && creator.first.len() <= 1
                && !creator.last[0].is_empty()
                && !has_equivalent_creator(collection, *index, creator, style)?
                && let Some(grammar) = apa_creator_run_grammar(&locale)
            {
                return Ok(Some(source_citation_runs(
                    &source,
                    Some(creator),
                    grammar.0,
                    grammar.1,
                )?));
            }
            if matches!(
                style,
                BibliographyStyle::Iso690AuthorDate | BibliographyStyle::Sist02
            ) && members.len() == 1
                && member.pages.is_none()
                && member.volume.is_none()
                && member.prefix.is_none()
                && member.suffix.is_none()
                && !member.suppress_author
                && !member.suppress_year
                && !member.suppress_title
                && !first_source_property(&source, BibliographySourceField::Year).is_empty()
            {
                let title = matches!(
                    source.kind,
                    BibliographySourceKind::JournalArticle
                        | BibliographySourceKind::ConferenceProceedings
                );
                let creator = if title {
                    None
                } else {
                    single_citation_creator(&source, style)
                };
                if title
                    && first_source_property(&source, BibliographySourceField::Title).is_empty()
                {
                    return Err(bibliography_error(
                        "catalogued missing-title citation branch is still being implemented",
                    ));
                }
                if !title
                    && !creator.is_some_and(|person| {
                        person.last.len() == 1
                            && !person.last[0].is_empty()
                            && person.first.len() <= 1
                    })
                {
                    return Err(bibliography_error(
                        "catalogued contributor citation branch is still being implemented",
                    ));
                }
                if let Some(creator) = creator
                    && has_equivalent_creator(collection, *index, creator, style)?
                {
                    return Err(bibliography_error(
                        "catalogued creator-context disambiguation is still being implemented",
                    ));
                }
                let grammar = match style {
                    BibliographyStyle::Iso690AuthorDate => {
                        iso_author_date_run_grammar(&locale, title)
                    }
                    BibliographyStyle::Sist02 => sist_run_grammar(&locale, title),
                    _ => unreachable!(),
                };
                if let Some(grammar) = grammar {
                    return Ok(Some(source_citation_runs(
                        &source, creator, grammar.0, grammar.1,
                    )?));
                }
            }
            if matches!(
                style,
                BibliographyStyle::Gb7714
                    | BibliographyStyle::GostName
                    | BibliographyStyle::GostTitle
                    | BibliographyStyle::HarvardAnglia
                    | BibliographyStyle::MlaSeventhEdition
                    | BibliographyStyle::Turabian
            ) && members.len() == 1
                && member.pages.is_none()
                && member.volume.is_none()
                && member.prefix.is_none()
                && member.suffix.is_none()
                && !member.suppress_author
                && !member.suppress_year
                && !member.suppress_title
                && (style == BibliographyStyle::MlaSeventhEdition
                    || !first_source_property(&source, BibliographySourceField::Year).is_empty())
            {
                let title = matches!(
                    (style, source.kind),
                    (
                        BibliographyStyle::Gb7714,
                        BibliographySourceKind::JournalArticle
                            | BibliographySourceKind::ConferenceProceedings
                    ) | (
                        BibliographyStyle::HarvardAnglia,
                        BibliographySourceKind::Film | BibliographySourceKind::Case
                    ) | (
                        BibliographyStyle::MlaSeventhEdition,
                        BibliographySourceKind::Case
                    )
                );
                let creator = if title {
                    None
                } else {
                    single_citation_creator(&source, style)
                };
                if title
                    && first_source_property(
                        &source,
                        if style == BibliographyStyle::MlaSeventhEdition {
                            BibliographySourceField::ShortTitle
                        } else {
                            BibliographySourceField::Title
                        },
                    )
                    .is_empty()
                {
                    return Err(bibliography_error(
                        "catalogued missing-title citation branch is still being implemented",
                    ));
                }
                if !title
                    && !creator.is_some_and(|person| {
                        person.last.len() == 1
                            && !person.last[0].is_empty()
                            && person.first.len() <= 1
                    })
                {
                    return Err(bibliography_error(
                        "catalogued contributor citation branch is still being implemented",
                    ));
                }
                if let Some(creator) = creator
                    && has_equivalent_creator(collection, *index, creator, style)?
                {
                    return Err(bibliography_error(
                        "catalogued creator-context disambiguation is still being implemented",
                    ));
                }
                if let Some(grammar) = other_style_citation_run_grammar(style, &locale, title) {
                    return Ok(Some(source_citation_runs(
                        &source, creator, grammar.0, grammar.1,
                    )?));
                }
            }
            let patent_punctuation = apa_plain_patent_punctuation(&locale);
            let measured_plain_patent = source.kind == BibliographySourceKind::Patent
                && !first_source_property(&source, BibliographySourceField::CountryRegion)
                    .is_empty()
                && !first_source_property(&source, BibliographySourceField::PatentNumber)
                    .is_empty()
                && patent_punctuation.is_some()
                && member.pages.is_none()
                && member.volume.is_none();
            let measured_french_plain = locale == BibliographyFormattingLocale::Numeric(1036)
                && source.kind == BibliographySourceKind::Book
                && member.pages.as_deref().is_none_or(str::is_empty)
                && member.volume.as_deref().is_none_or(str::is_empty)
                && source.contributors.iter().filter(|contributor| contributor.role == BibliographyContributorRole::Author)
                    .all(|contributor| matches!(&contributor.value, BibliographyAuthor::People(people) if people.len() == 1));
            if style != BibliographyStyle::ApaSixthEdition
                || !(locale == BibliographyFormattingLocale::Numeric(1033)
                    || measured_french_plain
                    || measured_plain_patent)
            {
                return Err(bibliography_error(
                    "catalogued citation formatter branch is still being implemented",
                ));
            }
            fragments.push(apa_citation_member(&source, member, patent_punctuation));
        }
        if let Some((separate, profile, square)) = numeric_profile {
            let separator = if style == BibliographyStyle::Ieee {
                ", "
            } else {
                "; "
            };
            let content = fragments.join(separator);
            let text = if square {
                format!("[{content}]")
            } else {
                format!("({content})")
            };
            let text = rdocx_layout::engine::format_numeric_field_general(full_instruction, &text)
                .map_err(|diagnostic| bibliography_error(&diagnostic))?;
            if profile == 0 && !text.is_ascii() {
                if text.chars().any(|character| {
                    !matches!(character,
                    '\u{0000}'..='\u{00ff}' | '\u{0370}'..='\u{03ff}' |
                    '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
                }) {
                    return Err(bibliography_error(
                        "catalogued numeric source-script run branch is still being implemented",
                    ));
                }
                let mut runs = bibliography_display_runs(
                    &if separate { text } else { format!(" {text}") },
                    false,
                );
                for run in &mut runs {
                    if let Some(properties) = &mut run.properties {
                        properties.language = None;
                    }
                }
                if separate {
                    let mut leading = rdocx_oxml::text::CT_R::new(" ");
                    leading.properties = Some(native_citation_run_properties(0)?);
                    runs.insert(0, leading);
                }
                return Ok(Some(runs));
            }
            let mut run = rdocx_oxml::text::CT_R::new(if separate { &text } else { "" });
            if !separate {
                run = rdocx_oxml::text::CT_R::new(&format!(" {text}"));
            }
            run.properties = Some(native_citation_run_properties(profile)?);
            if !separate {
                return Ok(Some(vec![run]));
            }
            let mut leading = rdocx_oxml::text::CT_R::new(" ");
            leading.properties = Some(native_citation_run_properties(0)?);
            return Ok(Some(vec![leading, run]));
        }
        let mut leading = rdocx_oxml::text::CT_R::new(" ");
        leading.properties = Some(rdocx_oxml::CT_RPr {
            no_proof: Some(true),
            ..Default::default()
        });
        let mut citation = rdocx_oxml::text::CT_R::new(&format!("({})", fragments.join("; ")));
        citation.properties = leading.properties.clone();
        Ok(Some(vec![leading, citation]))
    }
}

fn iso_numeric_run_profile(locale: &BibliographyFormattingLocale) -> Option<(bool, usize, bool)> {
    let numeric = |number| match number {
        1025 | 1037 | 1056 | 1065 | 1085 | 1114 | 1119 | 1120 | 1123 | 1125 | 1152 | 2049
        | 2080 | 2118 | 2137 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
        | 12289 | 13313 | 14337 | 15361 | 16385 => Some((true, 1, false)),
        1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
        | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
        | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
        | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
        | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
        | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
        | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
        | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
        | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
        | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
        | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250 | 10252
        | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324 | 14345 | 14346
        | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418 | 18441 | 18442 | 19466
        | 20490 | 21514 | 58378 | 58380 => Some((true, 0, false)),
        1028 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124 => Some((true, 3, false)),
        1041 => Some((true, 3, true)),
        1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
        | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
        | 2144 | 2145 => Some((true, 6, false)),
        2057 | 2129 => Some((false, 0, false)),
        _ => None,
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

fn first_source_property(source: &BibliographySource, field: BibliographySourceField) -> &str {
    source
        .properties
        .iter()
        .find(|property| property.field == field)
        .map(|property| property.value.as_str())
        .unwrap_or("")
}

#[derive(Debug, PartialEq, Eq)]
enum NativeCitationRun {
    Literal(&'static str, usize),
    Country(usize),
    Number {
        letters: usize,
        digits: usize,
    },
    Year(usize),
    CaseTitle {
        letters: usize,
        digits: usize,
    },
    CreatorLast {
        letters: usize,
        digits: usize,
    },
    CreatorFirst {
        letters: usize,
        digits: usize,
    },
    CreatorMiddle {
        letters: usize,
        digits: usize,
    },
    CreatorSpace {
        middle: bool,
        letters: usize,
        digits: usize,
    },
    SourceTitle {
        letters: usize,
        digits: usize,
        spaces_after_letters: usize,
        spaces_after_digits: usize,
    },
}

// Native citation fragments contain literal language grammar, never cached source values.
fn apa_patent_run_grammar(
    locale: &BibliographyFormattingLocale,
) -> Option<(bool, &'static [NativeCitationRun])> {
    use NativeCitationRun::{Country, Literal, Number, Year};
    let numeric = |number| -> Option<(bool, &'static [NativeCitationRun])> {
        match number {
            1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
            | 12289 | 13313 | 14337 | 15361 | 16385 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" رقم براءة الاختراع ", 1),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal("، ", 1),
                    Year(1),
                    Literal(")", 1),
                ],
            )),
            1026 | 1049 | 1087 | 1088 | 1092 | 2073 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Патент № ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1027 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent núm. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1028 | 3076 | 5124 => Some((
                true,
                &[
                    Literal("(", 3),
                    Country(3),
                    Literal(" ", 3),
                    Literal("專利號碼", 4),
                    Literal(" ", 3),
                    Number {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1029 | 1051 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent č. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1030 | 1031 | 1043 | 1044 | 1070 | 1078 | 1080 | 1083 | 1122 | 1145 | 2055 | 2067
            | 2068 | 3079 | 4103 | 5127 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patentnr. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1032 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Ευρεσιτεχνία Αρ. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1033 | 1055 | 1068 | 1072 | 1073 | 1075 | 1084 | 1117 | 1126 | 1127 | 1129 | 1137
            | 1141 | 1142 | 1143 | 2092 | 2115 | 2143 | 2163 | 3081 | 4105 | 5129 | 6153 | 7177
            | 8201 | 9225 | 10249 | 11273 | 12297 | 13321 | 14345 | 15369 | 16393 | 17417
            | 18441 | 58378 | 58380 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent No. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1034 | 1140 | 2058 | 3082 | 4106 | 5130 | 6154 | 7178 | 8202 | 9226 | 10250 | 11274
            | 12298 | 13322 | 14346 | 15370 | 16394 | 17418 | 18442 | 19466 | 20490 | 21514 => {
                Some((
                    true,
                    &[
                        Literal("(", 0),
                        Country(0),
                        Literal(" Patente nº ", 0),
                        Number {
                            letters: 0,
                            digits: 0,
                        },
                        Literal(", ", 0),
                        Year(0),
                        Literal(")", 0),
                    ],
                ))
            }
            1035 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patenttinro ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1036 | 2060 | 3084 | 4108 | 5132 | 6156 | 7180 | 8204 | 9228 | 10252 | 11276
            | 12300 | 13324 | 14348 | 15372 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Brevet n° ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1037 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" פטנט מס' ", 1),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(", ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            1038 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Szabadalom száma: ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1039 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Einkaleyfi nr. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1040 | 2064 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Brevetto n. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1041 => Some((
                true,
                &[
                    Literal("[", 3),
                    Country(3),
                    Literal(" ", 3),
                    Literal("特許番号", 4),
                    Literal(": ", 3),
                    Number {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal("]", 3),
                ],
            )),
            1042 => Some((
                true,
                &[
                    Literal("(", 3),
                    Country(3),
                    Literal(" ", 3),
                    Literal("특허권", 5),
                    Literal(" ", 3),
                    Literal("번호", 5),
                    Literal(": ", 3),
                    Number {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1045 | 1061 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent nr ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1046 | 2070 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patente Nº ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1047 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent Nr. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1048 | 2072 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Brevet nr. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1050 | 2074 | 4122 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Br. patenta ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1052 | 1062 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patenta nr. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1053 | 2077 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patentnr ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1054 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("เลขที่สิทธิบัตร", 7),
                    Literal(" ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1056 | 2080 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" پیٹینٹ نمبر ", 1),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(", ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            1057 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Paten No. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1058 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Патент №", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1059 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Патэнт № ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1060 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Št. patenta ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1063 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patento Nr. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1064 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Патенти рақ. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1065 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" ش. حق انحصاري ", 1),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(", ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            1066 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Đăng ký Độc quyền Nhãn hiệu Số ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1067 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" ", 0),
                    Literal("Արտոնագիր", 8),
                    Literal(" ", 0),
                    Literal("հմ", 8),
                    Literal(". ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1069 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent zk. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1071 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Патент бр. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1074 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Nomoro ya Patente. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1076 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Inomb. yepeyitenti ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1077 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Inomb. Yephathenti ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1079 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" ", 0),
                    Literal("პატენტის", 9),
                    Literal(" ", 0),
                    Literal("ნომ", 9),
                    Literal(". ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1081 | 1112 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("पेटेंट", 10),
                    Literal(" ", 6),
                    Literal("क्र", 10),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1082 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Nru tal-Privattiva ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1085 | 1119 | 1120 | 1125 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" Patent No. ", 2),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(", ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            1086 | 2110 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" No. Paten ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1089 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Hataza Na. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1090 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent belgisi ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1091 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal("-sonli patent, ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1093 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("পেটেন্ট", 11),
                    Literal(" ", 6),
                    Literal("নম্বর", 11),
                    Literal(" ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1094 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("ਪੇਟੰਟ", 12),
                    Literal(" ", 6),
                    Literal("ਨੰ", 12),
                    Literal(" ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1095 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("પેટન્ટ", 13),
                    Literal(" ", 6),
                    Literal("નં", 13),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1096 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("ପେଟେଣ୍ଟ", 14),
                    Literal(" ", 6),
                    Literal("କ୍ର", 14),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1097 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("காப்புரிமை", 15),
                    Literal(" ", 6),
                    Literal("எண்", 15),
                    Literal(" ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1098 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("పేటెంట్", 16),
                    Literal(" ", 6),
                    Literal("సం", 16),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1099 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("ಪೇಟೆಂಟ್", 17),
                    Literal(" ", 6),
                    Literal("ಸಂಖ್ಯೆ", 17),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1100 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("പേറ്റന്റ്", 18),
                    Literal(" ", 6),
                    Literal("നമ്പര്‍", 18),
                    Literal(" ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1101 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("পেটেন্ট", 11),
                    Literal(" ", 6),
                    Literal("ক্ৰ", 11),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1102 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("पेटंट", 10),
                    Literal(" ", 6),
                    Literal("क्र", 10),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1103 | 1109 | 1113 | 2144 | 2145 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" Patent No. ", 2),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1104 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Патентийн Д.д. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1105 | 2128 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("专", 19),
                    Literal("利号", 4),
                    Literal(" ", 2),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1106 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Rhif Patent ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1107 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("ប៉ាតង់លេខ", 20),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1108 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("ເລກທີສິດ", 21),
                    Literal("\u{200B}", 6),
                    Literal("ທິ", 21),
                    Literal("\u{200B}", 6),
                    Literal("ບັດ", 21),
                    Literal(" ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1110 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patente núm. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1111 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("पेटेंट", 10),
                    Literal(" ", 6),
                    Literal("सं", 10),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1115 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("පේටන්ට්", 22),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1116 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" ", 0),
                    Literal("ᎠᏤᏝᏅ", 23),
                    Literal(" ", 0),
                    Literal("ᏗᏎᏍᏗ", 23),
                    Literal(". ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1118 | 1138 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" ", 0),
                    Literal("የፓ", 24),
                    Literal(". ", 0),
                    Literal("ቁ", 24),
                    Literal(". ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1121 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("विशिष्ट", 10),
                    Literal(" ", 6),
                    Literal("अधिकार", 10),
                    Literal(" ", 6),
                    Literal("पत्र", 10),
                    Literal(" ", 6),
                    Literal("संख्या", 10),
                    Literal(" ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            1123 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" ", 2),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal("امتیازی حق نمبر, ", 1),
                    Year(1),
                    Literal(")", 1),
                ],
            )),
            1124 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Blg. ng Patent ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1128 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Lambar takaddar izini ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1130 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" NỌ. Aṣẹ ọja tita ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1131 | 2155 | 3179 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Sut'i nº ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1132 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Nomoro ya Phatente No. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1136 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Nọmba Ikike ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1139 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" ", 0),
                    Literal("ቁ", 24),
                    Literal(". ", 0),
                    Literal("ፍቃድ", 24),
                    Literal(" ", 0),
                    Literal("ሓላፍነት", 24),
                    Literal(" ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1144 | 2052 | 4100 => Some((
                true,
                &[
                    Literal("(", 3),
                    Country(3),
                    Literal(" ", 3),
                    Literal("专", 19),
                    Literal("利号", 4),
                    Literal(" ", 3),
                    Number {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1152 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" پاتېنت نومۇرى: ", 1),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(", ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            1153 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Tau Tohu tiaki tenenga ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            2057 | 2129 => Some((
                false,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent No. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            2108 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Uimh. Phaitinne: ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            2117 => Some((
                true,
                &[
                    Literal("(", 6),
                    Country(2),
                    Literal(" ", 2),
                    Literal("পেটেন্ট", 11),
                    Literal("/", 6),
                    Literal("স্বত্ব", 11),
                    Literal(" ", 6),
                    Literal("নং", 11),
                    Literal(". ", 6),
                    Number {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            2118 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" پیٹنٹ نمبر ", 1),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(", ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            2137 => Some((
                true,
                &[
                    Literal("(", 1),
                    Country(2),
                    Literal(" ظاھري نمبر۔ ", 1),
                    Number {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(", ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            3098 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Бр. патента ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            5146 => Some((
                true,
                &[
                    Literal("(", 0),
                    Country(0),
                    Literal(" Patent br. ", 0),
                    Number {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            _ => None,
        }
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

fn apa_case_run_grammar(
    locale: &BibliographyFormattingLocale,
) -> Option<(bool, &'static [NativeCitationRun])> {
    use NativeCitationRun::{CaseTitle, Literal, Year};
    let numeric = |number| -> Option<(bool, &'static [NativeCitationRun])> {
        match number {
            1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
            | 12289 | 13313 | 14337 | 15361 | 16385 => Some((
                true,
                &[
                    Literal("(", 1),
                    CaseTitle {
                        letters: 2,
                        digits: 2,
                    },
                    Literal("، ", 1),
                    Year(1),
                    Literal(")", 1),
                ],
            )),
            1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
            | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
            | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
            | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
            | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
            | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
            | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
            | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
            | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
            | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
            | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
            | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
            | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
            | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380 => Some((
                true,
                &[
                    Literal("(", 0),
                    CaseTitle {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1028 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124 => Some((
                true,
                &[
                    Literal("(", 3),
                    CaseTitle {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137 => {
                Some((
                    true,
                    &[
                        Literal("(", 1),
                        CaseTitle {
                            letters: 2,
                            digits: 2,
                        },
                        Literal(", ", 2),
                        Year(2),
                        Literal(")", 1),
                    ],
                ))
            }
            1041 => Some((
                true,
                &[
                    Literal("[", 3),
                    CaseTitle {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal("]", 3),
                ],
            )),
            1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
            | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
            | 2144 | 2145 => Some((
                true,
                &[
                    Literal("(", 6),
                    CaseTitle {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            2057 | 2129 => Some((
                false,
                &[
                    Literal("(", 0),
                    CaseTitle {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            _ => None,
        }
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

// Chicago ordinary creator/year grammar from the authenticated full223 native matrix.
// APA ordinary creator/year grammar from unique-creator full223 native controls.
fn apa_creator_run_grammar(
    locale: &BibliographyFormattingLocale,
) -> Option<(bool, &'static [NativeCitationRun])> {
    use NativeCitationRun::{CreatorFirst, CreatorLast, Literal, Year};
    let numeric = |number| -> Option<(bool, &'static [NativeCitationRun])> {
        match number {
            1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
            | 12289 | 13313 | 14337 | 15361 | 16385 => Some((
                true,
                &[
                    Literal("(", 1),
                    CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    Literal("، ", 1),
                    Year(1),
                    Literal(")", 1),
                ],
            )),
            1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
            | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
            | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
            | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
            | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
            | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
            | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
            | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
            | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
            | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
            | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
            | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
            | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
            | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380 => Some((
                true,
                &[
                    Literal("(", 0),
                    CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1028 | 1144 | 2052 | 3076 | 4100 | 5124 => Some((
                true,
                &[
                    Literal("(", 3),
                    CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137 => {
                Some((
                    true,
                    &[
                        Literal("(", 1),
                        CreatorLast {
                            letters: 2,
                            digits: 2,
                        },
                        Literal(", ", 2),
                        Year(2),
                        Literal(")", 1),
                    ],
                ))
            }
            1041 => Some((
                true,
                &[
                    Literal("[", 3),
                    CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal("]", 3),
                ],
            )),
            1042 => Some((
                true,
                &[
                    Literal("(", 3),
                    CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    CreatorFirst {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(", ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
            | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
            | 2144 | 2145 => Some((
                true,
                &[
                    Literal("(", 6),
                    CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(", ", 2),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            2057 | 2129 => Some((
                false,
                &[
                    Literal("(", 0),
                    CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(", ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            _ => None,
        }
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

fn chicago_run_grammar(
    locale: &BibliographyFormattingLocale,
) -> Option<(bool, &'static [NativeCitationRun])> {
    use NativeCitationRun::{CreatorFirst, CreatorLast, Literal, Year};
    let numeric = |number| -> Option<(bool, &'static [NativeCitationRun])> {
        match number {
            1025 | 1037 | 1056 | 1065 | 1085 | 1114 | 1119 | 1120 | 1123 | 1125 | 1152 | 2049
            | 2080 | 2118 | 2137 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241
            | 11265 | 12289 | 13313 | 14337 | 15361 | 16385 => Some((
                true,
                &[
                    Literal("(", 1),
                    CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    Literal(" ", 2),
                    Year(2),
                    Literal(")", 1),
                ],
            )),
            1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
            | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
            | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
            | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
            | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
            | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
            | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
            | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
            | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
            | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
            | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
            | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
            | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
            | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380 => Some((
                true,
                &[
                    Literal("(", 0),
                    CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(" ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            1028 | 1144 | 2052 | 3076 | 4100 | 5124 => Some((
                true,
                &[
                    Literal("(", 3),
                    CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(" ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1041 => Some((
                true,
                &[
                    Literal("[", 3),
                    CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(" ", 3),
                    Year(3),
                    Literal("]", 3),
                ],
            )),
            1042 => Some((
                true,
                &[
                    Literal("(", 3),
                    CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    CreatorFirst {
                        letters: 3,
                        digits: 3,
                    },
                    Literal(" ", 3),
                    Year(3),
                    Literal(")", 3),
                ],
            )),
            1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
            | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
            | 2144 | 2145 => Some((
                true,
                &[
                    Literal("(", 6),
                    CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    Literal(" ", 6),
                    Year(6),
                    Literal(")", 6),
                ],
            )),
            2057 | 2129 => Some((
                false,
                &[
                    Literal("(", 0),
                    CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    Literal(" ", 0),
                    Year(0),
                    Literal(")", 0),
                ],
            )),
            _ => None,
        }
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

fn iso_author_date_run_grammar(
    locale: &BibliographyFormattingLocale,
    title: bool,
) -> Option<(bool, &'static [NativeCitationRun])> {
    let numeric = |number| -> Option<(bool, &'static [NativeCitationRun])> {
        match (title, number) {
            (
                false,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal("، ", 1),
                    NativeCitationRun::Year(1),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                true,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 2,
                        spaces_after_digits: 2,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal("، ", 1),
                    NativeCitationRun::Year(1),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                false,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                true,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (false, 1028 | 1144 | 2052 | 3076 | 4100 | 5124) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (true, 1028 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::SourceTitle {
                        letters: 3,
                        digits: 3,
                        spaces_after_digits: 3,
                        spaces_after_letters: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (
                false,
                1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                true,
                1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 2,
                        spaces_after_digits: 2,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (false, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (true, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::SourceTitle {
                        letters: 3,
                        digits: 3,
                        spaces_after_digits: 3,
                        spaces_after_letters: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (false, 1042) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::CreatorFirst {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (false, 1054) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorFirst {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorSpace {
                        middle: false,
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorMiddle {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorSpace {
                        middle: true,
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (
                true,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 6,
                        spaces_after_digits: 6,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (
                false,
                1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102 | 1103
                | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128 | 2144
                | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (false, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (true, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            _ => None,
        }
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

fn sist_run_grammar(
    locale: &BibliographyFormattingLocale,
    title: bool,
) -> Option<(bool, &'static [NativeCitationRun])> {
    let numeric = |number| -> Option<(bool, &'static [NativeCitationRun])> {
        match (title, number) {
            (
                false,
                1025 | 1037 | 1056 | 1065 | 1085 | 1114 | 1119 | 1120 | 1123 | 1125 | 1152 | 2049
                | 2080 | 2118 | 2137 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241
                | 11265 | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 25),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal("）", 25),
                ],
            )),
            (
                true,
                1025 | 1037 | 1056 | 1065 | 1085 | 1114 | 1119 | 1120 | 1123 | 1125 | 1152 | 2049
                | 2080 | 2118 | 2137 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241
                | 11265 | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 25),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 2,
                        spaces_after_digits: 2,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal("）", 25),
                ],
            )),
            (
                false,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2057 | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092
                | 2108 | 2110 | 2115 | 2129 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098
                | 3179 | 4103 | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153
                | 6154 | 6156 | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228
                | 10249 | 10250 | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321
                | 13322 | 13324 | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394
                | 17417 | 17418 | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 4),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal("）", 4),
                ],
            )),
            (
                true,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2057 | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092
                | 2108 | 2110 | 2115 | 2129 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098
                | 3179 | 4103 | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153
                | 6154 | 6156 | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228
                | 10249 | 10250 | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321
                | 13322 | 13324 | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394
                | 17417 | 17418 | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 4),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal("）", 4),
                ],
            )),
            (false, 1028 | 1041 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 4),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("）", 4),
                ],
            )),
            (true, 1028 | 1041 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 4),
                    NativeCitationRun::SourceTitle {
                        letters: 3,
                        digits: 3,
                        spaces_after_digits: 3,
                        spaces_after_letters: 3,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("）", 4),
                ],
            )),
            (
                false,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 4),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal("）", 4),
                ],
            )),
            (
                true,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("（", 4),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 6,
                        spaces_after_digits: 6,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal("，", 4),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal("）", 4),
                ],
            )),
            _ => None,
        }
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

fn other_style_citation_run_grammar(
    style: BibliographyStyle,
    locale: &BibliographyFormattingLocale,
    title: bool,
) -> Option<(bool, &'static [NativeCitationRun])> {
    let numeric = |number| -> Option<(bool, &'static [NativeCitationRun])> {
        match (style, title, number) {
            (
                BibliographyStyle::Gb7714,
                false,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal("، ", 1),
                    NativeCitationRun::Year(1),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::Gb7714,
                true,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 2,
                        spaces_after_digits: 2,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal("، ", 1),
                    NativeCitationRun::Year(1),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::Gb7714,
                false,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::Gb7714,
                true,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (BibliographyStyle::Gb7714, false, 1028 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124) => {
                Some((
                    true,
                    &[
                        NativeCitationRun::Literal("(", 3),
                        NativeCitationRun::CreatorLast {
                            letters: 3,
                            digits: 3,
                        },
                        NativeCitationRun::Literal(", ", 3),
                        NativeCitationRun::Year(3),
                        NativeCitationRun::Literal(")", 3),
                    ],
                ))
            }
            (BibliographyStyle::Gb7714, true, 1028 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124) => {
                Some((
                    true,
                    &[
                        NativeCitationRun::Literal("(", 3),
                        NativeCitationRun::SourceTitle {
                            letters: 3,
                            digits: 3,
                            spaces_after_digits: 3,
                            spaces_after_letters: 3,
                        },
                        NativeCitationRun::Literal(", ", 3),
                        NativeCitationRun::Year(3),
                        NativeCitationRun::Literal(")", 3),
                    ],
                ))
            }
            (
                BibliographyStyle::Gb7714,
                false,
                1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::Gb7714,
                true,
                1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 2,
                        spaces_after_digits: 2,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (BibliographyStyle::Gb7714, false, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (BibliographyStyle::Gb7714, true, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::SourceTitle {
                        letters: 3,
                        digits: 3,
                        spaces_after_digits: 3,
                        spaces_after_letters: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (
                BibliographyStyle::Gb7714,
                false,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (
                BibliographyStyle::Gb7714,
                true,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::SourceTitle {
                        letters: 2,
                        digits: 6,
                        spaces_after_digits: 6,
                        spaces_after_letters: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (BibliographyStyle::Gb7714, false, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (BibliographyStyle::Gb7714, true, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::GostName,
                false,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal("، ", 1),
                    NativeCitationRun::Year(1),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::GostName,
                false,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (BibliographyStyle::GostName, false, 1028 | 1144 | 2052 | 3076 | 4100 | 5124) => {
                Some((
                    true,
                    &[
                        NativeCitationRun::Literal("(", 3),
                        NativeCitationRun::CreatorLast {
                            letters: 3,
                            digits: 3,
                        },
                        NativeCitationRun::Literal(", ", 3),
                        NativeCitationRun::Year(3),
                        NativeCitationRun::Literal(")", 3),
                    ],
                ))
            }
            (
                BibliographyStyle::GostName,
                false,
                1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (BibliographyStyle::GostName, false, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (BibliographyStyle::GostName, false, 1042) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::CreatorFirst {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (BibliographyStyle::GostName, false, 1054) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorFirst {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorSpace {
                        middle: false,
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorMiddle {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorSpace {
                        middle: true,
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (
                BibliographyStyle::GostName,
                false,
                1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102 | 1103
                | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128 | 2144
                | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (BibliographyStyle::GostName, false, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::GostTitle,
                false,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal("، ", 1),
                    NativeCitationRun::Year(1),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::GostTitle,
                false,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (BibliographyStyle::GostTitle, false, 1028 | 1144 | 2052 | 3076 | 4100 | 5124) => {
                Some((
                    true,
                    &[
                        NativeCitationRun::Literal("(", 3),
                        NativeCitationRun::CreatorLast {
                            letters: 3,
                            digits: 3,
                        },
                        NativeCitationRun::Literal(", ", 3),
                        NativeCitationRun::Year(3),
                        NativeCitationRun::Literal(")", 3),
                    ],
                ))
            }
            (
                BibliographyStyle::GostTitle,
                false,
                1037 | 1056 | 1065 | 1085 | 1119 | 1120 | 1123 | 1125 | 1152 | 2080 | 2118 | 2137,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (BibliographyStyle::GostTitle, false, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (BibliographyStyle::GostTitle, false, 1042) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::CreatorFirst {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(", ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (BibliographyStyle::GostTitle, false, 1054) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorFirst {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorSpace {
                        middle: false,
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorMiddle {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorSpace {
                        middle: true,
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (
                BibliographyStyle::GostTitle,
                false,
                1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102 | 1103
                | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128 | 2144
                | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(", ", 2),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (BibliographyStyle::GostTitle, false, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::HarvardAnglia,
                false,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal("، ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::HarvardAnglia,
                true,
                1025 | 1114 | 2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265
                | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal("، ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::HarvardAnglia,
                false,
                1026 | 1027 | 1028 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1037
                | 1038 | 1039 | 1040 | 1042 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050
                | 1051 | 1052 | 1053 | 1054 | 1055 | 1056 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062
                | 1063 | 1064 | 1065 | 1066 | 1067 | 1068 | 1069 | 1070 | 1071 | 1072 | 1073 | 1074
                | 1075 | 1076 | 1077 | 1078 | 1079 | 1080 | 1081 | 1082 | 1083 | 1084 | 1085 | 1086
                | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098
                | 1099 | 1100 | 1101 | 1102 | 1103 | 1104 | 1105 | 1106 | 1107 | 1108 | 1109 | 1110
                | 1111 | 1112 | 1113 | 1115 | 1116 | 1117 | 1118 | 1119 | 1120 | 1121 | 1122 | 1123
                | 1124 | 1125 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131 | 1132 | 1136 | 1137 | 1138
                | 1139 | 1140 | 1141 | 1142 | 1143 | 1144 | 1145 | 1152 | 1153 | 2052 | 2055 | 2057
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2080 | 2092
                | 2108 | 2110 | 2115 | 2117 | 2118 | 2128 | 2129 | 2137 | 2143 | 2144 | 2145 | 2155
                | 2163 | 3076 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4100 | 4103 | 4105 | 4106
                | 4108 | 4122 | 5124 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156 | 7177
                | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250 | 10252
                | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324 | 14345
                | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418 | 18441
                | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::HarvardAnglia,
                true,
                1026 | 1027 | 1028 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1037
                | 1038 | 1039 | 1040 | 1042 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050
                | 1051 | 1052 | 1053 | 1054 | 1055 | 1056 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062
                | 1063 | 1064 | 1065 | 1066 | 1067 | 1068 | 1069 | 1070 | 1071 | 1072 | 1073 | 1074
                | 1075 | 1076 | 1077 | 1078 | 1079 | 1080 | 1081 | 1082 | 1083 | 1084 | 1085 | 1086
                | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098
                | 1099 | 1100 | 1101 | 1102 | 1103 | 1104 | 1105 | 1106 | 1107 | 1108 | 1109 | 1110
                | 1111 | 1112 | 1113 | 1115 | 1116 | 1117 | 1118 | 1119 | 1120 | 1121 | 1122 | 1123
                | 1124 | 1125 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131 | 1132 | 1136 | 1137 | 1138
                | 1139 | 1140 | 1141 | 1142 | 1143 | 1144 | 1145 | 1152 | 1153 | 2052 | 2055 | 2057
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2080 | 2092
                | 2108 | 2110 | 2115 | 2117 | 2118 | 2128 | 2129 | 2137 | 2143 | 2144 | 2145 | 2155
                | 2163 | 3076 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4100 | 4103 | 4105 | 4106
                | 4108 | 4122 | 5124 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156 | 7177
                | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250 | 10252
                | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324 | 14345
                | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418 | 18441
                | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (BibliographyStyle::HarvardAnglia, false, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal("]", 0),
                ],
            )),
            (BibliographyStyle::HarvardAnglia, true, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 0),
                    NativeCitationRun::SourceTitle {
                        letters: 0,
                        digits: 0,
                        spaces_after_digits: 0,
                        spaces_after_letters: 0,
                    },
                    NativeCitationRun::Literal(", ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal("]", 0),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                false,
                1025 | 1037 | 1056 | 1065 | 1085 | 1114 | 1119 | 1120 | 1123 | 1125 | 1152 | 2049
                | 2080 | 2118 | 2137 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241
                | 11265 | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                true,
                1025 | 1037 | 1056 | 1065 | 1085 | 1114 | 1119 | 1120 | 1123 | 1125 | 1152 | 2049
                | 2080 | 2118 | 2137 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241
                | 11265 | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CaseTitle {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                false,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                true,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CaseTitle {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                false,
                1028 | 1144 | 2052 | 3076 | 4100 | 5124,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                true,
                1028 | 1042 | 1144 | 2052 | 3076 | 4100 | 5124,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CaseTitle {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (BibliographyStyle::MlaSeventhEdition, false, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (BibliographyStyle::MlaSeventhEdition, true, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::CaseTitle {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (BibliographyStyle::MlaSeventhEdition, false, 1042) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::CreatorFirst {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                false,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (
                BibliographyStyle::MlaSeventhEdition,
                true,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CaseTitle {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (BibliographyStyle::MlaSeventhEdition, false, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (BibliographyStyle::MlaSeventhEdition, true, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CaseTitle {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (
                BibliographyStyle::Turabian,
                false,
                1025 | 1037 | 1056 | 1065 | 1085 | 1114 | 1119 | 1120 | 1123 | 1125 | 1152 | 2049
                | 2080 | 2118 | 2137 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241
                | 11265 | 12289 | 13313 | 14337 | 15361 | 16385,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 1),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 2,
                    },
                    NativeCitationRun::Literal(" ", 2),
                    NativeCitationRun::Year(2),
                    NativeCitationRun::Literal(")", 1),
                ],
            )),
            (
                BibliographyStyle::Turabian,
                false,
                1026 | 1027 | 1029 | 1030 | 1031 | 1032 | 1033 | 1034 | 1035 | 1036 | 1038 | 1039
                | 1040 | 1043 | 1044 | 1045 | 1046 | 1047 | 1048 | 1049 | 1050 | 1051 | 1052 | 1053
                | 1055 | 1057 | 1058 | 1059 | 1060 | 1061 | 1062 | 1063 | 1064 | 1066 | 1067 | 1068
                | 1069 | 1070 | 1071 | 1072 | 1073 | 1074 | 1075 | 1076 | 1077 | 1078 | 1079 | 1080
                | 1082 | 1083 | 1084 | 1086 | 1087 | 1088 | 1089 | 1090 | 1091 | 1092 | 1104 | 1106
                | 1110 | 1116 | 1117 | 1118 | 1122 | 1124 | 1126 | 1127 | 1128 | 1129 | 1130 | 1131
                | 1132 | 1136 | 1137 | 1138 | 1139 | 1140 | 1141 | 1142 | 1143 | 1145 | 1153 | 2055
                | 2058 | 2060 | 2064 | 2067 | 2068 | 2070 | 2072 | 2073 | 2074 | 2077 | 2092 | 2108
                | 2110 | 2115 | 2143 | 2155 | 2163 | 3079 | 3081 | 3082 | 3084 | 3098 | 3179 | 4103
                | 4105 | 4106 | 4108 | 4122 | 5127 | 5129 | 5130 | 5132 | 5146 | 6153 | 6154 | 6156
                | 7177 | 7178 | 7180 | 8201 | 8202 | 8204 | 9225 | 9226 | 9228 | 10249 | 10250
                | 10252 | 11273 | 11274 | 11276 | 12297 | 12298 | 12300 | 13321 | 13322 | 13324
                | 14345 | 14346 | 14348 | 15369 | 15370 | 15372 | 16393 | 16394 | 17417 | 17418
                | 18441 | 18442 | 19466 | 20490 | 21514 | 58378 | 58380,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(" ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            (BibliographyStyle::Turabian, false, 1028 | 1144 | 2052 | 3076 | 4100 | 5124) => {
                Some((
                    true,
                    &[
                        NativeCitationRun::Literal("(", 3),
                        NativeCitationRun::CreatorLast {
                            letters: 3,
                            digits: 3,
                        },
                        NativeCitationRun::Literal(" ", 3),
                        NativeCitationRun::Year(3),
                        NativeCitationRun::Literal(")", 3),
                    ],
                ))
            }
            (BibliographyStyle::Turabian, false, 1041) => Some((
                true,
                &[
                    NativeCitationRun::Literal("[", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(" ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal("]", 3),
                ],
            )),
            (BibliographyStyle::Turabian, false, 1042) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 3),
                    NativeCitationRun::CreatorLast {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::CreatorFirst {
                        letters: 3,
                        digits: 3,
                    },
                    NativeCitationRun::Literal(" ", 3),
                    NativeCitationRun::Year(3),
                    NativeCitationRun::Literal(")", 3),
                ],
            )),
            (
                BibliographyStyle::Turabian,
                false,
                1054 | 1081 | 1093 | 1094 | 1095 | 1096 | 1097 | 1098 | 1099 | 1100 | 1101 | 1102
                | 1103 | 1105 | 1107 | 1108 | 1109 | 1111 | 1112 | 1113 | 1115 | 1121 | 2117 | 2128
                | 2144 | 2145,
            ) => Some((
                true,
                &[
                    NativeCitationRun::Literal("(", 6),
                    NativeCitationRun::CreatorLast {
                        letters: 2,
                        digits: 6,
                    },
                    NativeCitationRun::Literal(" ", 6),
                    NativeCitationRun::Year(6),
                    NativeCitationRun::Literal(")", 6),
                ],
            )),
            (BibliographyStyle::Turabian, false, 2057 | 2129) => Some((
                false,
                &[
                    NativeCitationRun::Literal("(", 0),
                    NativeCitationRun::CreatorLast {
                        letters: 0,
                        digits: 0,
                    },
                    NativeCitationRun::Literal(" ", 0),
                    NativeCitationRun::Year(0),
                    NativeCitationRun::Literal(")", 0),
                ],
            )),
            _ => None,
        }
    };
    match locale {
        BibliographyFormattingLocale::Numeric(number) => numeric(*number),
        BibliographyFormattingLocale::Lexical(name) => {
            let mut choices = SOURCE_LOCALE_ENCODINGS
                .iter()
                .filter(|(_, encoding)| encoding.eq_ignore_ascii_case(name))
                .map(|(number, _)| numeric(*number));
            let first = choices.next()??;
            choices.all(|choice| choice == Some(first)).then_some(first)
        }
    }
}

fn citation_creator_role(
    source: &BibliographySource,
    style: BibliographyStyle,
) -> Option<BibliographyContributorRole> {
    use BibliographyContributorRole as Role;
    use BibliographySourceKind as Kind;
    let roles: &[Role] = match (style, source.kind) {
        (BibliographyStyle::Gb7714, Kind::SoundRecording) => &[Role::Artist],
        (BibliographyStyle::Gb7714, Kind::Performance) => &[Role::Writer],
        (BibliographyStyle::Gb7714, Kind::Art) => &[Role::Artist],
        (BibliographyStyle::Gb7714, Kind::Film) => &[Role::Director],
        (BibliographyStyle::Gb7714, Kind::Interview) => &[Role::Interviewee],
        (BibliographyStyle::Gb7714, Kind::Patent) => &[Role::Inventor],
        (BibliographyStyle::GostName, Kind::SoundRecording) => &[Role::Artist],
        (BibliographyStyle::GostName, Kind::Performance) => &[Role::Writer],
        (BibliographyStyle::GostName, Kind::Art) => &[Role::Artist],
        (BibliographyStyle::GostName, Kind::Film) => &[Role::Director],
        (BibliographyStyle::GostName, Kind::Interview) => &[Role::Interviewee],
        (BibliographyStyle::GostName, Kind::Patent) => &[Role::Inventor],
        (BibliographyStyle::GostTitle, Kind::SoundRecording) => &[Role::Artist],
        (BibliographyStyle::GostTitle, Kind::Performance) => &[Role::Writer],
        (BibliographyStyle::GostTitle, Kind::Art) => &[Role::Artist],
        (BibliographyStyle::GostTitle, Kind::Film) => &[Role::Director],
        (BibliographyStyle::GostTitle, Kind::Interview) => &[Role::Interviewee],
        (BibliographyStyle::GostTitle, Kind::Patent) => &[Role::Inventor],
        (BibliographyStyle::HarvardAnglia, Kind::SoundRecording) => &[Role::Composer],
        (BibliographyStyle::HarvardAnglia, Kind::Performance) => &[Role::Writer],
        (BibliographyStyle::HarvardAnglia, Kind::Art) => &[Role::Artist],
        (BibliographyStyle::HarvardAnglia, Kind::Interview) => &[Role::Interviewee],
        (BibliographyStyle::HarvardAnglia, Kind::Patent) => &[Role::Inventor],
        (BibliographyStyle::MlaSeventhEdition, Kind::SoundRecording) => &[Role::Performer],
        (BibliographyStyle::MlaSeventhEdition, Kind::Performance) => &[Role::Writer],
        (BibliographyStyle::MlaSeventhEdition, Kind::Art) => &[Role::Artist],
        (BibliographyStyle::MlaSeventhEdition, Kind::Film) => &[Role::Writer],
        (BibliographyStyle::MlaSeventhEdition, Kind::Interview) => &[Role::Interviewee],
        (BibliographyStyle::MlaSeventhEdition, Kind::Patent) => &[Role::Inventor],
        (BibliographyStyle::Turabian, Kind::SoundRecording) => &[Role::Performer],
        (BibliographyStyle::Turabian, Kind::Performance) => &[Role::Writer],
        (BibliographyStyle::Turabian, Kind::Art) => &[Role::Artist],
        (BibliographyStyle::Turabian, Kind::Film) => &[Role::Writer],
        (BibliographyStyle::Turabian, Kind::Interview) => &[Role::Interviewee],
        (BibliographyStyle::Turabian, Kind::Patent) => &[Role::Inventor],
        (BibliographyStyle::Iso690AuthorDate | BibliographyStyle::Sist02, Kind::SoundRecording) => {
            &[Role::Artist]
        }
        (BibliographyStyle::Iso690AuthorDate | BibliographyStyle::Sist02, Kind::Film) => {
            &[Role::Director]
        }
        (BibliographyStyle::Iso690AuthorDate | BibliographyStyle::Sist02, Kind::Performance) => {
            &[Role::Writer]
        }
        (
            BibliographyStyle::Iso690AuthorDate | BibliographyStyle::Sist02,
            Kind::JournalArticle | Kind::ConferenceProceedings,
        ) => &[],
        (BibliographyStyle::Chicago, Kind::SoundRecording) => &[Role::Performer],
        (BibliographyStyle::Chicago, Kind::Film) => &[Role::Writer],
        (BibliographyStyle::Chicago, Kind::Case) => &[Role::Author],
        (BibliographyStyle::Chicago, Kind::Performance) => &[Role::Writer],
        (BibliographyStyle::ApaSixthEdition, Kind::SoundRecording) => {
            &[Role::Composer, Role::Conductor]
        }
        (BibliographyStyle::ApaSixthEdition, Kind::Film) => {
            &[Role::ProducerName, Role::Director, Role::Writer]
        }
        (BibliographyStyle::ApaSixthEdition, Kind::Case) => &[],
        (BibliographyStyle::ApaSixthEdition, Kind::Book) => &[Role::Author, Role::Editor],
        (_, Kind::Performance) => &[Role::Writer, Role::Performer],
        (_, Kind::Art) => &[Role::Artist],
        (_, Kind::Interview) => &[Role::Interviewee],
        (_, Kind::Patent) => &[Role::Inventor],
        _ => &[Role::Author],
    };
    roles.iter().copied().find(|role| {
        source
            .contributors
            .iter()
            .any(|contributor| contributor.role == *role)
    })
}

fn single_citation_creator(
    source: &BibliographySource,
    style: BibliographyStyle,
) -> Option<&BibliographyPerson> {
    let role = citation_creator_role(source, style)?;
    let mut people = source
        .contributors
        .iter()
        .filter(|contributor| contributor.role == role)
        .flat_map(|contributor| match &contributor.value {
            BibliographyAuthor::People(people) => people.as_slice(),
            _ => &[],
        });
    let person = people.next()?;
    people.next().is_none().then_some(person)
}

fn has_equivalent_creator(
    collection: &BibliographyCollection,
    index: usize,
    creator: &BibliographyPerson,
    style: BibliographyStyle,
) -> Result<bool> {
    for (other_index, other_identity) in &collection.sources {
        if *other_index == index {
            continue;
        }
        if let Some(other) = collection.parsed.supported_source(
            *other_index,
            &other_identity.tag,
            other_identity.guid.as_deref(),
            other_identity.source_type.as_deref(),
            false,
        )? && single_citation_creator(&other, style).is_some_and(|person| {
            person.last == creator.last
                && person.first == creator.first
                && person.middle == creator.middle
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn native_citation_run_properties(profile: usize) -> Result<rdocx_oxml::CT_RPr> {
    use rdocx_oxml::CT_RPr;
    Ok(match profile {
        0 => CT_RPr {
            no_proof: Some(true),
            ..Default::default()
        },
        1 => CT_RPr {
            no_proof: Some(true),
            font_hint: Some("cs".into()),
            rtl: Some(true),
            ..Default::default()
        },
        2 => CT_RPr {
            no_proof: Some(true),
            font_hint: Some("cs".into()),
            ..Default::default()
        },
        3 => CT_RPr {
            no_proof: Some(true),
            font_hint: Some("eastAsia".into()),
            ..Default::default()
        },
        4 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("MS Gothic".into()),
            font_east_asia: Some("MS Gothic".into()),
            font_hansi: Some("MS Gothic".into()),
            font_cs: Some("MS Gothic".into()),
            font_hint: Some("eastAsia".into()),
            ..Default::default()
        },
        5 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Malgun Gothic".into()),
            font_east_asia: Some("Malgun Gothic".into()),
            font_hansi: Some("Malgun Gothic".into()),
            font_cs: Some("Malgun Gothic".into()),
            font_hint: Some("eastAsia".into()),
            ..Default::default()
        },
        6 => CT_RPr {
            no_proof: Some(true),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        7 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Angsana New".into()),
            font_hansi: Some("Angsana New".into()),
            font_cs: Some("Angsana New".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        8 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Tahoma".into()),
            font_hansi: Some("Tahoma".into()),
            font_cs: Some("Tahoma".into()),
            ..Default::default()
        },
        9 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Helvetica".into()),
            font_hansi: Some("Helvetica".into()),
            font_cs: Some("Helvetica".into()),
            ..Default::default()
        },
        10 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Kohinoor Devanagari".into()),
            font_hansi: Some("Kohinoor Devanagari".into()),
            font_cs: Some("Kohinoor Devanagari".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        11 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Kohinoor Bangla".into()),
            font_hansi: Some("Kohinoor Bangla".into()),
            font_cs: Some("Kohinoor Bangla".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        12 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Gurmukhi MN".into()),
            font_hansi: Some("Gurmukhi MN".into()),
            font_cs: Some("Gurmukhi MN".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        13 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Gujarati Sangam MN".into()),
            font_hansi: Some("Gujarati Sangam MN".into()),
            font_cs: Some("Gujarati Sangam MN".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        14 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Noto Sans Oriya".into()),
            font_hansi: Some("Noto Sans Oriya".into()),
            font_cs: Some("Noto Sans Oriya".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        15 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Latha".into()),
            font_hansi: Some("Latha".into()),
            font_cs: Some("Latha".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        16 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Gautami".into()),
            font_hansi: Some("Gautami".into()),
            font_cs: Some("Gautami".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        17 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Tunga".into()),
            font_hansi: Some("Tunga".into()),
            font_cs: Some("Tunga".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        18 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Kartika".into()),
            font_hansi: Some("Kartika".into()),
            font_cs: Some("Kartika".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        19 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Microsoft JhengHei".into()),
            font_east_asia: Some("Microsoft JhengHei".into()),
            font_hansi: Some("Microsoft JhengHei".into()),
            font_cs: Some("Microsoft JhengHei".into()),
            font_hint: Some("eastAsia".into()),
            ..Default::default()
        },
        20 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Khmer Sangam MN".into()),
            font_hansi: Some("Khmer Sangam MN".into()),
            font_cs: Some("Khmer Sangam MN".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        21 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Lao Sangam MN".into()),
            font_hansi: Some("Lao Sangam MN".into()),
            font_cs: Some("Lao Sangam MN".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        22 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Sinhala Sangam MN".into()),
            font_hansi: Some("Sinhala Sangam MN".into()),
            font_cs: Some("Sinhala Sangam MN".into()),
            font_hint: Some("cs".into()),
            complex_script: Some(true),
            ..Default::default()
        },
        23 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Plantagenet Cherokee".into()),
            font_hansi: Some("Plantagenet Cherokee".into()),
            font_cs: Some("Plantagenet Cherokee".into()),
            ..Default::default()
        },
        24 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("Nyala".into()),
            font_hansi: Some("Nyala".into()),
            font_cs: Some("Nyala".into()),
            ..Default::default()
        },
        25 => CT_RPr {
            no_proof: Some(true),
            font_ascii: Some("MS Mincho".into()),
            font_hansi: Some("MS Mincho".into()),
            font_east_asia: Some("MS Mincho".into()),
            font_cs: Some("MS Mincho".into()),
            font_hint: Some("eastAsia".into()),
            rtl: Some(true),
            ..Default::default()
        },
        _ => return Err(bibliography_error("invalid native citation run profile")),
    })
}

fn source_citation_runs(
    source: &BibliographySource,
    creator: Option<&BibliographyPerson>,
    separate_leading: bool,
    grammar: &[NativeCitationRun],
) -> Result<Vec<rdocx_oxml::text::CT_R>> {
    use rdocx_oxml::text::CT_R;
    let country = first_source_property(source, BibliographySourceField::CountryRegion);
    let number = first_source_property(source, BibliographySourceField::PatentNumber);
    let year = first_source_property(source, BibliographySourceField::Year);
    let case_title = first_source_property(source, BibliographySourceField::ShortTitle);
    let source_title = first_source_property(source, BibliographySourceField::Title);
    let creator_last = creator
        .and_then(|person| person.last.first())
        .map_or("", String::as_str);
    let creator_first = creator
        .and_then(|person| person.first.first())
        .map_or("", String::as_str);
    let creator_middle = creator
        .and_then(|person| person.middle.first())
        .map_or("", String::as_str);
    if (grammar
        .iter()
        .any(|token| matches!(token, NativeCitationRun::SourceTitle { .. }))
        && (!source_title
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == ' ')
            || source_title.starts_with(' ')
            || source_title.ends_with(' ')))
        || (grammar
            .iter()
            .any(|token| matches!(token, NativeCitationRun::CaseTitle { .. }))
            && !case_title
                .chars()
                .all(|character| character.is_ascii_alphanumeric()))
        || (grammar.iter().any(|token| {
            matches!(
                token,
                NativeCitationRun::Country(_) | NativeCitationRun::Number { .. }
            )
        }) && (!country
            .chars()
            .all(|character| character.is_ascii_alphabetic() || character == ' ')
            || !number
                .chars()
                .all(|character| character.is_ascii_alphanumeric())))
        || (grammar
            .iter()
            .any(|token| matches!(token, NativeCitationRun::CreatorLast { .. }))
            && !creator_last
                .chars()
                .all(|character| character.is_ascii_alphanumeric()))
        || (grammar
            .iter()
            .any(|token| matches!(token, NativeCitationRun::CreatorFirst { .. }))
            && !creator_first
                .chars()
                .all(|character| character.is_ascii_alphanumeric()))
        || (grammar
            .iter()
            .any(|token| matches!(token, NativeCitationRun::CreatorMiddle { .. }))
            && (creator.is_some_and(|person| person.middle.len() > 1)
                || !creator_middle
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())))
        || (grammar
            .iter()
            .any(|token| matches!(token, NativeCitationRun::Year(_)))
            && !year.chars().all(|character| character.is_ascii_digit()))
    {
        return Err(bibliography_error(
            "catalogued source-script citation run branch is still being implemented",
        ));
    }
    let mut leading = CT_R::new(" ");
    leading.properties = Some(native_citation_run_properties(0)?);
    let mut body: Vec<CT_R> = Vec::new();
    let mut append = |text: &str, profile| -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let properties = native_citation_run_properties(profile)?;
        if let Some(last) = body.last_mut()
            && last.properties.as_ref() == Some(&properties)
        {
            let Some(rdocx_oxml::text::RunContent::Text(content)) = last.content.first_mut() else {
                return Err(bibliography_error("invalid assembled citation run"));
            };
            content.text.push_str(text);
            content.preserve_space = content.text.starts_with(' ') || content.text.ends_with(' ');
        } else {
            let mut run = CT_R::new(text);
            run.properties = Some(properties);
            body.push(run);
        }
        Ok(())
    };
    for token in grammar {
        match token {
            NativeCitationRun::Literal(text, profile) => append(text, *profile)?,
            NativeCitationRun::Country(profile) => append(country, *profile)?,
            NativeCitationRun::Year(profile) => append(year, *profile)?,
            NativeCitationRun::CaseTitle { letters, digits }
            | NativeCitationRun::CreatorLast { letters, digits } => {
                let value = match token {
                    NativeCitationRun::CreatorLast { .. } => creator_last,
                    _ => case_title,
                };
                for character in value.chars() {
                    append(
                        &character.to_string(),
                        if character.is_ascii_digit() {
                            *digits
                        } else {
                            *letters
                        },
                    )?;
                }
            }
            NativeCitationRun::CreatorFirst { letters, digits }
            | NativeCitationRun::CreatorMiddle { letters, digits } => {
                let value = if matches!(token, NativeCitationRun::CreatorFirst { .. }) {
                    creator_first
                } else {
                    creator_middle
                };
                for character in value.chars() {
                    append(
                        &character.to_string(),
                        if character.is_ascii_digit() {
                            *digits
                        } else {
                            *letters
                        },
                    )?;
                }
            }
            NativeCitationRun::CreatorSpace {
                middle,
                letters,
                digits,
            } => {
                let component = if *middle {
                    creator_middle
                } else {
                    creator_first
                };
                append(
                    " ",
                    if component.ends_with(|character: char| character.is_ascii_digit()) {
                        *digits
                    } else {
                        *letters
                    },
                )?;
            }
            NativeCitationRun::SourceTitle {
                letters,
                digits,
                spaces_after_letters,
                spaces_after_digits,
            } => {
                let mut after_digit = false;
                for character in source_title.chars() {
                    let profile = if character.is_ascii_digit() {
                        *digits
                    } else if character == ' ' {
                        if after_digit {
                            *spaces_after_digits
                        } else {
                            *spaces_after_letters
                        }
                    } else {
                        *letters
                    };
                    append(&character.to_string(), profile)?;
                    if character != ' ' {
                        after_digit = character.is_ascii_digit();
                    }
                }
            }
            NativeCitationRun::Number { letters, digits } => {
                for character in number.chars() {
                    append(
                        &character.to_string(),
                        if character.is_ascii_digit() {
                            *digits
                        } else {
                            *letters
                        },
                    )?;
                }
            }
        }
    }
    if !separate_leading {
        let first = body
            .first_mut()
            .ok_or_else(|| bibliography_error("empty assembled citation"))?;
        if first.properties != leading.properties {
            return Err(bibliography_error(
                "inconsistent citation leading-run properties",
            ));
        }
        let Some(rdocx_oxml::text::RunContent::Text(content)) = first.content.first_mut() else {
            return Err(bibliography_error("invalid citation leading run"));
        };
        content.text.insert(0, ' ');
        content.preserve_space = true;
        return Ok(body);
    }
    let mut runs = vec![leading];
    runs.extend(body);
    Ok(runs)
}

// Literal labels and punctuation measured in the authenticated APA6 locale sweep.
// The caller supplies country, patent number and year from its source record.
fn apa_plain_patent_punctuation(
    locale: &BibliographyFormattingLocale,
) -> Option<(&'static str, &'static str)> {
    use NativeCitationRun::{Country, Literal, Number, Year};
    match apa_patent_run_grammar(locale)?.1 {
        [
            Literal("(", 0),
            Country(0),
            Literal(label, 0),
            Number {
                letters: 0,
                digits: 0,
            },
            Literal(separator, 0),
            Year(0),
            Literal(")", 0),
        ] => Some((*label, *separator)),
        _ => None,
    }
}

fn apa_citation_member(
    source: &BibliographySource,
    options: &CitationSourceOptions,
    patent_punctuation: Option<(&str, &str)>,
) -> String {
    use BibliographySourceKind as Kind;
    let role = citation_creator_role(source, BibliographyStyle::ApaSixthEdition);
    let mut names = source
        .contributors
        .iter()
        .filter(|contributor| role.is_some_and(|role| contributor.role == role))
        .flat_map(|author| match &author.value {
            BibliographyAuthor::Corporate(value) => vec![value.clone()],
            BibliographyAuthor::People(people) => people
                .iter()
                .filter_map(|person| {
                    person
                        .last
                        .first()
                        .or_else(|| person.first.first())
                        .filter(|name| !name.is_empty())
                        .cloned()
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let country = first_source_property(source, BibliographySourceField::CountryRegion);
    let patent = first_source_property(source, BibliographySourceField::PatentNumber);
    let patent_identity = source.kind == Kind::Patent && !country.is_empty() && !patent.is_empty();
    if patent_identity {
        let label = patent_punctuation.map_or(" Patent No. ", |(label, _)| label);
        names = vec![format!("{country}{label}{patent}")];
    }
    let title = first_source_property(source, BibliographySourceField::Title);
    let short_title = first_source_property(source, BibliographySourceField::ShortTitle);
    let year = first_source_property(source, BibliographySourceField::Year);
    let mut result = options.prefix.clone().unwrap_or_default();
    if options.suppress_author || names.is_empty() {
        let label = if short_title.is_empty() {
            title
        } else {
            short_title
        };
        result.push_str(if label.is_empty() && year.is_empty() {
            &source.tag
        } else {
            label
        });
    } else {
        match names.as_slice() {
            [name] => result.push_str(name),
            [left, right] => result.push_str(&format!("{left} & {right}")),
            names if names.len() <= 5 => {
                result.push_str(&names[..names.len() - 1].join(", "));
                result.push_str(&format!(
                    ", & {}",
                    names.last().expect("nonempty author list")
                ));
            }
            names => result.push_str(&format!("{}, et al.", names[0])),
        }
        if !patent_identity && !options.suppress_title && !short_title.is_empty() {
            result.push_str(&format!(", {short_title}"));
        } else if !options.suppress_year && year.is_empty() && !title.is_empty() {
            result.push_str(&format!(", {title}"));
        }
    }
    let displayed_year =
        if source.kind == Kind::InternetSite && year.is_empty() && !title.is_empty() {
            "n.d."
        } else {
            year
        };
    if !options.suppress_year && !displayed_year.is_empty() {
        if !result.is_empty() {
            result.push_str(if patent_identity {
                patent_punctuation.map_or(", ", |(_, separator)| separator)
            } else {
                ", "
            });
        }
        result.push_str(displayed_year);
    }
    if let Some(pages) = &options.pages
        && !pages.is_empty()
    {
        result.push_str(&format!(
            ", {} {pages}",
            if pages.contains(['-', '–']) {
                "pp."
            } else {
                "p."
            }
        ));
    }
    if let Some(volume) = &options.volume
        && !volume.is_empty()
    {
        result.push_str(&format!(", vol. {volume}"));
    }
    if let Some(suffix) = &options.suffix {
        result.push_str(suffix);
    }
    result
}

type BibliographySortKey = (Vec<(String, String, String)>, String, String);

fn bibliography_sort_key(
    source: &BibliographySource,
    style: BibliographyStyle,
    locale: &BibliographyFormattingLocale,
) -> Result<BibliographySortKey> {
    let full_names = style == BibliographyStyle::ApaSixthEdition
        && apa_bibliography_language(locale.clone())?.full_names;
    let mut people = Vec::new();
    let role = if style == BibliographyStyle::ApaSixthEdition {
        match source.kind {
            BibliographySourceKind::Patent => BibliographyContributorRole::Inventor,
            BibliographySourceKind::Art => BibliographyContributorRole::Artist,
            BibliographySourceKind::Interview => BibliographyContributorRole::Interviewee,
            BibliographySourceKind::Film => BibliographyContributorRole::ProducerName,
            BibliographySourceKind::SoundRecording => BibliographyContributorRole::Composer,
            BibliographySourceKind::Performance => BibliographyContributorRole::Writer,
            _ => BibliographyContributorRole::Author,
        }
    } else {
        BibliographyContributorRole::Author
    };
    for contributor in &source.contributors {
        if contributor.role != role {
            continue;
        }
        match &contributor.value {
            BibliographyAuthor::Corporate(value) => {
                people.push((value.to_lowercase(), String::new(), String::new()))
            }
            BibliographyAuthor::People(names) => {
                for person in names {
                    if full_names {
                        let name = [&person.first, &person.middle, &person.last]
                            .into_iter()
                            .flat_map(|parts| parts.iter())
                            .filter(|part| !part.is_empty())
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" ");
                        people.push((name.to_lowercase(), String::new(), String::new()));
                    } else {
                        people.push((
                            person.last.join(" ").to_lowercase(),
                            person.first.join(" ").to_lowercase(),
                            person.middle.join(" ").to_lowercase(),
                        ));
                    }
                }
            }
        }
    }
    let title = first_source_property(source, BibliographySourceField::Title).to_lowercase();
    if people.is_empty()
        || style == BibliographyStyle::GostTitle
        || (style == BibliographyStyle::ApaSixthEdition
            && source.kind == BibliographySourceKind::Case)
    {
        people = vec![(title.clone(), String::new(), String::new())];
    }
    let year = first_source_property(source, BibliographySourceField::Year).to_owned();
    if !people
        .iter()
        .all(|(last, first, middle)| last.is_ascii() && first.is_ascii() && middle.is_ascii())
        || !year.is_ascii()
        || !title.is_ascii()
    {
        return Err(bibliography_error(
            "catalogued bibliography source-key collation branch is still being implemented",
        ));
    }
    Ok((people, year, title))
}

fn ieee_bibliography_table(
    entries: Vec<(u32, rdocx_oxml::text::CT_P)>,
    label_width: i32,
    text_width: i32,
) -> Result<Vec<rdocx_oxml::document::BodyContent>> {
    use rdocx_oxml::document::BodyContent;
    use rdocx_oxml::table::{
        CT_Row, CT_Tbl, CT_TblCellMar, CT_TblGrid, CT_TblGridCol, CT_TblLook, CT_TblPr,
        CT_TblWidth, CT_Tc, CT_TcPr, CT_TrPr, CellContent,
    };
    use rdocx_oxml::text::CT_P;
    use rdocx_oxml::units::Twips;
    let total = text_width
        .checked_add(90)
        .ok_or_else(|| bibliography_error("bibliography table width overflow"))?;
    let body_width = total
        .checked_sub(label_width)
        .filter(|width| *width > 0)
        .ok_or_else(|| bibliography_error("bibliography label exceeds the available text width"))?;
    let mut table = CT_Tbl::new();
    table.properties = Some(CT_TblPr {
        width: Some(CT_TblWidth::pct(5000)),
        cell_spacing: Some(CT_TblWidth::dxa(15)),
        cell_margin: Some(CT_TblCellMar {
            top: Some(Twips(15)),
            bottom: Some(Twips(15)),
            left: Some(Twips(15)),
            right: Some(Twips(15)),
        }),
        look: Some(CT_TblLook {
            val: Some("04A0".into()),
            first_row: Some(true),
            last_row: Some(false),
            first_column: Some(true),
            last_column: Some(false),
            no_h_band: Some(false),
            no_v_band: Some(true),
        }),
        ..Default::default()
    });
    table.grid = Some(CT_TblGrid {
        columns: vec![
            CT_TblGridCol {
                width: Twips(label_width),
            },
            CT_TblGridCol {
                width: Twips(body_width),
            },
        ],
        grid_change_xml: None,
        extra_xml: Vec::new(),
    });
    for (index, (number, mut reference)) in entries.into_iter().enumerate() {
        let mut label = CT_P::new();
        label.properties = Some(bibliography_paragraph_properties(BibliographyStyle::Ieee));
        label.runs.push(bibliography_display_run(
            &format!("[{number}] "),
            false,
            false,
        ));
        if index != 0
            && let Some(properties) = label.properties.as_mut()
            && let Some(properties) = properties.rpr.as_mut()
        {
            properties.sz = None;
            properties.sz_cs = None;
        }
        if let Some(properties) = reference.properties.as_mut()
            && let Some(properties) = properties.rpr.as_mut()
        {
            properties.sz = None;
            properties.sz_cs = None;
        }
        let mut row = CT_Row::new();
        row.properties = Some(CT_TrPr {
            cell_spacing: Some(CT_TblWidth::dxa(15)),
            ..Default::default()
        });
        for (paragraph, width) in [
            (label, CT_TblWidth::pct(50)),
            (reference, CT_TblWidth::auto()),
        ] {
            row.cells.push(CT_Tc {
                properties: Some(CT_TcPr {
                    width: Some(width),
                    extra_xml: vec![(12, b"<w:hideMark/>".to_vec())],
                    ..Default::default()
                }),
                content: vec![CellContent::Paragraph(paragraph)],
                extra_xml: Vec::new(),
            });
        }
        table.rows.push(row);
    }
    let mut first = CT_P::new();
    first.properties = Some(rdocx_oxml::CT_PPr {
        rpr: Some(rdocx_oxml::CT_RPr {
            no_proof: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });
    let mut trailing = CT_P::new();
    trailing.properties = Some(rdocx_oxml::CT_PPr {
        rpr: Some(rdocx_oxml::CT_RPr {
            font_east_asia: Some("Times New Roman".into()),
            no_proof: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });
    Ok(vec![
        BodyContent::Paragraph(first),
        BodyContent::Table(table),
        BodyContent::Paragraph(trailing),
    ])
}

// Shared source-driven APA vocabulary. These fields are consumed by authored
// entries and by the existing secondary-contributor/retrieval callers.
#[derive(Clone, Copy)]
struct ApaBibliographyLanguage {
    initial_mark: char,
    rtl_cs_font: Option<&'static str>,
    day_first: bool,
    publication_month_first: bool,
    calendar_day_year: &'static str,
    author_pair: bool,
    missing_year: Option<&'static str>,
    contributor_separator: &'static str,
    conjunction: &'static str,
    date_prefix: &'static str,
    retrieval_date_after: bool,
    retrieval_after_prefix: &'static str,
    day_month: &'static str,
    retrieval_date_separator: &'static str,
    in_before_separator: &'static str,
    on_before_separator: &'static str,
    recorded_before_separator: &'static str,
    month_day: &'static str,

    month_year: &'static str,
    year_month: &'static str,
    date_suffix: &'static str,
    calendar_year_first: bool,
    year_day_first: bool,
    volume_before: bool,
    volume_separator: &'static str,
    volume_suffix: &'static str,
    volume: &'static str,
    pages: &'static str,
    language: Option<&'static str>,
    language_east_asia: Option<&'static str>,
    joined_names: bool,
    semantic_rtl: bool,
    numeric_cs_font: Option<&'static str>,
    literal_font: Option<&'static str>,
    literal_script: Option<(char, char)>,
    no_italic: bool,
    full_names: bool,
    rtl_labels: bool,
    location_separator: &'static str,
    language_bidi: Option<&'static str>,
    patent: &'static str,
    patent_suffix: &'static str,
    compiler: &'static str,
    interviewer: &'static str,
    in_label: &'static str,
    in_after: bool,
    in_separator: &'static str,
    producer: &'static str,
    writer: &'static str,
    director: &'static str,
    performer: &'static str,
    site_editor: &'static str,
    motion_picture: &'static str,
    recorded_by: &'static str,
    recorded_after: bool,
    recorded_separator: &'static str,
    recorded_prefix: &'static str,
    on_label: &'static str,
    on_after: bool,
    on_separator: &'static str,
    quote_album: bool,
    curly_titles: bool,
    editor: &'static str,
    editors: &'static str,
    translator: &'static str,
    translators: &'static str,
    edition: &'static str,
    edition_separator: &'static str,
    edition_before: bool,
    edition_suffix: &'static str,
    retrieved: &'static str,
    retrieved_without_date: Option<&'static str>,
    from_without_date: &'static str,
    from_with_date: &'static str,
    retrieval_suffix: &'static str,
}

const APA_BIBLIOGRAPHY_ITALIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    initial_mark: '.',
    rtl_cs_font: None,
    in_after: false,
    on_after: false,
    retrieval_suffix: "",
    author_pair: false,
    missing_year: Some("s.d."),
    publication_month_first: false,
    calendar_day_year: ", ",
    day_first: false,
    contributor_separator: ", ",
    conjunction: "&",
    date_prefix: "",
    retrieval_date_after: false,
    day_month: " ",
    month_year: " ",
    year_month: ", ",
    date_suffix: "",
    calendar_year_first: false,
    year_day_first: false,
    volume_before: true,
    volume_separator: " ",
    volume: "Vol.",
    pages: "p. ",
    retrieval_after_prefix: " ",
    recorded_separator: " ",
    on_separator: " ",
    in_separator: " ",
    patent_suffix: "",
    recorded_prefix: "",
    retrieval_date_separator: " ",
    in_before_separator: " ",
    on_before_separator: " ",
    recorded_before_separator: " ",
    month_day: " ",
    semantic_rtl: false,
    numeric_cs_font: None,
    language_east_asia: None,
    joined_names: false,
    volume_suffix: "",
    literal_font: None,
    literal_script: None,
    no_italic: false,
    retrieved_without_date: None,
    full_names: false,
    rtl_labels: false,
    location_separator: ", ",
    language_bidi: None,
    language: Some("it-IT"),
    patent: "Brevetto n. ",
    compiler: "Redatto da",
    interviewer: "Intervistatore",
    in_label: "In",
    producer: "Produttore",
    writer: "Scrittore",
    director: "Regia",
    performer: "Artista",
    site_editor: "A cura di",
    motion_picture: "Film",
    recorded_by: "Registrato da",
    recorded_after: false,
    on_label: "In",
    quote_album: false,
    curly_titles: false,
    editor: "A cura di",
    editors: "A cura di",
    translator: "Trad.",
    translators: "Trad.",
    edition: "ed.",
    edition_separator: " ",
    edition_suffix: "",
    edition_before: false,
    retrieved: " Tratto il giorno",
    from_without_date: " da ",
    from_with_date: " da ",
};

const APA_BIBLIOGRAPHY_GERMAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    initial_mark: '.',
    rtl_cs_font: None,
    in_after: false,
    on_after: false,
    retrieval_suffix: "",
    author_pair: false,
    missing_year: Some("kein Datum"),
    publication_month_first: false,
    calendar_day_year: ", ",
    day_first: true,
    contributor_separator: ", ",
    conjunction: "&",
    date_prefix: "",
    retrieval_date_after: false,
    day_month: ". ",
    month_year: " ",
    year_month: ", ",
    date_suffix: "",
    calendar_year_first: false,
    year_day_first: false,
    volume_before: true,
    volume_separator: " ",
    volume: "Bd.",
    pages: "S. ",
    retrieval_after_prefix: " ",
    recorded_separator: " ",
    on_separator: " ",
    in_separator: " ",
    patent_suffix: "",
    recorded_prefix: "",
    retrieval_date_separator: " ",
    in_before_separator: " ",
    on_before_separator: " ",
    recorded_before_separator: " ",
    month_day: " ",
    semantic_rtl: false,
    numeric_cs_font: None,
    language_east_asia: None,
    joined_names: false,
    volume_suffix: "",
    literal_font: None,
    literal_script: None,
    no_italic: false,
    retrieved_without_date: None,
    full_names: false,
    rtl_labels: false,
    location_separator: ", ",
    language_bidi: None,
    language: Some("de-DE"),
    patent: "Patentnr. ",
    compiler: "Redakteur",
    interviewer: "Interviewer",
    in_label: "In",
    producer: "Produzent",
    writer: "Autor",
    director: "Regisseur",
    performer: "Interpret",
    site_editor: "Herausgeber",
    motion_picture: "Kinofilm",
    recorded_by: "Aufgezeichnet von",
    recorded_after: false,
    on_label: "Auf",
    quote_album: false,
    curly_titles: false,
    editor: "Hrsg.",
    editors: "Hrsg.",
    translator: "Übers.",
    translators: "Übers.",
    edition: "Ausg.",
    edition_separator: " ",
    edition_suffix: "",
    edition_before: false,
    retrieved: " Abgerufen am",
    from_without_date: " von ",
    from_with_date: " von ",
};

const APA_BIBLIOGRAPHY_SPANISH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    initial_mark: '.',
    rtl_cs_font: None,
    in_after: false,
    on_after: false,
    retrieval_suffix: "",
    author_pair: false,
    missing_year: Some("s.f."),
    publication_month_first: false,
    calendar_day_year: ", ",
    day_first: true,
    contributor_separator: ", ",
    conjunction: "&",
    date_prefix: "",
    retrieval_date_after: false,
    day_month: " de ",
    month_year: " de ",
    year_month: ", ",
    date_suffix: "",
    calendar_year_first: false,
    year_day_first: false,
    volume_before: true,
    volume_separator: " ",
    volume: "Vol.",
    pages: "págs. ",
    retrieval_after_prefix: " ",
    recorded_separator: " ",
    on_separator: " ",
    in_separator: " ",
    patent_suffix: "",
    recorded_prefix: "",
    retrieval_date_separator: " ",
    in_before_separator: " ",
    on_before_separator: " ",
    recorded_before_separator: " ",
    month_day: " ",
    semantic_rtl: false,
    numeric_cs_font: None,
    language_east_asia: None,
    joined_names: false,
    volume_suffix: "",
    literal_font: None,
    literal_script: None,
    no_italic: false,
    retrieved_without_date: None,
    full_names: false,
    rtl_labels: false,
    location_separator: ", ",
    language_bidi: None,
    language: Some("es-ES"),
    patent: "Patente nº ",
    compiler: "Recopilador",
    interviewer: "Entrevistador",
    in_label: "En",
    producer: "Productor",
    writer: "Escritor",
    director: "Dirección",
    performer: "Intérprete",
    site_editor: "Editor",
    motion_picture: "Película",
    recorded_by: "Grabado por",
    recorded_after: false,
    on_label: "De",
    quote_album: false,
    curly_titles: false,
    editor: "Ed.",
    editors: "Edits.",
    translator: "Trad.",
    translators: "Trads.",
    edition: "ed.",
    edition_separator: " ",
    edition_suffix: "",
    edition_before: false,
    retrieved: " Recuperado el",
    from_without_date: " de ",
    from_with_date: ", de ",
};

const APA_BIBLIOGRAPHY_FRENCH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    initial_mark: '.',
    rtl_cs_font: None,
    in_after: false,
    on_after: false,
    retrieval_suffix: "",
    author_pair: false,
    missing_year: Some("s.d."),
    publication_month_first: false,
    calendar_day_year: ", ",
    day_first: false,
    contributor_separator: ", ",
    conjunction: "&",
    date_prefix: "",
    retrieval_date_after: false,
    day_month: " ",
    month_year: " ",
    year_month: ", ",
    date_suffix: "",
    calendar_year_first: false,
    year_day_first: false,
    volume_before: true,
    volume_separator: " ",
    volume: "Vol.",
    pages: "pp. ",
    retrieval_after_prefix: " ",
    recorded_separator: " ",
    on_separator: " ",
    in_separator: " ",
    patent_suffix: "",
    recorded_prefix: "",
    retrieval_date_separator: " ",
    in_before_separator: " ",
    on_before_separator: " ",
    recorded_before_separator: " ",
    month_day: " ",
    semantic_rtl: false,
    numeric_cs_font: None,
    language_east_asia: None,
    joined_names: false,
    volume_suffix: "",
    literal_font: None,
    literal_script: None,
    no_italic: false,
    retrieved_without_date: None,
    full_names: false,
    rtl_labels: false,
    location_separator: ", ",
    language_bidi: None,
    language: Some("fr-FR"),
    patent: "Brevet n°\u{a0}",
    compiler: "Compilateur",
    interviewer: "Intervieweur",
    in_label: "Dans",
    producer: "Producteur",
    writer: "Écrivain",
    director: "Réalisateur",
    performer: "Interprète",
    site_editor: "Éditeur",
    motion_picture: "Film",
    recorded_by: "Enregistré par",
    recorded_after: false,
    on_label: "Sur",
    quote_album: false,
    curly_titles: false,
    editor: "Éd.",
    editors: "Éds.",
    translator: "Trad.",
    translators: "Trads.",
    edition: "éd.",
    edition_separator: " ",
    edition_suffix: "",
    edition_before: true,
    retrieved: " Consulté le",
    from_without_date: " sur ",
    from_with_date: ", sur ",
};

const APA_BIBLIOGRAPHY_ENGLISH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    initial_mark: '.',
    rtl_cs_font: None,
    in_after: false,
    on_after: false,
    retrieval_suffix: "",
    author_pair: false,
    missing_year: Some("n.d."),
    publication_month_first: false,
    calendar_day_year: ", ",
    day_first: false,
    contributor_separator: ", ",
    conjunction: "&",
    date_prefix: "",
    retrieval_date_after: false,
    day_month: " ",
    month_year: " ",
    year_month: ", ",
    date_suffix: "",
    calendar_year_first: false,
    year_day_first: false,
    volume_before: true,
    volume_separator: " ",
    volume: "Vol.",
    pages: "pp. ",
    retrieval_after_prefix: " ",
    recorded_separator: " ",
    on_separator: " ",
    in_separator: " ",
    patent_suffix: "",
    recorded_prefix: "",
    retrieval_date_separator: " ",
    in_before_separator: " ",
    on_before_separator: " ",
    recorded_before_separator: " ",
    month_day: " ",
    semantic_rtl: false,
    numeric_cs_font: None,
    language_east_asia: None,
    joined_names: false,
    volume_suffix: "",
    literal_font: None,
    literal_script: None,
    no_italic: false,
    retrieved_without_date: None,
    full_names: false,
    rtl_labels: false,
    location_separator: ", ",
    language_bidi: None,
    language: Some("en-US"),
    patent: "Patent No. ",
    compiler: "Compiler",
    interviewer: "Interviewer",
    in_label: "In",
    producer: "Producer",
    writer: "Writer",
    director: "Director",
    performer: "Performer",
    site_editor: "Editor",
    motion_picture: "Motion Picture",
    recorded_by: "Recorded by",
    recorded_after: false,
    on_label: "On",
    quote_album: false,
    curly_titles: false,
    editor: "Ed.",
    editors: "Eds.",
    translator: "Trans.",
    translators: "Trans.",
    edition: "ed.",
    edition_separator: " ",
    edition_suffix: "",
    edition_before: false,
    retrieved: " Retrieved",
    from_without_date: " from ",
    from_with_date: ", from ",
};

const APA_BIBLIOGRAPHY_DUTCH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: Some("sd"),
    language: Some("nl-NL"),
    patent: "Patentnr. ",
    compiler: "Samensteller",
    interviewer: "Interviewer",
    in_label: "In",
    producer: "Producent",
    writer: "Auteur",
    director: "Regisseur",
    performer: "Uitvoerend artiest",
    site_editor: "Redacteur",
    motion_picture: "Film",
    recorded_by: "Geregistreerd door",
    on_label: "Op",
    editor: "Red.",
    editors: "Red.",
    translator: "Vert.",
    translators: "Vert.",
    retrieved: " Opgeroepen op",
    from_without_date: " van ",
    from_with_date: ", van ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_DANISH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: Some("u.d."),
    day_first: true,
    day_month: ". ",
    month_year: " ",
    volume: "Årg.",
    pages: "s. ",
    language: Some("da-DK"),
    patent: "Patentnr. ",
    compiler: "Redaktør",
    interviewer: "Interviewer",
    in_label: "I",
    producer: "Producer",
    writer: "Skribent",
    director: "Instruktør",
    performer: "Udøvende kunstner",
    site_editor: "Redaktør",
    motion_picture: "Film",
    recorded_by: "Registreret af",
    on_label: "På",
    editor: "Red.",
    editors: "Red.",
    translator: "Ovs.",
    translators: "Ovs.",
    edition: "udg.",
    retrieved: " Hentet",
    from_without_date: " fra ",
    from_with_date: " fra ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_PORTUGUESE: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: Some("s.d."),
    day_first: true,
    day_month: " de ",
    month_year: " de ",
    language: Some("pt-PT"),
    patent: "Patente Nº ",
    compiler: "Compilador",
    interviewer: "Entrevistador",
    in_label: "Em",
    producer: "Produtor",
    writer: "Escritor",
    director: "Realizador",
    performer: "Artista",
    motion_picture: "Filme",
    recorded_by: "gravado",
    recorded_after: true,
    on_label: "Em",
    editors: "Edits.",
    translator: "Trad.",
    translators: "Trads.",
    retrieved: " Obtido em",
    from_without_date: " de ",
    from_with_date: ", de ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_SWEDISH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: Some("u.d."),
    day_first: true,
    date_prefix: "den ",
    retrieval_date_after: true,
    day_month: " ",
    month_year: " ",
    pages: "ss. ",
    language: Some("sv-SE"),
    patent: "Patentnr ",
    compiler: "Sammanställare",
    interviewer: "Intervjuare",
    in_label: "i",
    producer: "Producent",
    writer: "Skribent",
    director: "Regissör",
    performer: "Artist",
    site_editor: "Redaktör",
    motion_picture: "Film",
    recorded_by: "Inspelat av",
    on_label: "På",
    editor: "Red.",
    editors: "Red.",
    translator: "Övers.",
    translators: "Övers.",
    edition: "uppl.",
    retrieved: " Hämtat från",
    from_without_date: " ",
    from_with_date: " ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_FINNISH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: Some("ei pvm"),
    contributor_separator: ";",
    volume: "Osa/vuosik.",
    pages: "ss. ",
    language: Some("fi-FI"),
    patent: "Patenttinro ",
    compiler: "Kokoaja",
    interviewer: "Haastattelija",
    in_label: "Teoksessa",
    producer: "Tuottaja",
    writer: "Kirjoittaja",
    director: "Ohjaaja",
    performer: "Esiintyjä",
    site_editor: "Toimittaja",
    motion_picture: "Elokuva",
    recorded_by: "Tallentanut",
    on_label: ",",
    editor: "Toim.",
    editors: "Toim.",
    translator: "Käänt.",
    translators: "Käänt.",
    edition: "p.",
    retrieved: " Haettu",
    from_without_date: " osoitteesta ",
    from_with_date: " osoitteesta ",
    ..APA_BIBLIOGRAPHY_DANISH
};

const APA_BIBLIOGRAPHY_CATALAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    day_first: true,
    day_month: " / ",
    month_year: " / ",
    pages: "p. ",
    language: Some("ca-ES"),
    patent: "Patent núm. ",
    compiler: "Compilador",
    interviewer: "Entrevistador",
    in_label: "A",
    producer: "Productor",
    writer: "Escriptor",
    director: "Director",
    performer: "Intèrpret",
    motion_picture: "Pel·lícula",
    recorded_by: "Alta feta per",
    on_label: "De",
    editor: "Ed.",
    // Plural labels require the separate count evidence, not singleton inference.
    editors: "",
    translator: "Trad.",
    translators: "",
    retrieved: " Consultat el",
    from_without_date: " a ",
    from_with_date: ", a ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_CZECH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    volume: "Sv.",
    pages: "stránky ",
    language: Some("cs-CZ"),
    patent: "Patent č. ",
    compiler: "Kompilátor",
    interviewer: "Tazatel",
    in_label: "V",
    producer: "Producent",
    writer: "Autor",
    director: "Režisér",
    performer: "Umělec",
    site_editor: "Redaktor",
    recorded_by: "Zaznamenal",
    on_label: "Na albu",
    editor: "Editor",
    editors: "",
    translator: "Překl.",
    translators: "",
    edition: "vyd.",
    edition_separator: ". ",
    retrieved: " Získáno",
    from_without_date: " z ",
    from_with_date: ", z ",
    ..APA_BIBLIOGRAPHY_DANISH
};

const APA_BIBLIOGRAPHY_GREEK: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    volume: "Τόμ.",
    pages: "σσ. ",
    language: Some("el-GR"),
    patent: "Ευρεσιτεχνία Αρ. ",
    compiler: "Συντάκτης",
    interviewer: "Δημοσιογράφος",
    in_label: "Στο",
    producer: "Παραγωγός",
    writer: "Συγγραφέας",
    director: "Σκηνοθέτης",
    performer: "Ερμηνευτής",
    site_editor: "Επιμελητής",
    motion_picture: "Ταινία",
    recorded_by: "Καταγράφηκε από το χρήστη",
    on_label: "Στο",
    editor: "Επιμ.",
    editors: "",
    translator: "Μεταφρ.",
    translators: "",
    edition: "εκδ.",
    retrieved: " Ανάκτηση",
    from_without_date: " από ",
    from_with_date: ", από ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_POLISH: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    conjunction: "i",
    volume: "Tom",
    pages: "strony ",
    language: Some("pl-PL"),
    patent: "Patent nr ",
    compiler: "Kompilator",
    interviewer: "Osoba przeprowadzająca wywiad",
    in_label: "W",
    producer: "Producent",
    writer: "Autor",
    director: "Reżyser",
    performer: "Wykonawca",
    site_editor: "Redaktor",
    motion_picture: "Film",
    recorded_by: "Nagrane przez:",
    on_label: "Na",
    editor: "Red.",
    editors: "",
    translator: "Tłum.",
    translators: "",
    edition: "wyd.",
    retrieved: " Pobrano",
    from_without_date: " z lokalizacji ",
    from_with_date: " z lokalizacji ",
    ..APA_BIBLIOGRAPHY_FRENCH
};

const APA_BIBLIOGRAPHY_BULGARIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    day_first: true,
    date_suffix: " r.",
    volume: "Том",
    pages: "стр. ",
    language: Some("bg-BG"),
    patent: "Патент № ",
    compiler: "Съставител",
    interviewer: "Интервюиращ",
    in_label: "От",
    producer: "Продуцент",
    writer: "Писател",
    director: "Режисьор",
    performer: "Изпълнител",
    site_editor: "Редактор",
    motion_picture: "Филм",
    recorded_by: "Записано от",
    on_label: "От",
    editor: "Ред.",
    editors: "",
    translator: "Прев.",
    translators: "",
    edition: "изд.",
    retrieved: " Изтеглено на",
    from_without_date: "",
    from_with_date: " r. от ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_HUNGARIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    year_month: ". ",
    calendar_year_first: true,
    volume_before: false,
    volume_separator: ". ",
    volume: "kötet",
    pages: "old.: ",
    language: Some("hu-HU"),
    patent: "Szabadalom száma: ",
    compiler: "Sajtó alá rendezte:",
    interviewer: "Kérdező:",
    in_label: "In",
    producer: "Producer",
    writer: "Szerző",
    director: "Rendező",
    performer: "Előadó",
    site_editor: "Szerkesztő:",
    motion_picture: "Film",
    recorded_by: "rögzítette:",
    on_label: "Album:",
    editor: "Szerk.",
    editors: "",
    translator: "Ford.",
    translators: "",
    edition: "kiad.",
    edition_separator: ". ",
    retrieved: " Letöltés dátuma:",
    from_without_date: "",
    from_with_date: ", forrás: ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_ICELANDIC: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    day_first: true,
    day_month: ". ",
    volume: "B.",
    pages: "bls. ",
    language: Some("is-IS"),
    patent: "Einkaleyfi nr. ",
    compiler: "Ritstjóri",
    interviewer: "Spyrill",
    in_label: "Í",
    producer: "Framleiðandi",
    writer: "Rithöfundur",
    director: "Leikstjóri",
    performer: "Flytjandi",
    site_editor: "Ritstjóri",
    motion_picture: "Kvikmynd",
    recorded_by: "Tekið upp af",
    on_label: "Á",
    editor: "Ritstj.",
    translator: "Þýð.",
    edition: "útg.",
    retrieved: " Sótt",
    from_with_date: " frá ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_NORWEGIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    pages: "ss. ",
    language: Some("nb-NO"),
    patent: "Patentnr. ",
    compiler: "Kompilator",
    interviewer: "Intervjuer",
    in_label: "I",
    producer: "Produsent",
    writer: "Forfatter",
    director: "Regissør",
    performer: "Artist",
    site_editor: "Redaktør",
    motion_picture: "Film",
    recorded_by: "Registrert av",
    on_label: "På",
    editor: "Red.",
    translator: "Overs.",
    edition: "utg.",
    edition_separator: ". ",
    retrieved: " Hentet",
    from_with_date: " fra ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_ROMANIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    pages: "pg. ",
    language: Some("ro-RO"),
    patent: "Brevet nr. ",
    compiler: "Redactor",
    interviewer: "Operator interviu",
    in_label: "În",
    producer: "Producător",
    writer: "Autor",
    director: "Regizor",
    performer: "Interpret",
    site_editor: "Editor",
    motion_picture: "Film",
    recorded_by: "Înregistrat de",
    on_label: "De pe",
    editor: "Ed.",
    translator: "Trad.",
    edition_before: true,
    retrieved: " Preluat pe",
    from_with_date: ", de pe ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_CROATIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    day_first: true,
    day_month: ". ",
    volume: "Svez.",
    pages: "str. ",
    language: Some("hr-HR"),
    patent: "Br. patenta ",
    compiler: "Kompilator",
    interviewer: "Ispitivač",
    in_label: "U",
    producer: "Producent",
    writer: "Pisac",
    director: "Režiser",
    performer: "Izvođač",
    site_editor: "Urednik",
    motion_picture: "Film",
    recorded_by: "Snimio",
    on_label: "Na",
    editor: "Ur.",
    translator: "Prev.",
    edition: "izd.",
    retrieved: " Preuzeto",
    from_with_date: " iz ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_SLOVAK: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    day_first: true,
    day_month: ". ",
    producer: "Producent",
    motion_picture: "Film",
    on_label: "Na",
    volume: "Zv.",
    pages: "s. ",
    language: Some("sk-SK"),
    patent: "Patent č. ",
    compiler: "Kompilátor",
    interviewer: "Dotazovateľ",
    in_label: "In",
    writer: "Autor",
    director: "Režisér",
    performer: "Umelec",
    site_editor: "Editor",
    recorded_by: "Zaznamenal:",
    editor: "Ed.",
    translator: "Prekl.",
    edition: "vyd.",
    edition_separator: ". ",
    retrieved: " Cit.",
    from_with_date: ". Dostupné na Internete: ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_SLOVENIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    day_first: true,
    day_month: ". ",
    producer: "Producent",
    motion_picture: "Film",
    on_label: "Na",
    volume: "Izv.",
    pages: "str. ",
    language: Some("sl-SI"),
    patent: "Št. patenta ",
    compiler: "Sestavljavec",
    interviewer: "Izpraševalec",
    in_label: "V",
    writer: "Pisatelj",
    director: "Režiser",
    performer: "Izvajalec",
    site_editor: "Urednik",
    recorded_by: "Avtor posnetka",
    editor: "Ured.",
    translator: "Prev.",
    edition: "izd.",
    retrieved: " Pridobljeno",
    from_with_date: " iz ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_LATVIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    calendar_year_first: true,
    year_day_first: true,
    year_month: ". gada ",
    day_month: ". ",
    volume: "Sēj.",
    pages: "lpp. ",
    language: Some("lv-LV"),
    patent: "Patenta nr. ",
    compiler: "Sastādītājs",
    interviewer: "Intervētājs",
    in_label: "",
    producer: "Producents",
    writer: "Rakstnieks",
    director: "Režisors",
    performer: "Izpildītājs",
    site_editor: "Redaktors",
    motion_picture: "Filma",
    recorded_by: "Ierakstīja",
    on_label: "",
    editor: "Red.",
    translator: "Tulk.",
    edition: "izd.",
    retrieved: " Ielādēts",
    from_with_date: " no ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_LITHUANIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    calendar_year_first: true,
    year_month: " m. ",
    date_suffix: " d.",
    volume: "T.",
    pages: "p. ",
    language: Some("lt-LT"),
    patent: "Patento Nr. ",
    compiler: "Sudarytojas",
    interviewer: "Interviu ėmėjas",
    in_label: "Esantis",
    producer: "Prodiuseris",
    writer: "Rašytojas",
    director: "Režisierius",
    performer: "Atlikėjas",
    site_editor: "Redaktorius",
    motion_picture: "Vaidybinis filmas",
    recorded_by: "Įrašyta:",
    on_label: "Esantis",
    editor: "Mont.",
    translator: "Vert.",
    edition: "leid.",
    retrieved: " Paimta",
    from_with_date: " iš ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_RUSSIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    day_first: true,
    volume: "Т.",
    patent: "Патент № ",
    producer: "Продюсер",
    site_editor: "Редактор",
    editor: "Ред.",
    date_suffix: " г.",
    pages: "стр. ",
    language: Some("ru-RU"),
    compiler: "Составитель",
    interviewer: "Интервьюер",
    in_label: "В",
    writer: "Писатель",
    director: "Режиссер",
    performer: "Исполнитель",
    motion_picture: "Кино",
    recorded_by: "Автор:",
    on_label: "На",
    translator: "Перев.",
    edition: "изд.",
    edition_before: true,
    retrieved: " Получено",
    from_with_date: " г., из ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

const APA_BIBLIOGRAPHY_UKRAINIAN: ApaBibliographyLanguage = ApaBibliographyLanguage {
    missing_year: None,
    editors: "",
    translators: "",
    from_without_date: "",
    day_first: true,
    volume: "Т.",
    patent: "Патент №",
    producer: "Продюсер",
    site_editor: "Редактор",
    editor: "Ред.",
    date_suffix: " p.",
    pages: "сс. ",
    language: Some("uk-UA"),
    compiler: "Компілятор",
    interviewer: "Інтерв'юер",
    in_label: "у",
    writer: "Письменник",
    director: "Режисер",
    performer: "Виконавець",
    motion_picture: "Фільм",
    recorded_by: "Автор запису:",
    on_label: "Альбом",
    quote_album: true,
    translator: "Перекл.",
    edition: "вид.",
    retrieved: " Отримано",
    from_with_date: " p. з ",
    ..APA_BIBLIOGRAPHY_ENGLISH
};

fn apa_bibliography_language(
    locale: BibliographyFormattingLocale,
) -> Result<ApaBibliographyLanguage> {
    Ok(match locale {
        BibliographyFormattingLocale::Numeric(1047) => ApaBibliographyLanguage {
            language: Some("rm-CH"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            day_month: ". ",
            month_year: " ",
            date_suffix: "",
            editor: "Ed.",
            translator: "Trans.",
            compiler: "Cumpilader",
            interviewer: "Intervistader",
            site_editor: "Editur",
            performer: "Actur",
            producer: "Producent",
            writer: "Scriptur",
            director: "Reschissur",
            in_label: "En",
            motion_picture: "Film",
            recorded_by: "Recorded by",
            on_label: "Sin",
            patent: "Patent Nr. ",
            retrieved: " Obtegnì",
            from_with_date: " da ",
            pages: "pp. ",
            edition: "ed.",
            edition_before: false,
            volume: "Vol.",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1052) => ApaBibliographyLanguage {
            language: Some("sq-AL"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Re.",
            translator: "Përkth.",
            compiler: "Hartuesi",
            interviewer: "Intervistuesi",
            site_editor: "Redaktori",
            performer: "Interpretuesi",
            producer: "Producenti",
            writer: "Shkrimtari",
            director: "Drejtuesi",
            in_label: "Në",
            motion_picture: "Filmi",
            recorded_by: "Regjistruar nga",
            on_label: "Më",
            patent: "Patenta nr. ",
            retrieved: " Gjetur",
            from_with_date: ", nga ",
            pages: "fv. ",
            edition: "bot. i",
            edition_before: true,
            volume: "Vëll. i",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1057) => ApaBibliographyLanguage {
            language: Some("id-ID"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Penyunt.",
            translator: "Penerj.",
            compiler: "Kompilator",
            interviewer: "Pewawancara",
            site_editor: "Editor",
            performer: "Pemain",
            producer: "Produser",
            writer: "Penulis",
            director: "Sutradara",
            in_label: "Dalam",
            motion_picture: "Gambar Hidup",
            recorded_by: "Direkam oleh",
            on_label: "Dalam",
            patent: "Paten No. ",
            retrieved: " Dipetik",
            from_with_date: ", dari ",
            pages: "hal. ",
            edition: "ed.",
            edition_before: false,
            volume: "Vol.",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1059) => ApaBibliographyLanguage {
            language: Some("be-BY"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            day_month: " ",
            month_year: " ",
            date_suffix: " г.",
            editor: "Рэд.",
            translator: "Перакл.",
            compiler: "Укладальнік",
            interviewer: "Інтэрв'юер",
            site_editor: "Рэдактар",
            performer: "Выканаўца",
            producer: "Прад'юсер",
            writer: "Пісьменнік",
            director: "Рэжысёр",
            in_label: "У",
            motion_picture: "Кіно",
            recorded_by: "Кiм запiсана:",
            on_label: "На",
            patent: "Патэнт № ",
            retrieved: " Адноўлена",
            from_with_date: " г., з ",
            pages: "ст-кі ",
            edition: "рэд.",
            edition_before: false,
            volume: "Том",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1061) => ApaBibliographyLanguage {
            language: Some("et-EE"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            day_month: ". ",
            month_year: " ",
            date_suffix: ". a.",
            editor: "Toim.",
            translator: "Tõlk.",
            compiler: "Koostaja",
            interviewer: "Intervjueerija",
            site_editor: "Toimetaja",
            performer: "Esitaja",
            producer: "Produtsent",
            writer: "Kirjanik",
            director: "Režissöör",
            in_label: "rmt:",
            motion_picture: "Film",
            recorded_by: "Salvestanud",
            on_label: "Albumil",
            patent: "Patent nr ",
            retrieved: " Kasutamise kuupäev:",
            from_with_date: ". a., allikas ",
            pages: "lk ",
            edition: "tr.",
            edition_before: false,
            volume: "Kd.",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2064) => ApaBibliographyLanguage {
            language: Some("it-CH"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            day_month: ". ",
            month_year: " ",
            date_suffix: "",
            editor: "A cura di",
            translator: "Trad.",
            compiler: "Redatto da",
            interviewer: "Intervistatore",
            site_editor: "A cura di",
            performer: "Artista",
            producer: "Produttore",
            writer: "Scrittore",
            director: "Regia",
            in_label: "In",
            motion_picture: "Film",
            recorded_by: "Registrato da",
            on_label: "In",
            patent: "Brevetto n. ",
            retrieved: " Tratto il giorno",
            from_with_date: " da ",
            pages: "p. ",
            edition: "ed.",
            edition_before: false,
            volume: "Vol.",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1083) => ApaBibliographyLanguage {
            language: Some("se-NO"),
            missing_year: Some("u.d."),
            editors: "",
            translators: "",
            from_without_date: " fra ",
            day_first: false,
            day_month: " ",
            month_year: ". b. ",
            date_suffix: "",
            publication_month_first: true,
            calendar_day_year: ". b. ",
            editor: "Red.",
            translator: "Overs.",
            compiler: "Kompilator",
            interviewer: "Intervjuer",
            site_editor: "Redaktør",
            performer: "Artist",
            producer: "Produsent",
            writer: "Forfatter",
            director: "Regissør",
            in_label: "I",
            motion_picture: "Film",
            recorded_by: "Registrert av",
            on_label: "På",
            patent: "Patentnr. ",
            retrieved: " Hentet",
            from_with_date: " fra ",
            pages: "ss. ",
            edition: "utg.",
            edition_separator: ". ",
            edition_before: false,
            volume: "Vol.",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1089) => ApaBibliographyLanguage {
            language: Some("sw-KE"),
            missing_year: Some("h.t."),
            editors: "",
            translators: "",
            from_without_date: " toka ",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            publication_month_first: true,
            calendar_day_year: ", ",
            editor: "Mhar.",
            translator: "Mfas.",
            compiler: "Mkusanyaji",
            interviewer: "Msaili",
            site_editor: "Kihariri",
            performer: "Muigizaji",
            producer: "Mtoaji",
            writer: "Mwandishi",
            director: "Mwongozaji",
            in_label: "Kwenye",
            motion_picture: "Picha ya Sinema",
            recorded_by: "Imerekodiwa kwa",
            on_label: "Washa",
            patent: "Hataza Na. ",
            retrieved_without_date: Some(" Imechukuliwa"),
            retrieved: " Imenukuliwa",
            from_with_date: " kutoka ",
            pages: "kur. ",
            edition: "Tol. la",
            edition_suffix: ".",
            edition_before: true,
            volume: "Juzuu",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1124) => ApaBibliographyLanguage {
            language: Some("fil-PH"),
            missing_year: Some("walang petsa"),
            editors: "",
            translators: "",
            from_without_date: " mula sa/kay ",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            publication_month_first: true,
            calendar_day_year: ", ",
            editor: "Ed.",
            translator: "Mga Tagas.",
            compiler: "Nag-ipon",
            interviewer: "tagapanayam",
            site_editor: "Editor",
            performer: "Performer",
            producer: "Producer",
            writer: "Manunulat",
            director: "Direktor",
            in_label: "In",
            motion_picture: "Pelikula",
            recorded_by: "Na-record ni",
            on_label: "On",
            patent: "Blg. ng Patent ",
            retrieved: " Ipinanumbalik",
            from_with_date: ", mula sa/kay ",
            pages: "pp. ",
            edition: "ed.",
            edition_before: false,
            volume: "Vol.",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1128) => ApaBibliographyLanguage {
            language: Some("ha-Latn-NG"),
            missing_year: Some("n.d."),
            editors: "",
            translators: "",
            from_without_date: " daga ",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            publication_month_first: true,
            calendar_day_year: ", ",
            editor: "Mai shiryawa",
            translator: "Masu fassarawa",
            compiler: "Mai tarawa",
            interviewer: "Mai bada intabiyu",
            site_editor: "Mai shiryawa",
            performer: "Mai Aikatawa",
            producer: "Mai shiryawa",
            writer: "Mai rubutawa",
            director: "Mai bada umurni",
            in_label: "Cikin",
            motion_picture: "Hoto Mai Motsi",
            recorded_by: "Wanda ya dauka",
            on_label: "Akan",
            patent: "Lambar takaddar izini ",
            retrieved_without_date: Some(" An maido"),
            retrieved: " An maida",
            from_with_date: ", daga ",
            pages: "pp. ",
            edition: "ed.",
            edition_before: false,
            volume: "Kashi",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1130) => ApaBibliographyLanguage {
            language: Some("yo-NG"),
            missing_year: Some("n.d."),
            editors: "",
            translators: "",
            from_without_date: " lati ",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            publication_month_first: true,
            calendar_day_year: ", ",
            editor: "OluṢatunkỌ",
            translator: "AwỌn olutumỌ",
            compiler: "Oluṣakojọ",
            interviewer: "Olubeere",
            site_editor: "Aṣàtúnṣe",
            performer: "OlumuṢe",
            producer: "Olugbejade",
            writer: "OlukỌwe",
            director: "Oludari",
            in_label: "Ninu",
            motion_picture: "Aworan ti nṢipopada",
            recorded_by: "Ti a ṣegbasilẹ nipasẹ",
            on_label: "Lori",
            patent: "NỌ. Aṣẹ ọja tita ",
            retrieved: " Mimupada",
            from_with_date: ", lati ",
            pages: "oju ewe ",
            edition: "ÌṢàtúntò",
            edition_suffix: "",
            edition_before: true,
            volume: "Agbejade",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1136) => ApaBibliographyLanguage {
            language: Some("ig-NG"),
            missing_year: Some("enweghị deeti"),
            editors: "",
            translators: "",
            from_without_date: " site na ",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            publication_month_first: true,
            calendar_day_year: ", ",
            editor: "Onye ndezi",
            translator: "Onye ntụgharị",
            compiler: "Onye ndekọ",
            interviewer: "ọjụ ajụjụ",
            site_editor: "Onye ndezi",
            performer: "Ihe mmemme",
            producer: "Onye nrụpụta",
            writer: "Ọdee",
            director: "Onye nduzi",
            in_label: "N'ime",
            motion_picture: "Eserese Ihe Onyonyo",
            recorded_by: "dekọrọ ya",
            recorded_after: true,
            on_label: "N'elu",
            patent: "Nọmba Ikike ",
            retrieved: " Eweghachitere",
            from_with_date: ", site na ",
            pages: "ibe ",
            edition: "Usoro",
            edition_suffix: "",
            edition_before: true,
            volume: "Olu",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1087) => ApaBibliographyLanguage {
            language: Some("kk-KZ"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            calendar_year_first: false,
            year_day_first: false,
            date_suffix: " ж.",
            day_month: " ",
            month_year: " ",
            editor: "Редакторлар",
            translator: "Аудармашы",
            compiler: "Құрастырушы",
            interviewer: "Сұхбат алушы",
            site_editor: "Редактор",
            performer: "Орындаушы",
            producer: "Продюсер",
            writer: "Жазушы",
            director: "Режиссер",
            motion_picture: "Фильм",
            patent: "Патент № ",
            edition: "бас.",
            edition_before: true,
            volume: "-том",
            volume_before: false,
            volume_separator: "",
            recorded_after: false,
            recorded_by: "Жазған:",
            recorded_separator: "",
            on_after: true,
            on_label: "альбомында",
            on_separator: " ",
            in_after: true,
            in_label: "ішінде",
            in_separator: " ",
            retrieved: "",
            from_with_date: " ",
            retrieval_date_after: true,
            retrieval_after_prefix: " ішінен алынған ",
            retrieval_suffix: " ж.",
            pages: "беттер ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1088) => ApaBibliographyLanguage {
            language: Some("ky-KG"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            calendar_year_first: false,
            year_day_first: false,
            date_suffix: "-ж.",
            day_month: "-",
            month_year: " ",
            editor: "Ред.",
            translator: "Котор.",
            compiler: "Түзүүчү",
            interviewer: "Интервью алуучу",
            site_editor: "Редактор",
            performer: "Аткаруучу",
            producer: "Продюсер",
            writer: "Жазуучу",
            director: "Режиссер",
            motion_picture: "Фильм",
            patent: "Патент № ",
            edition: "чыг.",
            edition_before: false,
            volume: "Том",
            volume_before: true,
            volume_separator: " ",
            recorded_after: true,
            recorded_by: "тарабынан жазылды",
            recorded_separator: " ",
            on_after: true,
            on_label: "да",
            on_separator: "",
            in_after: true,
            in_label: "де",
            in_separator: "",
            retrieved: "",
            from_with_date: "-ж. кайтарылган, ",
            retrieval_suffix: " дан",
            pages: "бет. ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1091) => ApaBibliographyLanguage {
            language: Some("uz-Latn-UZ"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: true,
            year_day_first: true,
            date_suffix: "",
            year_month: " yil ",
            day_month: "-",
            editor: "Muh.",
            translator: "Tarj.",
            compiler: "Kompilyator",
            interviewer: "Suhbatdosh",
            site_editor: "Muharrir",
            performer: "Ijrochi",
            producer: "Prodyuser",
            writer: "Yozuvchi",
            director: "Direktor",
            motion_picture: "Kinofilm",
            patent: "",
            patent_suffix: "-sonli patent",
            edition: "nash.",
            edition_before: false,
            volume: "-tom",
            volume_before: false,
            volume_separator: "",
            recorded_after: true,
            recorded_by: "-da yozib olindi",
            recorded_separator: "",
            on_after: true,
            on_label: "ustida",
            on_separator: " ",
            in_after: true,
            in_label: "da",
            in_separator: "",
            retrieved: "",
            from_with_date: " ",
            retrieval_date_after: true,
            retrieval_after_prefix: "dan ",
            retrieval_suffix: " tiklangan",
            pages: "bet. ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1104) => ApaBibliographyLanguage {
            language: Some("mn-MN"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: true,
            year_day_first: false,
            date_suffix: "",
            year_month: " оны ",
            day_month: " ",
            editor: "Хян.",
            translator: "Орч.",
            compiler: "Хөрвүүлэгч",
            interviewer: "Ярилцагч",
            site_editor: "Засварлагч",
            performer: "Гүйцэтгэгч",
            producer: "Найруулагч",
            writer: "Зохиолч",
            director: "Найруулагч",
            motion_picture: "Уран сайхны кино",
            patent: "Патентийн Д.д. ",
            edition: "хян.",
            edition_before: false,
            volume: "Б.",
            volume_before: true,
            volume_separator: " ",
            recorded_after: true,
            recorded_by: "бичигдсэн",
            recorded_separator: " ",
            on_after: true,
            on_label: "-д",
            on_separator: "",
            in_after: true,
            in_label: "-Д",
            in_separator: "",
            retrieved: " Гаргасан",
            from_with_date: ", ",
            retrieval_suffix: "-аас",
            pages: "хуудсд. ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1064) => ApaBibliographyLanguage {
            language: Some("tg-Cyrl-TJ"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "Муҳ.",
            translator: "Тарҷ.",
            compiler: "Мураттиб",
            interviewer: "Мусоҳиб",
            site_editor: "Муҳаррир",
            performer: "Иҷрокунанда",
            producer: "Продюсер",
            writer: "Нависанда",
            director: "Коргардон",
            motion_picture: "Кинофилм",
            in_label: "Дар",
            on_label: "Дар",
            recorded_by: "Сабт аз",
            patent: "Патенти рақ. ",
            retrieved: " Дастрас шудааст",
            from_with_date: " аз ",
            retrieval_suffix: "",
            pages: "саҳ. ",
            edition: "нашри",
            edition_separator: " ",
            edition_before: true,
            volume: "Ҷилд",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1066) => ApaBibliographyLanguage {
            language: Some("vi-VN"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "Biên tập viên",
            translator: "Dịch giả",
            compiler: "Người biên soạn",
            interviewer: "Người phỏng vấn",
            site_editor: "Biên tập viên",
            performer: "Diễn viên/Nghệ sĩ",
            producer: "Nhà sản xuất",
            writer: "Tác giả",
            director: "Đạo diễn",
            motion_picture: "Phim Điện Ảnh",
            in_label: "Trong",
            on_label: "Trong",
            recorded_by: "Đã ghi",
            patent: "Đăng ký Độc quyền Nhãn hiệu Số ",
            retrieved: " Đã truy lục",
            from_with_date: ", từ ",
            retrieval_suffix: "",
            pages: "trang ",
            edition: "lần xuất bản",
            edition_separator: " ",
            edition_before: true,
            volume: "Tập",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1071) => ApaBibliographyLanguage {
            language: Some("mk-MK"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "Ур.",
            translator: "Прев.",
            compiler: "Составувач",
            interviewer: "Водител на интервју",
            site_editor: "Уредник",
            performer: "Изведувач",
            producer: "Продуцент",
            writer: "Писател",
            director: "Режисер",
            motion_picture: "Филм",
            in_label: "Во",
            on_label: "Во",
            recorded_by: "Снимено од",
            patent: "Патент бр. ",
            retrieved: " Преземено",
            from_with_date: " од ",
            retrieval_suffix: "",
            pages: "стр. ",
            edition: "изд.",
            edition_separator: " ",
            edition_before: false,
            volume: "Том",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1076) => ApaBibliographyLanguage {
            language: Some("xh-ZA"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "Umhleli",
            translator: "Umguquli",
            compiler: "Umqulunqi",
            interviewer: "Owenza udliwanondlebe",
            site_editor: "Umhleli",
            performer: "Umdlali-qonga",
            producer: "Umphuhlisi",
            writer: "Umbhali",
            director: "Umbhexeshi",
            motion_picture: "Ifilimu",
            in_label: "e",
            on_label: "e",
            recorded_by: "IErekhodiweyo yi",
            patent: "Inomb. yepeyitenti ",
            retrieved: " Ifumene",
            from_with_date: ", kwi- ",
            retrieval_suffix: "",
            pages: "amaphepha ",
            edition: "ushicilelo",
            edition_separator: " ",
            edition_before: false,
            volume: "Ivolyum",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1078) => ApaBibliographyLanguage {
            language: Some("af-ZA"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "Red.",
            translator: "Verts.",
            compiler: "Samesteller",
            interviewer: "Onderhoudvoerder",
            site_editor: "Redigeerder",
            performer: "Uitv. kunstenaar",
            producer: "Vervaardiger",
            writer: "Skrywer",
            director: "Regisseur",
            motion_picture: "Rolprent",
            in_label: "In",
            on_label: "Op",
            recorded_by: "Opgeneem deur",
            patent: "Patentnr. ",
            retrieved: "",
            from_with_date: " van ",
            retrieval_suffix: " herwin",
            pages: "ble. ",
            edition: "uitg.",
            edition_separator: " ",
            edition_before: false,
            volume: "Vol.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1086) => ApaBibliographyLanguage {
            language: Some("ms-MY"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "Ed.",
            translator: "Penterjemah",
            compiler: "Penyusun",
            interviewer: "Penemu duga",
            site_editor: "Editor",
            performer: "Penghibur",
            producer: "Penerbit",
            writer: "Penulis",
            director: "Pengarah",
            motion_picture: "Wayang Gambar",
            in_label: "Dalam",
            on_label: "Pada",
            recorded_by: "Dirakam oleh",
            patent: "No. Paten ",
            retrieved: " Didapatkan",
            from_with_date: ", daripada ",
            retrieval_suffix: "",
            pages: "hlm. ",
            edition: "ed.",
            edition_separator: " ",
            edition_before: true,
            volume: "Jld.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1068) => ApaBibliographyLanguage {
            language: Some("az-Latn-AZ"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            recorded_after: true,
            recorded_separator: " ",
            recorded_by: "tərəfindən qeydə alınıb",
            editor: "Red.",
            translator: "Tərc.",
            compiler: "Tərtibçi",
            interviewer: "Müsahibə götürən",
            site_editor: "Redaktor",
            performer: "İcraçı",
            producer: "Prodüser",
            writer: "Müəllif",
            director: "Rejissor",
            motion_picture: "Kino Təsvir",
            in_label: "Mənbə:",
            on_label: "Mənbə:",
            patent: "Patent No. ",
            retrieved: "",
            from_with_date: " tarixində bu ",
            retrieval_suffix: " mənbədən tapılıb",
            pages: "ss. ",
            edition: "buraxılış",
            edition_separator: " ",
            edition_before: true,
            volume: "Cild",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1106) => ApaBibliographyLanguage {
            language: Some("cy-GB"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Gol.",
            translator: "Cyfieithydd",
            compiler: "Crynhöwr",
            interviewer: "Cyfwelydd",
            site_editor: "Golygydd",
            performer: "Perfformiwr",
            producer: "Cynhyrchydd",
            writer: "Ysgrifennwr",
            director: "Cyfarwyddwr",
            motion_picture: "Ffilm",
            in_label: "Yn",
            on_label: "Ar",
            recorded_by: "Recordiwyd gan",
            patent: "Rhif Patent ",
            retrieved: " Adferwyd",
            from_with_date: " o ",
            retrieval_suffix: "",
            pages: "tt. ",
            edition: "ed.",
            edition_separator: " ",
            edition_before: false,
            volume: "Cyfrol",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1110) => ApaBibliographyLanguage {
            language: Some("gl-ES"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            day_month: " de ",
            month_year: " de ",
            date_suffix: "",
            editor: "Ed.",
            translator: "Trad.",
            compiler: "Compilador",
            interviewer: "Entrevistador",
            site_editor: "Editor",
            performer: "Intérprete",
            producer: "Produtor",
            writer: "Escritor",
            director: "Director",
            motion_picture: "Película",
            in_label: "En",
            on_label: "De",
            recorded_by: "Rexistrado por",
            patent: "Patente núm. ",
            retrieved: " Obtido o",
            from_with_date: ", de ",
            retrieval_suffix: "",
            pages: "pp. ",
            edition: "ed.",
            edition_separator: " ",
            edition_before: false,
            volume: "Vol.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1132) => ApaBibliographyLanguage {
            language: Some("nso-ZA"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Morulaganyi.",
            translator: "mofêtolêdi",
            compiler: "Mokgoboketši",
            interviewer: "Modiradipoledišano",
            site_editor: "Serulaganyi",
            performer: "Modiragatši",
            producer: "Motšweletši",
            writer: "Mongwadi",
            director: "Molaodi",
            motion_picture: "Filimi",
            in_label: "Ka",
            on_label: "Ka",
            recorded_by: "E gatišitšwe ke",
            patent: "Nomoro ya Phatente No. ",
            retrieved: " Tšerwe",
            from_with_date: ", go ",
            retrieval_suffix: "",
            pages: "matlakala. ",
            edition: "kgatišo.",
            edition_separator: " ",
            edition_before: false,
            volume: "Bolumu.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1153) => ApaBibliographyLanguage {
            language: Some("mi-NZ"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            recorded_after: true,
            recorded_prefix: "Nā ",
            recorded_separator: " ",
            recorded_by: "i rīkoata",
            editor: "Kaiwhakatika",
            translator: "Kaiwhakamāori",
            compiler: "Kaiwhakahiato",
            interviewer: "Kaipatapātai",
            site_editor: "Kaiwhakatika",
            performer: "Kaiwhakaari",
            producer: "Kaihautū",
            writer: "Kaituhi",
            director: "Tumuaki",
            motion_picture: "Kiriata",
            in_label: "I roto",
            on_label: "I runga",
            patent: "Tau Tohu tiaki tenenga ",
            retrieved: " I tīkina",
            from_with_date: " mai ",
            retrieval_suffix: "",
            pages: "wh. ",
            edition: "Putanga",
            edition_separator: " ",
            edition_before: true,
            volume: "Pukapuka",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2068) => ApaBibliographyLanguage {
            language: Some("nn-NO"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Red.",
            translator: "Omset.",
            compiler: "Kompilator",
            interviewer: "Intervjuar",
            site_editor: "Redaktør",
            performer: "Artist",
            producer: "Produsent",
            writer: "Forfattar",
            director: "Regissør",
            motion_picture: "Film",
            in_label: "I",
            on_label: "På",
            recorded_by: "Innspelt av",
            patent: "Patentnr. ",
            retrieved: " Henta",
            from_with_date: " frå ",
            retrieval_suffix: "",
            pages: "ss. ",
            edition: ". utg.",
            edition_separator: "",
            edition_before: false,
            volume: "Vol.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2074) => ApaBibliographyLanguage {
            language: Some("sr-Latn-CS"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Ur.",
            translator: "Prev.",
            compiler: "Kompajler",
            interviewer: "Osoba koja intervjuiše",
            site_editor: "Urednik",
            performer: "Izvođač",
            producer: "Producent",
            writer: "Pisac",
            director: "Režiser",
            motion_picture: "Igrani film",
            in_label: "U",
            on_label: "Na",
            recorded_by: "Snimio",
            patent: "Br. patenta ",
            retrieved: " Preuzeto",
            from_with_date: " sa ",
            retrieval_suffix: "",
            pages: "str. ",
            edition: "izd.",
            edition_separator: " ",
            edition_before: false,
            volume: "T.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2108) => ApaBibliographyLanguage {
            language: Some("ga-IE"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Eag.",
            translator: "Aistr.",
            compiler: "Tiomsaitheoir",
            interviewer: "Agallóir",
            site_editor: "Eagarthóir",
            performer: "Taibheoir",
            producer: "Léiritheoir",
            writer: "Scríbhneoir",
            director: "Stiúrthóir",
            motion_picture: "Scannán",
            in_label: "I",
            on_label: "Ar",
            recorded_by: "Taifeadta ag",
            patent: "Uimh. Phaitinne: ",
            retrieved: " Aisghafa",
            from_with_date: " ó ",
            retrieval_suffix: "",
            pages: "lgh ",
            edition: "eag.",
            edition_separator: " ",
            edition_before: false,
            volume: "Iml.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2110) => ApaBibliographyLanguage {
            language: Some("ms-BN"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Ed.",
            translator: "Penterjemah",
            compiler: "Penyusun",
            interviewer: "Penemu duga",
            site_editor: "Editor",
            performer: "Penghibur",
            producer: "Penerbit",
            writer: "Penulis",
            director: "Pengarah",
            motion_picture: "Wayang Gambar",
            in_label: "Dalam",
            on_label: "Pada",
            recorded_by: "Dirakam oleh",
            patent: "No. Paten ",
            retrieved: " Didapatkan",
            from_with_date: ", daripada ",
            retrieval_suffix: "",
            pages: "hlm. ",
            edition: "ed.",
            edition_separator: " ",
            edition_before: true,
            volume: "Jil.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(3098) => ApaBibliographyLanguage {
            language: Some("sr-Cyrl-CS"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Ур.",
            translator: "Прев.",
            compiler: "Приређивач",
            interviewer: "Новинар",
            site_editor: "Уредник",
            performer: "Извођач",
            producer: "Продуцент",
            writer: "Писац",
            director: "Режисер",
            motion_picture: "Играни филм",
            in_label: "У",
            on_label: "На",
            recorded_by: "Снимио",
            patent: "Бр. патента ",
            retrieved: " Преузето",
            from_with_date: " са ",
            retrieval_suffix: "",
            pages: "стр. ",
            edition: "изд.",
            edition_separator: " ",
            edition_before: false,
            volume: "Т.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(5146) => ApaBibliographyLanguage {
            language: Some("bs-Latn-BA"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            day_month: ". ",
            month_year: " ",
            date_suffix: "",
            editor: "Ur.",
            translator: "Prev.",
            compiler: "Sakupio",
            interviewer: "Voditelj intervjua",
            site_editor: "Urednik",
            performer: "Izvođač",
            producer: "Producent",
            writer: "Pisac",
            director: "Režiser",
            motion_picture: "Film",
            in_label: "U",
            on_label: "Na",
            recorded_by: "Snimio(la)",
            patent: "Patent br. ",
            retrieved: " Preuzeto",
            from_with_date: " iz ",
            retrieval_suffix: "",
            pages: "str. ",
            edition: "izd.",
            edition_separator: " ",
            edition_before: false,
            volume: "Tom.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(21514) => ApaBibliographyLanguage {
            language: Some("es-US"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            day_month: " ",
            month_year: " ",
            date_suffix: "",
            editor: "Ed.",
            translator: "Trad.",
            compiler: "Recopilador",
            interviewer: "Entrevistador",
            site_editor: "Editor",
            performer: "Intérprete",
            producer: "Productor",
            writer: "Escritor",
            director: "Dirección",
            motion_picture: "Película",
            in_label: "En",
            on_label: "De",
            recorded_by: "Grabado por",
            patent: "Patente nº ",
            retrieved: " Recuperado el",
            from_with_date: ", de ",
            retrieval_suffix: "",
            pages: "págs. ",
            edition: "ed.",
            edition_separator: " ",
            edition_before: false,
            volume: "Vol.",
            volume_separator: " ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1055) => ApaBibliographyLanguage {
            language: Some("tr-TR"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: false,
            year_day_first: false,
            date_suffix: "",
            year_month: ", ",
            month_day: " ",
            editor: "Dü.",
            translator: "Çev.",
            compiler: "Derleyici",
            interviewer: "Röportaj Yapan",
            site_editor: "Editör",
            performer: "Hazırlayan",
            producer: "Prodüktör",
            writer: "Yazar",
            director: "Yöneten",
            motion_picture: "Sinema Filmi",
            patent: "Patent No. ",
            retrieved: "",
            retrieval_date_separator: " ",
            from_with_date: " tarihinde ",
            retrieval_suffix: " adresinden alındı",
            pages: "s. ",
            edition: "b.",
            edition_separator: " ",
            edition_before: false,
            volume: "Cilt",
            volume_separator: " ",
            volume_before: true,
            recorded_after: true,
            recorded_by: "tarafından kaydedildi",
            recorded_separator: " ",
            recorded_before_separator: "",
            on_after: true,
            on_label: "albümünde",
            on_separator: " ",
            on_before_separator: "",
            in_after: true,
            in_label: "içinde",
            in_separator: " ",
            in_before_separator: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1069) => ApaBibliographyLanguage {
            language: Some("eu-ES"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: true,
            year_day_first: false,
            date_suffix: "",
            year_month: ".eko ",
            month_day: "k ",
            editor: "Ed.",
            translator: "Itzul.",
            compiler: "Bildumaratzailea",
            interviewer: "Elkarrizketatzailea",
            site_editor: "Editorea",
            performer: "Antzezlea",
            producer: "Produktorea",
            writer: "Idazlea",
            director: "Zuzendaria",
            motion_picture: "Filma",
            patent: "Patent zk. ",
            retrieved: " Eskuratze-eguna:",
            retrieval_date_separator: " ",
            from_with_date: ". Iturria: ",
            retrieval_suffix: "",
            pages: "or. ",
            edition: ". ed.",
            edition_separator: "",
            edition_before: false,
            volume: ". bol.",
            volume_separator: "",
            volume_before: false,
            recorded_after: false,
            recorded_by: "Honek grabatua:",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "Albuma:",
            on_separator: "",
            on_before_separator: " ",
            in_after: true,
            in_label: "en",
            in_separator: " ",
            in_before_separator: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1074) => ApaBibliographyLanguage {
            language: Some("tn-ZA"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: false,
            year_day_first: false,
            date_suffix: "",
            year_month: ", ",
            month_day: " ",
            editor: "Motseleganyi",
            translator: "Mofetoledi",
            compiler: "Morulaganyi",
            interviewer: "Mmotsolotsi",
            site_editor: "Motseleganyi",
            performer: "Modiragatsi",
            producer: "Motlhagisi",
            writer: "Mokwadi",
            director: "Mokaedi",
            motion_picture: "Setshwantsho sa baesekopo",
            patent: "Nomoro ya Patente. ",
            retrieved: " Gogilwe",
            retrieval_date_separator: "",
            from_with_date: ", go tswa ",
            retrieval_suffix: "",
            pages: "dits. ",
            edition: "kgatiso",
            edition_separator: " ",
            edition_before: false,
            volume: "Bol.",
            volume_separator: " ",
            volume_before: true,
            recorded_after: false,
            recorded_by: "E rekotilwe ke",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "Mo",
            on_separator: "",
            on_before_separator: "",
            in_after: false,
            in_label: "Mo",
            in_separator: "",
            in_before_separator: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1077) => ApaBibliographyLanguage {
            language: Some("zu-ZA"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: false,
            year_day_first: false,
            date_suffix: "",
            year_month: ", ",
            month_day: " ",
            editor: "Umhl.",
            translator: "Abahum.",
            compiler: "Umdidiyeli",
            interviewer: "Ophethe Ingxoxo",
            site_editor: "Umhleli",
            performer: "Umdlali",
            producer: "Uphrojusa",
            writer: "Umlobi",
            director: "Umqondisi",
            motion_picture: "Isithombe Esinyakazayo",
            patent: "Inomb. Yephathenti ",
            retrieved: " Kubuyisiwe",
            retrieval_date_separator: " ",
            from_with_date: " ku-",
            retrieval_suffix: "",
            pages: "kk. ",
            edition: "uhl.",
            edition_separator: " ",
            edition_before: false,
            volume: "Umq.",
            volume_separator: " ",
            volume_before: true,
            recorded_after: false,
            recorded_by: "Irekhodwe u-",
            recorded_separator: "",
            recorded_before_separator: "",
            on_after: false,
            on_label: "Ku",
            on_separator: "",
            on_before_separator: " ",
            in_after: false,
            in_label: "Ku-",
            in_separator: "",
            in_before_separator: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1082) => ApaBibliographyLanguage {
            language: Some("mt-MT"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            calendar_year_first: false,
            year_day_first: false,
            date_suffix: "",
            day_month: " ta' ",
            month_year: " ",
            editor: "Ed.",
            translator: "Trad.",
            compiler: "Kompilatur",
            interviewer: "Intervistatur",
            site_editor: "Editur",
            performer: "Eżekutur",
            producer: "Produttur",
            writer: "Kittieb",
            director: "Direttur",
            motion_picture: "Film",
            patent: "Nru tal-Privattiva ",
            retrieved: " Irkupra",
            retrieval_date_separator: " ",
            from_with_date: ", minn ",
            retrieval_suffix: "",
            pages: "pp. ",
            edition: "ed.",
            edition_separator: " ",
            edition_before: false,
            volume: "Vol.",
            volume_separator: " ",
            volume_before: true,
            recorded_after: false,
            recorded_by: "Irrekordjat minn",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "Fuq",
            on_separator: "",
            on_before_separator: " ",
            in_after: false,
            in_label: "F'",
            in_separator: "",
            in_before_separator: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1090) => ApaBibliographyLanguage {
            language: Some("tk-TM"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: true,
            year_day_first: false,
            date_suffix: "",
            year_month: " ý. ",
            month_day: " ",
            editor: "Redaktor",
            translator: "Terj.",
            compiler: "Ýygnan",
            interviewer: "Söhbetdeş bolan",
            site_editor: "Redaktor",
            performer: "Ýerine ýetiriji",
            producer: "Prodýuser",
            writer: "Ýazyjy",
            director: "Režisýor",
            motion_picture: "Kino",
            patent: "Patent belgisi ",
            retrieved: " Alnan wagty",
            retrieval_date_separator: " ",
            from_with_date: " ýeri ",
            retrieval_suffix: "",
            pages: "Sah-lar. ",
            edition: "neşir",
            edition_separator: " ",
            edition_before: true,
            volume: "Tom",
            volume_separator: " ",
            volume_before: true,
            recorded_after: false,
            recorded_by: "Ýazga geçiren",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: true,
            on_label: "içinde",
            on_separator: " ",
            on_before_separator: "",
            in_after: true,
            in_label: "içinde",
            in_separator: " ",
            in_before_separator: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1092) => ApaBibliographyLanguage {
            language: Some("tt-RU"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: true,
            calendar_year_first: false,
            year_day_first: false,
            date_suffix: " г.",
            day_month: " ",
            month_year: " ",
            editor: "Мхр.",
            translator: "Тәрҗ.",
            compiler: "Төзүче",
            interviewer: "Интервью алучы",
            site_editor: "Редактор",
            performer: "Башкаручы",
            producer: "Продюсер",
            writer: "Язучы",
            director: "Режиссер",
            motion_picture: "Фильм",
            patent: "Патент № ",
            retrieved: " Алынды",
            retrieval_date_separator: " ",
            from_with_date: " г., моннан: ",
            retrieval_suffix: "",
            pages: "битләр ",
            edition: "Рив.:",
            edition_separator: " ",
            edition_before: true,
            volume: "Том",
            volume_separator: " ",
            volume_before: true,
            recorded_after: true,
            recorded_by: "белән язылган",
            recorded_separator: " ",
            recorded_before_separator: "",
            on_after: false,
            on_label: "Көне:",
            on_separator: "",
            on_before_separator: " ",
            in_after: false,
            in_label: "Урыны:",
            in_separator: "",
            in_before_separator: " ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2115) => ApaBibliographyLanguage {
            language: Some("uz-Cyrl-UZ"),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            day_first: false,
            calendar_year_first: true,
            year_day_first: true,
            date_suffix: "",
            year_month: " йил ",
            day_month: "-",
            editor: "Ed.",
            translator: "Trans.",
            compiler: "Compiler",
            interviewer: "Interviewer",
            site_editor: "Editor",
            performer: "Performer",
            producer: "Producer",
            writer: "Writer",
            director: "Director",
            motion_picture: "Motion Picture",
            patent: "Patent No. ",
            retrieved: " Retrieved",
            retrieval_date_separator: " ",
            from_with_date: ", from ",
            retrieval_suffix: "",
            pages: "pp. ",
            edition: "ed.",
            edition_separator: " ",
            edition_before: false,
            volume: "Vol.",
            volume_separator: " ",
            volume_before: true,
            recorded_after: false,
            recorded_by: "Recorded by",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "On",
            on_separator: "",
            on_before_separator: " ",
            in_after: false,
            in_label: "In",
            in_separator: "",
            in_before_separator: " ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1025) => ApaBibliographyLanguage {
            language: None,
            semantic_rtl: true,
            full_names: true,
            rtl_labels: true,
            location_separator: "، ",
            pages: "الصفحات ",
            in_label: "تأليف",
            producer: "المنتج",
            writer: "الكاتب",
            director: "المخرج",
            performer: "المؤدي",
            site_editor: "المحرر",
            compiler: "محول برمجي",
            interviewer: "المحاور",
            motion_picture: "فيلم سينمائي",
            recorded_by: "مسجل من قِبل",
            on_label: "من",
            patent: "رقم براءة الاختراع ",
            day_first: true,
            month_year: ", ",
            contributor_separator: "، ",
            conjunction: "و",
            edition: "الإصدار",
            edition_before: true,
            volume: "المجلد",
            volume_before: true,
            editor: "المحرر",
            editors: "المحررون",
            translator: "المترجمون",
            translators: "المترجمون",
            retrieved: " تاريخ الاسترداد",
            from_with_date: "، من ",
            from_without_date: " من ",
            missing_year: None,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1028 | 3076 | 5124) => ApaBibliographyLanguage {
            language: None,
            language_east_asia: Some(match locale {
                BibliographyFormattingLocale::Numeric(1028) => "zh-TW",
                BibliographyFormattingLocale::Numeric(3076) => "zh-HK",
                _ => "zh-MO",
            }),
            joined_names: true,
            literal_font: Some("MS Gothic"),
            literal_script: Some(('\u{3040}', '\u{9fff}')),
            no_italic: true,
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            calendar_year_first: true,
            year_month: "年",
            month_day: "月",
            date_suffix: "日",
            editor: "編者",
            translator: "譯者",
            compiler: "剪輯",
            interviewer: "採訪者",
            site_editor: "編者",
            performer: "演出者",
            producer: "製作人",
            writer: "作家",
            director: "導演",
            motion_picture: "動畫",
            patent: "專利號碼 ",
            pages: "頁 ",
            recorded_by: "錄製者為",
            on_label: "出自",
            in_label: "於",
            edition: "第",
            edition_suffix: " 版",
            volume: "第",
            volume_suffix: " 冊",
            retrieved: "",
            retrieval_date_separator: " ",
            from_with_date: " 擷取自 ",
            edition_before: true,
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1144 | 2052 | 4100) => ApaBibliographyLanguage {
            language: None,
            language_east_asia: Some(match locale {
                BibliographyFormattingLocale::Numeric(1144) => "ii-CN",
                BibliographyFormattingLocale::Numeric(2052) => "zh-CN",
                _ => "zh-SG",
            }),
            joined_names: true,
            literal_font: Some("MS Gothic"),
            literal_script: Some(('\u{3040}', '\u{9fff}')),
            no_italic: true,
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            calendar_year_first: true,
            year_month: "年",
            month_day: "月",
            date_suffix: "日",
            editor: "编辑",
            translator: "翻译",
            compiler: "编辑者",
            interviewer: "采访人",
            site_editor: "编辑者",
            performer: "演员",
            producer: "制片人",
            writer: "作者",
            director: "导演",
            motion_picture: "电影",
            patent: "专利号 ",
            pages: "页 ",
            recorded_by: "录音员:",
            on_label: "位于",
            in_label: "出处",
            edition: "版本",
            volume: "卷",
            retrieved: " 检索日期:",
            retrieval_date_separator: " ",
            from_with_date: "，来源: ",
            volume_before: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1152) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some("ug-CN"),
            semantic_rtl: true,
            full_names: true,
            rtl_labels: true,
            no_italic: true,
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            calendar_year_first: true,
            year_day_first: true,
            year_month: "-كۈنى-",
            day_month: "ئاينىڭ-",
            date_suffix: "يىلى",
            editor: "مۇھەررىر",
            translator: "تەرجىمە قىلغۇچى",
            compiler: "تۈزگۈچى",
            interviewer: "زىيارەت قىلغۇچى",
            site_editor: "مۇھەررىر",
            performer: "ئورۇنلىغۇچى",
            producer: "فىلىم ئىشلىگۈچى",
            writer: "يازغۇچى",
            director: "رېژىسسور",
            motion_picture: "مۇقەددىمە",
            patent: "پاتېنت نومۇرى: ",
            recorded_by: "خاتىرىلەندى",
            recorded_after: true,
            recorded_prefix: "",
            recorded_separator: " ",
            on_label: "دە",
            on_after: true,
            on_before_separator: " ",
            in_label: "ئىچىدە",
            in_after: true,
            in_separator: " ",
            edition: "نەشىرى",
            volume: "— توم",
            volume_before: false,
            retrieved: "",
            retrieval_date_separator: " ",
            from_with_date: " ",
            retrieval_suffix: " دىن ئېلىندى",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(
            2049 | 3073 | 4097 | 5121 | 6145 | 7169 | 8193 | 9217 | 10241 | 11265 | 12289 | 13313
            | 14337 | 15361 | 16385,
        ) => ApaBibliographyLanguage {
            // Own15 native records share the concrete1025 operations, preserving
            // each independently asserted bidi language.
            language_bidi: Some(match locale {
                BibliographyFormattingLocale::Numeric(2049) => "ar-IQ",
                BibliographyFormattingLocale::Numeric(3073) => "ar-EG",
                BibliographyFormattingLocale::Numeric(4097) => "ar-LY",
                BibliographyFormattingLocale::Numeric(5121) => "ar-DZ",
                BibliographyFormattingLocale::Numeric(6145) => "ar-MA",
                BibliographyFormattingLocale::Numeric(7169) => "ar-TN",
                BibliographyFormattingLocale::Numeric(8193) => "ar-OM",
                BibliographyFormattingLocale::Numeric(9217) => "ar-YE",
                BibliographyFormattingLocale::Numeric(10241) => "ar-SY",
                BibliographyFormattingLocale::Numeric(11265) => "ar-JO",
                BibliographyFormattingLocale::Numeric(12289) => "ar-LB",
                BibliographyFormattingLocale::Numeric(13313) => "ar-KW",
                BibliographyFormattingLocale::Numeric(14337) => "ar-AE",
                BibliographyFormattingLocale::Numeric(15361) => "ar-BH",
                _ => "ar-QA",
            }),
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1025))?
        },
        BibliographyFormattingLocale::Numeric(1041) => ApaBibliographyLanguage {
            language: None,
            language_east_asia: Some("ja-JP"),
            joined_names: true,
            literal_font: Some("MS Gothic"),
            literal_script: Some(('\u{3040}', '\u{9fff}')),
            no_italic: true,
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            calendar_year_first: true,
            year_month: "年",
            month_day: "月",
            date_suffix: "日",
            editor: "編",
            translator: "訳",
            compiler: "編者",
            interviewer: "インタビュー質問者",
            site_editor: "編集者",
            performer: "出演者/演奏者",
            producer: "プロデューサー",
            writer: "著者",
            director: "監督",
            motion_picture: "映画",
            patent: "特許番号: ",
            pages: "ページ: ",
            recorded_by: "録音担当:",
            on_label: "アルバム:",
            in_label: "著:",
            conjunction: "",
            edition: "第",
            edition_before: true,
            edition_suffix: " 版",
            volume: "第",
            volume_before: true,
            volume_separator: " ",
            volume_suffix: " 巻",
            retrieved: " 参照日:",
            from_with_date: ", 参照先: ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1042) => ApaBibliographyLanguage {
            language: None,
            language_east_asia: Some("ko-KR"),
            joined_names: true,
            literal_font: Some("Malgun Gothic"),
            literal_script: Some(('\u{ac00}', '\u{d7af}')),
            no_italic: true,
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            calendar_year_first: true,
            year_month: "년 ",
            month_day: "월 ",
            date_suffix: "일",
            editor: "편집자",
            translator: "역자",
            compiler: "편찬자",
            interviewer: "질문자",
            site_editor: "편집자",
            performer: "연주자",
            producer: "제작자",
            writer: "작가",
            director: "영화 감독",
            motion_picture: "영화",
            patent: "특허권 번호: ",
            pages: "페이지: ",
            recorded_by: "녹음:",
            on_label: "",
            in_label: ",",
            edition: "판",
            edition_separator: "",
            volume: "제",
            volume_before: true,
            retrieved: " ",
            from_with_date: "",
            retrieval_date_after: true,
            retrieval_after_prefix: "에서 검색된 날짜: ",
            quote_album: true,
            curly_titles: true,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1067) => ApaBibliographyLanguage {
            language: Some("hy-AM"),
            literal_font: Some("Tahoma"),
            literal_script: Some(('\u{530}', '\u{58f}')),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "Խմբ.",
            translator: "Թարգ.",
            compiler: "Կազմարկող",
            interviewer: "Զրուցավար",
            site_editor: "Խմբագիր",
            performer: "Կատարող",
            producer: "Արտադրող",
            writer: "Գրող",
            director: "Տնօրեն",
            motion_picture: "Շարժանկար",
            patent: "Արտոնագիր հմ. ",
            retrieved: " Առբերված է",
            from_with_date: "-ին, ",
            retrieval_suffix: "-ից",
            edition: "խմբ.",
            volume: "Հատոր",
            volume_before: true,
            pages: "էջեր ",
            in_after: true,
            in_label: "ի մոտ",
            in_separator: "",
            in_before_separator: "",
            recorded_after: true,
            recorded_by: "-ի կողմից",
            recorded_prefix: "Գրանցված է ",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: true,
            on_label: "-ի վրա",
            on_separator: "",
            on_before_separator: "",
            day_first: true,
            day_month: " ",
            month_year: ", ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1079) => ApaBibliographyLanguage {
            language: Some("ka-GE"),
            literal_font: Some("Helvetica"),
            literal_script: Some(('\u{10a0}', '\u{10ff}')),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "რედ.",
            translator: "თარჯ.",
            compiler: "შემდგენელი",
            interviewer: "ინტერვიუერი",
            site_editor: "რედაქტორი",
            performer: "შემსრულებელი",
            producer: "პროდიუსერი",
            writer: "მწერალი",
            director: "რეჟისორი",
            motion_picture: "მოძრავი სურათი",
            patent: "პატენტის ნომ. ",
            retrieved: " მოპოვებული",
            from_with_date: ", ",
            retrieval_suffix: "-დან",
            edition: "გამ.",
            volume: "ხმა",
            volume_before: true,
            pages: "გვ. ",
            in_after: true,
            in_label: "-ში",
            in_separator: "",
            in_before_separator: "",
            recorded_after: false,
            recorded_by: "ჩაწერილია:",
            recorded_prefix: "",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: true,
            on_label: "-ზე",
            on_separator: "",
            on_before_separator: "",
            year_day_first: true,
            calendar_year_first: true,
            year_month: " წლის ",
            day_month: " ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1116) => ApaBibliographyLanguage {
            language: Some("chr-Cher-US"),
            literal_font: Some("Plantagenet Cherokee"),
            literal_script: Some(('\u{13a0}', '\u{13ff}')),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "ᎦᏁᏟᏴᏍᎩ",
            translator: "ᏗᎾᏁᎸᏗᏍᎩ",
            compiler: "ᏗᎦᏒᏛᏍᎩ",
            interviewer: "ᎩᎶ ᎠᏟᏃᎮᏙᏗ",
            site_editor: "ᏚᏳᎪᏛ",
            performer: "ᎠᏛᏁᎵᏍᎩ",
            producer: "ᏗᎪᏢᏍᎩ",
            writer: "ᎪᏪᎵᏍᎩ",
            director: "ᎠᏓᏎᎮᎯ",
            motion_picture: "ᎠᏓᏅᏏᏙᎯ ᏗᏟᎶᏍᏔᏅ",
            patent: "ᎠᏤᏝᏅ ᏗᏎᏍᏗ. ",
            retrieved: " ᏮᎤᎩᏒᎢ",
            from_with_date: ", ᎾᎿ ",
            retrieval_suffix: "",
            edition: "ᏗᎪᏛᏔᏅᎯ",
            volume: "ᎤᏍ.",
            volume_before: true,
            pages: "ᏚᎦᏅᏓᏛᎢ ",
            in_after: false,
            in_label: "ᎿᎾ",
            in_separator: "",
            in_before_separator: " ",
            recorded_after: false,
            recorded_by: "ᎤᏃᏪᎳᏅ",
            recorded_prefix: "",
            recorded_separator: "",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "ᎾᎿ",
            on_separator: "",
            on_before_separator: " ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1118) => ApaBibliographyLanguage {
            language: Some("am-ET"),
            literal_font: Some("Nyala"),
            literal_script: Some(('\u{1200}', '\u{137f}')),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "አዒ.",
            translator: "ተር.",
            compiler: "አጠናቃሪ",
            interviewer: "ቃለ መጠይቅ አድራጊ",
            site_editor: "አርታዒ",
            performer: "ተዋናይ",
            producer: "እዘጋጅ",
            writer: "ጸሓፊ",
            director: "አዘጋጅ",
            motion_picture: "ተንቀሳቃሽ ስእል",
            patent: "የፓ. ቁ. ",
            retrieved: " መረጃ የተገኘው",
            from_with_date: ", ከ ",
            retrieval_suffix: "",
            edition: "ሕት.",
            volume: "ይዘ.",
            volume_before: true,
            pages: "ገጾ. ",
            in_after: false,
            in_label: "በ",
            in_separator: "",
            in_before_separator: " ",
            recorded_after: true,
            recorded_by: "የተቀዳ",
            recorded_prefix: "በ ",
            recorded_separator: " ",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "ላይ",
            on_separator: "",
            on_before_separator: " ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1138) => ApaBibliographyLanguage {
            language: Some("om-ET"),
            literal_font: Some("Nyala"),
            literal_script: Some(('\u{1200}', '\u{137f}')),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "አዒ.",
            translator: "ተር.",
            compiler: "አጠናቃሪ",
            interviewer: "ቃለ መጠይቅ አድራጊ",
            site_editor: "አርታዒ",
            performer: "ተዋናይ",
            producer: "እዘጋጅ",
            writer: "ጸሓፊ",
            director: "አዘጋጅ",
            motion_picture: "ተንቀሳቃሽ ስእል",
            patent: "የፓ. ቁ. ",
            retrieved: " መረጃ የተገኘው",
            from_with_date: ", ከ ",
            retrieval_suffix: "",
            edition: "ሕት.",
            volume: "ይዘ.",
            volume_before: true,
            pages: "ገጾ. ",
            in_after: false,
            in_label: "በ",
            in_separator: "",
            in_before_separator: " ",
            recorded_after: true,
            recorded_by: "የተቀዳ",
            recorded_prefix: "በ ",
            recorded_separator: " ",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "ላይ",
            on_separator: "",
            on_before_separator: " ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1139) => ApaBibliographyLanguage {
            language: Some("ti-ET"),
            literal_font: Some("Nyala"),
            literal_script: Some(('\u{1200}', '\u{137f}')),
            missing_year: None,
            editors: "",
            translators: "",
            from_without_date: "",
            editor: "ኢዲ።",
            translator: "ተርጓሚ",
            compiler: "ኣሰናዳኢ",
            interviewer: "ሓታታይ",
            site_editor: "ኣርታዒ",
            performer: "ፈጻሚ",
            producer: "ኣዳላዊ",
            writer: "ጸሓፊ",
            director: "ዳይረክተር",
            motion_picture: "ተንቀሳቓሲ ስእሊ",
            patent: "ቁ. ፍቃድ ሓላፍነት ",
            retrieved: " ዝተበርበረሉ",
            from_with_date: "፣ ካብ ",
            retrieval_suffix: "",
            edition: "ሕት።",
            volume: "ሕት ቁ",
            volume_before: true,
            pages: "መ.ጸ። ",
            in_after: false,
            in_label: "ኣብ",
            in_separator: "",
            in_before_separator: " ",
            recorded_after: true,
            recorded_by: "ተቀዲሑ",
            recorded_prefix: "ብ ",
            recorded_separator: " ",
            recorded_before_separator: " ",
            on_after: false,
            on_label: "ብ",
            on_separator: "",
            on_before_separator: " ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2129) => ApaBibliographyLanguage {
            language: None,
            literal_font: Some("Microsoft Himalaya"),
            literal_script: Some(('\u{f00}', '\u{fff}')),
            no_italic: true,
            full_names: true,
            year_month: "ལོའི་ཟླ",
            month_day: "ཚེས",
            calendar_year_first: true,
            from_with_date: " from ",
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1094) => ApaBibliographyLanguage {
            language_bidi: Some("pa-IN"),
            numeric_cs_font: Some("Raavi"),
            literal_script: Some(('\u{a00}', '\u{a7f}')),
            editor: "ਸੰਪਾ.",
            translator: "ਅਨੁਵਾ.",
            compiler: "ਸੰਕਲਨਕਾਰ",
            interviewer: "ਇੰਟਰਵਿਊ ਲੈਣ ਵਾਲਾ",
            site_editor: "ਸੰਪਾਦਕ",
            performer: "ਅਦਾਕਾਰ",
            producer: "ਨਿਰਮਾਤਾ",
            writer: "ਲੇਖਕ",
            director: "ਨਿਰਦੇਸ਼ਕ",
            motion_picture: "ਮੋਸ਼ਨ ਪਿਕਚਰ",
            patent: "ਪੇਟੰਟ ਨੰ ",
            edition: "ਸੰਸ.",
            volume: "ਜਿਲ.",
            pages: "ਪੰਨੇ ",
            in_label: "ਵਿੱਚ",
            in_after: true,
            recorded_by: "ਦੁਆਰਾ ਰਿਕੌਰਡ ਕੀਤਾ ਗਿਆ",
            on_label: "ਤੇ",
            retrieval_after_prefix: " ਤੋਂ ",
            retrieval_suffix: " ਪ੍ਰਾਪਤ ਕੀਤੀ",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1102))?
        },
        BibliographyFormattingLocale::Numeric(1095) => ApaBibliographyLanguage {
            language_bidi: Some("gu-IN"),
            numeric_cs_font: Some("Shruti"),
            literal_script: Some(('\u{a80}', '\u{aff}')),
            editor: "સંપા.",
            translator: "અનુ.",
            compiler: "સંકલનકાર",
            interviewer: "ઇન્ટરવ્યૂ લેનાર",
            site_editor: "સંપાદક",
            performer: "અભિનેતા",
            producer: "નિર્માતા",
            writer: "લેખક",
            director: "દિગ્દર્શક",
            motion_picture: "ચાલતુ ચિત્ર",
            patent: "પેટન્ટ નં. ",
            edition: "આવૃત્તિ",
            volume: "વૉલ્યૂમ",
            pages: "પૃ. ",
            in_label: "માં",
            recorded_by: "દ્વારા રેકોર્ડ કરાયું",
            on_label: "પર",
            retrieval_after_prefix: " માંથી ",
            retrieval_suffix: " પ્રાપ્ત",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1081))?
        },
        BibliographyFormattingLocale::Numeric(1096) => ApaBibliographyLanguage {
            language_bidi: Some("or-IN"),
            numeric_cs_font: Some("Arial Unicode MS"),
            literal_script: Some(('\u{b00}', '\u{b7f}')),
            editor: "ସମ୍ପାଦକ",
            translator: "ଅନୁବାଦକ",
            compiler: "ସଙ୍କଳକ",
            interviewer: "ସାକ୍ଷାତକାରକ",
            site_editor: "ସମ୍ପାଦକ",
            performer: "ନିଷ୍ପାଦକ",
            producer: "ଉତ୍ପାଦକ",
            writer: "ଲେଖକ",
            director: "ନିର୍ଦ୍ଦେଶକ",
            motion_picture: "ସଚଳ ଛବି",
            patent: "ପେଟେଣ୍ଟ କ୍ର. ",
            edition: "ସଂ.",
            volume: "ଭଲ୍ୟୁମ୍.",
            pages: "ପୃ. ",
            in_label: "ଭିତର",
            recorded_by: "ଦ୍ଵାରା ରେକର୍ଡ୍ ହୋଇଛି",
            on_label: "ଉପର",
            retrieval_after_prefix: " ରୁ, ",
            retrieval_suffix: " ପୁନର୍ଲାଭ କଲା",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1081))?
        },
        BibliographyFormattingLocale::Numeric(1093) => ApaBibliographyLanguage {
            language_bidi: Some("bn-IN"),
            numeric_cs_font: Some("Vrinda"),
            literal_script: Some(('\u{980}', '\u{9ff}')),
            editor: "সম্পা.",
            translator: "অনুবাদক",
            compiler: "সঙ্কলকগণ",
            interviewer: "সাক্ষাতকার গ্রহীতা",
            site_editor: "সম্পাদক",
            performer: "শিল্পী",
            producer: "প্রযোজক",
            writer: "লেখক",
            director: "পরিচালক",
            motion_picture: "চলচ্চিত্র",
            patent: "পেটেন্ট নম্বর ",
            edition: "সংস্ক.",
            edition_before: true,
            volume: "সংখ্যা",
            pages: "পৃ. ",
            in_label: "-এর",
            in_separator: "",
            recorded_by: "-এর দ্বারা রেকর্ড করা",
            recorded_separator: "",
            on_label: "উপরে",
            retrieval_after_prefix: " থেকে ",
            retrieval_suffix: " উদ্ধার করা হয়েছে",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1081))?
        },
        BibliographyFormattingLocale::Numeric(1101) => ApaBibliographyLanguage {
            language_bidi: Some("as-IN"),
            numeric_cs_font: Some("Vrinda"),
            literal_script: Some(('\u{980}', '\u{9ff}')),
            full_names: true,
            publication_month_first: true,
            calendar_day_year: ", ",
            editor: "সম্পা.",
            translator: "অনুবা.",
            compiler: "সংকলক",
            interviewer: "সাক্ষাত্গ্ৰহণকাৰী",
            site_editor: "সম্পাদক",
            performer: "পৰিৱেশনকাৰী",
            producer: "নিৰ্মাতা",
            writer: "লিখক",
            director: "পৰিচালক",
            motion_picture: "চলচিত্ৰ",
            patent: "পেটেন্ট ক্ৰ. ",
            edition: "তাঙ.",
            volume: "ভলি.",
            pages: "পৃ. ",
            in_label: "ইয়াত",
            in_after: false,
            recorded_by: "-ৰ দ্বাৰা ৰেকৰ্ড কৰা",
            recorded_separator: "",
            on_label: "ওপৰত",
            on_after: false,
            retrieval_after_prefix: "-ৰ পৰা ",
            retrieval_suffix: " পুনঃপ্ৰাপ্তি কৰা হৈছে",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1102))?
        },
        BibliographyFormattingLocale::Numeric(1099) => ApaBibliographyLanguage {
            language_bidi: Some("kn-IN"),
            numeric_cs_font: Some("Tunga"),
            literal_script: Some(('\u{c80}', '\u{cff}')),
            editor: "ಸಂಪಾದಕ",
            translator: "ಭಾಷಾಂತರಗಾರ",
            compiler: "ಸಂಗ್ರಹಿಸಿದವರು",
            interviewer: "ಸಂದರ್ಶನಕಾರ",
            site_editor: "ಸಂಪಾದಕ",
            performer: "ಅಭಿನೇತೃ",
            producer: "ನಿರ್ಮಾಪಕ",
            writer: "ಬರಹಗಾರ",
            director: "ನಿರ್ದೇಶಕ",
            motion_picture: "ಚಲನ ಚಿತ್ರ",
            patent: "ಪೇಟೆಂಟ್ ಸಂಖ್ಯೆ. ",
            edition: "ಸಂಚಿಕೆ",
            volume: "ಸಂಪುಟ",
            pages: "ಪುಟ ",
            in_label: "ನಲ್ಲಿ",
            recorded_by: "ರಿಂದ ದಾಖಲಿಸಲಾಗಿದೆ",
            on_label: "ರಲ್ಲಿ",
            retrieval_after_prefix: " ನಿಂದ, ",
            retrieval_suffix: " ಪಡೆಯಲಾಗಿದೆ",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1081))?
        },
        BibliographyFormattingLocale::Numeric(1100) => ApaBibliographyLanguage {
            language_bidi: Some("ml-IN"),
            numeric_cs_font: Some("Arial Unicode MS"),
            literal_script: Some(('\u{d00}', '\u{d7f}')),
            editor: "എഡിറ്റര്‍",
            translator: "പരിഭാഷകര്‍",
            compiler: "സമാഹര്‍ത്താവ്",
            interviewer: "അഭിമുഖകാരന്‍",
            site_editor: "എഡിറ്റര്‍",
            performer: "കലാകാരന്‍",
            producer: "നിര്‍മ്മാതാവ്",
            writer: "രചയിതാവ്",
            director: "സംവിധായകന്‍",
            motion_picture: "ചലച്ചിത്രം",
            patent: "പേറ്റന്റ് നമ്പര്‍ ",
            edition: "പതിപ്പ്",
            volume: "വാല്യം",
            pages: "പേജുകള്‍ ",
            in_label: "-ലെ",
            in_after: true,
            in_separator: "",
            recorded_by: "റെക്കോര്‍ഡ് ചെയ്‌തത്",
            on_label: "-ല്‍",
            on_separator: "",
            retrieval_after_prefix: "-ല്‍ നിന്നും ",
            retrieval_suffix: " വീണ്ടെടുത്തു",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1102))?
        },
        BibliographyFormattingLocale::Numeric(1097) => ApaBibliographyLanguage {
            language_bidi: Some("ta-IN"),
            numeric_cs_font: Some("Latha"),
            literal_script: Some(('\u{b80}', '\u{bff}')),
            editor: "திருத்.",
            translator: "மொழிபெயர்.",
            compiler: "தொகுப்பாளர்",
            interviewer: "பேட்டியாளர்",
            site_editor: "பதிப்பாசிரியர்",
            performer: "நிகழ்த்துனர்",
            producer: "தயாரிப்பாளர்",
            writer: "எழுத்தாளர்",
            director: "இயக்குநர்",
            motion_picture: "சலனப்படக் காட்சி",
            patent: "காப்புரிமை எண் ",
            edition: "பதி.",
            volume: "தொகு.",
            pages: "பக். ",
            in_label: "இல்",
            in_after: true,
            recorded_by: "ஆல் பதிவுசெய்யப்பட்டது",
            on_label: "-இல்",
            retrieval_after_prefix: " -இல் இருந்து, ",
            retrieval_suffix: " எடுக்கப்பட்டது",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1102))?
        },
        BibliographyFormattingLocale::Numeric(1098) => ApaBibliographyLanguage {
            language_bidi: Some("te-IN"),
            numeric_cs_font: Some("Gautami"),
            literal_script: Some(('\u{c00}', '\u{c7f}')),
            editor: "సంపా.",
            translator: "అనువాద.",
            compiler: "సంకలనకర్త",
            interviewer: "ఇంటర్వ్యూ చేసే వ్యక్తి",
            site_editor: "సంపాదకుడు",
            performer: "ప్రదర్శకుడు",
            producer: "నిర్మాత",
            writer: "రచయిత",
            director: "దర్శకుడు",
            motion_picture: "మోషన్ పిక్చర్",
            patent: "పేటెంట్ సం. ",
            edition: "కూర్పు",
            volume: "పరిమాణము",
            pages: "పేజి ",
            in_label: "లో",
            in_after: true,
            recorded_by: "ద్వారా నమోదు చేయబడింది",
            on_label: "లో",
            retrieval_after_prefix: " నుంచి ",
            retrieval_suffix: " తిరిగి పొందబడింది",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1102))?
        },
        BibliographyFormattingLocale::Numeric(1103) => ApaBibliographyLanguage {
            language_bidi: Some("sa-IN"),
            from_with_date: ", from ",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(2144))?
        },
        BibliographyFormattingLocale::Numeric(1081) => ApaBibliographyLanguage {
            language_bidi: Some("hi-IN"),
            full_names: true,
            day_first: true,
            month_year: " ",
            editor: "सं.",
            translator: "अनु.",
            compiler: "कंपाइलर",
            interviewer: "साक्षात्कारकर्ता",
            site_editor: "संपादक",
            performer: "निष्पादक",
            producer: "निर्माता",
            writer: "लेखक",
            director: "निर्देशक",
            motion_picture: "चलचित्र",
            patent: "पेटेंट क्र. ",
            edition: "सं.",
            volume: "संस्क.",
            pages: "पृ. ",
            in_label: "में",
            in_after: true,
            recorded_by: "द्वारा रिकॉर्ड किया गया",
            recorded_after: true,
            on_label: "पर",
            on_after: true,
            retrieved: " ",
            from_with_date: "",
            from_without_date: "",
            retrieval_date_after: true,
            retrieval_after_prefix: " से, ",
            retrieval_suffix: " को पुनर्प्राप्त",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(2144))?
        },
        BibliographyFormattingLocale::Numeric(1102) => ApaBibliographyLanguage {
            language_bidi: Some("mr-IN"),
            editor: "संपा.",
            translator: "अनु.",
            compiler: "संग्राहक",
            interviewer: "मुलाखतकार",
            site_editor: "संपादक",
            performer: "सादरकर्ता",
            producer: "निर्माता",
            writer: "लेखक",
            director: "दिग्दर्शक",
            motion_picture: "चित्रपट",
            patent: "पेटंट क्र. ",
            edition: "आ.",
            volume: "भाग",
            pages: "पान ",
            in_label: "मध्ये",
            recorded_by: "द्वारे रेकॉर्ड केलेले",
            recorded_after: true,
            on_label: "वर",
            on_after: true,
            retrieved: " ",
            from_with_date: "",
            from_without_date: "",
            retrieval_date_after: true,
            retrieval_after_prefix: " मधून ",
            retrieval_suffix: " पुनर्प्राप्ती",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(2144))?
        },
        BibliographyFormattingLocale::Numeric(2144) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some("ks-Deva"),
            numeric_cs_font: Some("Mangal"),
            from_with_date: " from ",
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(lcid @ (1056 | 2080)) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some(if lcid == 1056 { "ur-PK" } else { "ur-IN" }),
            semantic_rtl: true,
            rtl_labels: true,
            day_first: true,
            month_year: ", ",
            missing_year: None,
            editors: "",
            translators: "",
            editor: "مدیر",
            translator: "ترجمہ نگار",
            compiler: "مؤلف",
            interviewer: "انٹرویو کنندہ",
            site_editor: "تدوین کار",
            performer: "تعمیل کنندہ",
            producer: "منتظم",
            writer: "مصنف",
            director: "ہدایت کار",
            motion_picture: "متحرک تصویر",
            patent: "پیٹینٹ نمبر ",
            edition: "ایڈیشن",
            volume: "جلد",
            volume_before: false,
            pages: "صفحہ ",
            in_label: "میں",
            recorded_by: "سے ریکارڈ",
            recorded_after: true,
            on_label: "پر",
            retrieved: " واپس، ",
            from_with_date: "",
            from_without_date: "",
            retrieval_date_after: true,
            retrieval_after_prefix: " سے ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1037) => ApaBibliographyLanguage {
            initial_mark: '\'',
            language: None,
            language_bidi: Some("he-IL"),
            semantic_rtl: true,
            rtl_labels: true,
            day_first: true,
            missing_year: None,
            editors: "",
            translators: "",
            editor: "עורך",
            translator: "מתרגמים",
            compiler: "יוצר האוסף",
            interviewer: "מראיין",
            site_editor: "עורך",
            performer: "מבצע",
            producer: "מפיק",
            writer: "סופר",
            director: "במאי",
            motion_picture: "סרט",
            patent: "פטנט מס' ",
            edition: "מהדורה",
            edition_before: true,
            volume: "כרך",
            pages: "עמ' ",
            in_label: "ב-",
            in_separator: "",
            recorded_by: "הוקלט על-ידי",
            on_label: "באלבום",
            retrieved: " אוחזר ב-",
            from_with_date: ", מתוך ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1065) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some("fa-IR"),
            semantic_rtl: true,
            rtl_labels: true,
            missing_year: None,
            editors: "",
            translators: "",
            editor: "تدوين",
            translator: "مترجم",
            compiler: "گرد آورنده",
            interviewer: "مصاحبه كننده",
            site_editor: "تدوين كننده",
            performer: "اجراكننده",
            producer: "تهيه كننده",
            writer: "نويسنده",
            director: "كارگردان",
            motion_picture: "تصوير متحرك",
            patent: "ش. حق انحصاري ",
            edition: "نسخه",
            edition_before: true,
            volume: "جلد",
            pages: "ص. ",
            in_label: "در",
            recorded_by: "ضبط شده توسط",
            on_label: "در",
            retrieved: " بازيابی در",
            from_with_date: "، از ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2137) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some("sd-Arab-PK"),
            semantic_rtl: true,
            rtl_labels: true,
            missing_year: None,
            editors: "",
            translators: "",
            editor: "ايڊيٽر",
            translator: "ترجمو ڪندڙ",
            compiler: "سھيندڙ",
            interviewer: "انٽرويو ڪندڙ",
            site_editor: "ايڊيٽر",
            performer: "ادا ڪندڙ",
            producer: "ٺاھيندڙ",
            writer: "ليکڪ",
            director: "ھدايت ڪار",
            motion_picture: "چرندڙ تصوير",
            patent: "ظاھري نمبر۔ ",
            edition: "ايڊيشن",
            volume: "آواز جي ڳوراڻ",
            pages: "صفحا ",
            in_label: "۾",
            recorded_by: "جي طرفان رڪارڊ ڪيل",
            recorded_after: true,
            on_label: "تي",
            retrieved: " حاصل ڪيو",
            from_with_date: "، کان ",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(2118) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some("pa-Arab-PK"),
            semantic_rtl: true,
            rtl_labels: true,
            missing_year: None,
            editors: "",
            translators: "",
            editor: "مدیران",
            translator: "مترجم",
            compiler: "ترتیب کار",
            interviewer: "انٹرویو کار",
            site_editor: "مدیر",
            performer: "فنکار",
            producer: "پیشکار",
            writer: "لکھاری",
            director: "ہدایتکار",
            motion_picture: "متحرک تصویراں",
            patent: "پیٹنٹ نمبر ",
            edition: "ایڈیشن",
            volume: "جلد",
            pages: "صفحے ",
            in_label: "میں",
            recorded_by: "توں ریکارڈ",
            recorded_after: true,
            on_label: "پر",
            retrieved: "",
            from_with_date: " وچوں ",
            retrieval_suffix: " لے لیا",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1123) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some("ps-AF"),
            semantic_rtl: true,
            rtl_labels: true,
            day_first: true,
            day_month: "/",
            month_year: "/",
            missing_year: None,
            editors: "",
            translators: "",
            editor: "ایډیټر",
            translator: "ژباړونکې",
            compiler: "را غونډونکې",
            interviewer: "مرکہ کونکې",
            site_editor: "خپرونکی",
            performer: "ترسرہ کونکې",
            producer: "جوړونکې",
            writer: "لیکوال",
            director: "مدیر",
            motion_picture: "موشن پکچر",
            patent: "",
            patent_suffix: "امتیازی حق نمبر",
            edition: "ایډیشن",
            volume: "ټوک",
            volume_before: false,
            pages: "پاڼې ",
            in_label: "دننہ",
            in_after: true,
            on_label: "پہ یو",
            on_after: true,
            retrieved: " لہ ",
            from_with_date: "",
            from_without_date: "",
            retrieval_date_after: true,
            retrieval_after_prefix: " څخہ یو ",
            retrieval_suffix: "ژغورل",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1114) => ApaBibliographyLanguage {
            language_bidi: Some("syr-SY"),
            rtl_cs_font: Some("Noto Sans Syriac Thin"),
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1025))?
        },
        BibliographyFormattingLocale::Numeric(1125) => ApaBibliographyLanguage {
            language_bidi: Some("dv-MV"),
            rtl_cs_font: Some("MV Boli"),
            day_first: true,
            day_month: "/",
            month_year: "/",
            from_with_date: ", from ",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1120))?
        },
        BibliographyFormattingLocale::Numeric(1085) => ApaBibliographyLanguage {
            initial_mark: '\'',
            language_bidi: Some("yi-Hebr"),
            day_first: true,
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1120))?
        },
        BibliographyFormattingLocale::Numeric(1119) => ApaBibliographyLanguage {
            language_bidi: Some("tzm-Arab-MA"),
            day_first: true,
            month_year: ", ",
            from_with_date: ", from ",
            ..apa_bibliography_language(BibliographyFormattingLocale::Numeric(1120))?
        },
        BibliographyFormattingLocale::Numeric(1120) => ApaBibliographyLanguage {
            language: None,
            language_bidi: Some("ks-Arab"),
            semantic_rtl: true,
            from_with_date: " from ",
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(1033) => APA_BIBLIOGRAPHY_ENGLISH,
        BibliographyFormattingLocale::Numeric(2057) => ApaBibliographyLanguage {
            language: None,
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(locale @ (1084 | 1127 | 1142 | 58378 | 58380)) => {
            ApaBibliographyLanguage {
                language: Some(match locale {
                    1127 => "ff-NG",
                    1142 => "la-Latn",
                    _ => "x-none",
                }),
                from_with_date: " from ",
                // Own missing-year and combined Author2/Editor2/Translator2 control.
                missing_year: Some("n.d."),
                author_pair: true,
                editors: "Eds.",
                translators: "Trans.",
                ..APA_BIBLIOGRAPHY_ENGLISH
            }
        }
        BibliographyFormattingLocale::Numeric(locale @ (1070 | 2055 | 3079 | 4103 | 5127)) => {
            ApaBibliographyLanguage {
                language: Some(match locale {
                    1070 => "hsb-DE",
                    2055 => "de-CH",
                    3079 => "de-AT",
                    4103 => "de-LU",
                    _ => "de-LI",
                }),
                // Own dense profiles establish these fields, not unmeasured sparse/plural branches.
                missing_year: None,
                editors: "",
                translators: "",
                ..APA_BIBLIOGRAPHY_GERMAN
            }
        }
        BibliographyFormattingLocale::Numeric(locale @ (1122 | 1145 | 2067)) => {
            ApaBibliographyLanguage {
                language: Some(match locale {
                    1122 => "fy-NL",
                    1145 => "pap-029",
                    _ => "nl-BE",
                }),
                // Own dense profiles establish these fields, not unmeasured sparse/plural branches.
                missing_year: None,
                editors: "",
                translators: "",
                ..APA_BIBLIOGRAPHY_DUTCH
            }
        }
        BibliographyFormattingLocale::Numeric(2072) => ApaBibliographyLanguage {
            language: Some("ro-MD"),
            // Own dense profiles establish these fields, not unmeasured sparse/plural branches.
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_ROMANIAN
        },
        BibliographyFormattingLocale::Numeric(2073) => ApaBibliographyLanguage {
            language: Some("ru-MD"),
            // Own dense profiles establish these fields, not unmeasured sparse/plural branches.
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_RUSSIAN
        },
        BibliographyFormattingLocale::Numeric(2077) => ApaBibliographyLanguage {
            language: Some("sv-FI"),
            // Own dense profiles establish these fields, not unmeasured sparse/plural branches.
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_SWEDISH
        },
        BibliographyFormattingLocale::Numeric(4122) => ApaBibliographyLanguage {
            language: Some("hr-BA"),
            // Own dense profiles establish these fields, not unmeasured sparse/plural branches.
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_CROATIAN
        },
        BibliographyFormattingLocale::Numeric(
            locale @ (1072 | 1073 | 1075 | 1117 | 1126 | 1129 | 1137 | 1141 | 1143 | 2092 | 2163
            | 3081 | 4105 | 5129 | 6153 | 7177 | 8201 | 9225 | 10249 | 11273 | 12297
            | 13321 | 14345 | 15369 | 16393),
        ) => ApaBibliographyLanguage {
            language: Some(match locale {
                1072 => "st-ZA",
                1073 => "ts-ZA",
                1075 => "ve-ZA",
                1117 => "iu-Cans-CA",
                1126 => "bin-NG",
                1129 => "ibb-NG",
                1137 => "kr-NG",
                1141 => "haw-US",
                1143 => "so-SO",
                2092 => "az-Cyrl-AZ",
                2163 => "ti-ER",
                3081 => "en-AU",
                4105 => "en-CA",
                5129 => "en-NZ",
                6153 => "en-IE",
                7177 => "en-ZA",
                8201 => "en-JM",
                9225 => "en-029",
                10249 => "en-BZ",
                11273 => "en-TT",
                12297 => "en-ZW",
                13321 => "en-PH",
                14345 => "en-ID",
                15369 => "en-HK",
                _ => "en-IN",
            }),
            // Own dense profiles do not establish missing-year or plural-role labels.
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_ENGLISH
        },
        BibliographyFormattingLocale::Numeric(
            locale @ (1140 | 2058 | 4106 | 5130 | 6154 | 7178 | 8202 | 9226 | 10250 | 11274 | 12298
            | 13322 | 14346 | 15370 | 16394 | 17418 | 18442 | 19466 | 20490),
        ) => ApaBibliographyLanguage {
            language: Some(match locale {
                1140 => "gn-PY",
                2058 => "es-MX",
                4106 => "es-GT",
                5130 => "es-CR",
                6154 => "es-PA",
                7178 => "es-DO",
                8202 => "es-VE",
                9226 => "es-CO",
                10250 => "es-PE",
                11274 => "es-AR",
                12298 => "es-EC",
                13322 => "es-CL",
                14346 => "es-UY",
                15370 => "es-PY",
                16394 => "es-BO",
                17418 => "es-SV",
                18442 => "es-HN",
                19466 => "es-NI",
                _ => "es-PR",
            }),
            // Own dense profiles do not establish missing-year or plural-role labels.
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_SPANISH
        },
        BibliographyFormattingLocale::Numeric(
            locale @ (2060 | 4108 | 5132 | 6156 | 7180 | 8204 | 9228 | 10252 | 11276 | 12300
            | 13324 | 14348 | 15372),
        ) => ApaBibliographyLanguage {
            language: Some(match locale {
                2060 => "fr-BE",
                4108 => "fr-CH",
                5132 => "fr-LU",
                6156 => "fr-MC",
                7180 => "fr-029",
                8204 => "fr-RE",
                9228 => "fr-CD",
                10252 => "fr-SN",
                11276 => "fr-CM",
                12300 => "fr-CI",
                13324 => "fr-ML",
                14348 => "fr-MA",
                _ => "fr-HT",
            }),
            // Own dense profiles do not establish missing-year or plural-role labels.
            missing_year: None,
            editors: "",
            translators: "",
            ..APA_BIBLIOGRAPHY_FRENCH
        },
        BibliographyFormattingLocale::Numeric(locale @ (2143 | 17417 | 18441)) => {
            ApaBibliographyLanguage {
                language: Some(match locale {
                    2143 => "tzm-Latn-DZ",
                    17417 => "en-MY",
                    _ => "en-SG",
                }),
                day_first: true,
                day_month: " ",
                month_year: ", ",
                missing_year: None,
                editors: "",
                translators: "",
                ..APA_BIBLIOGRAPHY_ENGLISH
            }
        }
        BibliographyFormattingLocale::Numeric(locale @ (1131 | 2155 | 3179)) => {
            ApaBibliographyLanguage {
                language: Some(match locale {
                    1131 => "quz-BO",
                    2155 => "quz-EC",
                    _ => "quz-PE",
                }),
                day_first: true,
                day_month: " de ",
                month_year: " de ",
                volume_before: false,
                volume: "suti",
                pages: "pags. ",
                patent: "Sut'i nº ",
                compiler: "Huñuqpa",
                interviewer: "Tapupayaq",
                in_label: "pi",
                in_after: true,
                producer: "Ruwaq",
                writer: "Qillqaq",
                director: "Kamachiq",
                performer: "T'ikraq",
                site_editor: "Liwru ruwaq",
                motion_picture: "Pilikula",
                recorded_by: "Ñit'ichasqaq",
                recorded_after: true,
                on_label: "manta",
                on_after: true,
                editor: "ed.",
                translator: "T'ikraq",
                edition: "Liwru ruwana wasi",
                retrieved: "",
                from_with_date: " Kutichiy ",
                retrieval_suffix: " manta",
                missing_year: None,
                editors: "",
                translators: "",
                ..APA_BIBLIOGRAPHY_ENGLISH
            }
        }
        BibliographyFormattingLocale::Numeric(1036) => APA_BIBLIOGRAPHY_FRENCH,
        BibliographyFormattingLocale::Numeric(3082) => APA_BIBLIOGRAPHY_SPANISH,
        BibliographyFormattingLocale::Numeric(1034) => ApaBibliographyLanguage {
            language: Some("es-ES_tradnl"),
            ..APA_BIBLIOGRAPHY_SPANISH
        },
        BibliographyFormattingLocale::Numeric(1031) => APA_BIBLIOGRAPHY_GERMAN,
        BibliographyFormattingLocale::Numeric(1040) => APA_BIBLIOGRAPHY_ITALIAN,
        BibliographyFormattingLocale::Numeric(1043) => APA_BIBLIOGRAPHY_DUTCH,
        BibliographyFormattingLocale::Numeric(1030) => APA_BIBLIOGRAPHY_DANISH,
        BibliographyFormattingLocale::Numeric(1080) => ApaBibliographyLanguage {
            language: Some("fo-FO"),
            // Own dense row55 and isolated missing-year/Editor2/Translator2 controls.
            editors: "Red.",
            translators: "Ovs.",
            ..APA_BIBLIOGRAPHY_DANISH
        },
        BibliographyFormattingLocale::Numeric(1027) => APA_BIBLIOGRAPHY_CATALAN,
        BibliographyFormattingLocale::Numeric(1029) => APA_BIBLIOGRAPHY_CZECH,
        BibliographyFormattingLocale::Numeric(1026) => APA_BIBLIOGRAPHY_BULGARIAN,
        BibliographyFormattingLocale::Numeric(1038) => APA_BIBLIOGRAPHY_HUNGARIAN,
        BibliographyFormattingLocale::Numeric(1039) => APA_BIBLIOGRAPHY_ICELANDIC,
        BibliographyFormattingLocale::Numeric(1044) => APA_BIBLIOGRAPHY_NORWEGIAN,
        BibliographyFormattingLocale::Numeric(1048) => APA_BIBLIOGRAPHY_ROMANIAN,
        BibliographyFormattingLocale::Numeric(1050) => APA_BIBLIOGRAPHY_CROATIAN,
        BibliographyFormattingLocale::Numeric(1051) => APA_BIBLIOGRAPHY_SLOVAK,
        BibliographyFormattingLocale::Numeric(1060) => APA_BIBLIOGRAPHY_SLOVENIAN,
        BibliographyFormattingLocale::Numeric(1062) => APA_BIBLIOGRAPHY_LATVIAN,
        BibliographyFormattingLocale::Numeric(1063) => APA_BIBLIOGRAPHY_LITHUANIAN,
        BibliographyFormattingLocale::Numeric(1049) => APA_BIBLIOGRAPHY_RUSSIAN,
        BibliographyFormattingLocale::Numeric(1058) => APA_BIBLIOGRAPHY_UKRAINIAN,
        BibliographyFormattingLocale::Numeric(1032) => APA_BIBLIOGRAPHY_GREEK,
        BibliographyFormattingLocale::Numeric(1045) => APA_BIBLIOGRAPHY_POLISH,
        BibliographyFormattingLocale::Numeric(1035) => APA_BIBLIOGRAPHY_FINNISH,
        BibliographyFormattingLocale::Numeric(1053) => APA_BIBLIOGRAPHY_SWEDISH,
        BibliographyFormattingLocale::Numeric(2070) => APA_BIBLIOGRAPHY_PORTUGUESE,
        BibliographyFormattingLocale::Numeric(1046) => ApaBibliographyLanguage {
            language: Some("pt-BR"),
            director: "Diretor",
            motion_picture: "Filme Cinematográfico",
            recorded_by: "Gravado por",
            recorded_after: false,
            editors: "Eds.",
            retrieved: " Acesso em",
            from_without_date: " disponível em ",
            from_with_date: ", disponível em ",
            ..APA_BIBLIOGRAPHY_PORTUGUESE
        },
        BibliographyFormattingLocale::Numeric(3084) => ApaBibliographyLanguage {
            language: Some("fr-CA"),
            editors: "Éd.",
            ..APA_BIBLIOGRAPHY_FRENCH
        },
        _ => {
            return Err(bibliography_error(
                "localized APA bibliography grammar is still being implemented",
            ));
        }
    })
}

fn source_bibliography_paragraph(
    source: &BibliographySource,
    style: BibliographyStyle,
    reference: Option<u32>,
    locale: BibliographyFormattingLocale,
) -> Result<rdocx_oxml::text::CT_P> {
    if style == BibliographyStyle::ApaSixthEdition {
        let grammar = apa_bibliography_language(locale)?;
        let measured_sparse_dates = source.kind == BibliographySourceKind::DocumentFromInternetSite
            && grammar.publication_month_first;
        if grammar.language != Some("en-US")
            && ((!measured_sparse_dates
                && first_source_property(source, BibliographySourceField::Year).is_empty()
                && !(source.kind == BibliographySourceKind::Book
                    && grammar.missing_year.is_some()
                    && !grammar.publication_month_first))
                || (!measured_sparse_dates
                    && (grammar.day_first
                        || grammar.calendar_year_first
                        || grammar.publication_month_first)
                    && [BibliographySourceField::Month, BibliographySourceField::Day]
                        .iter()
                        .any(|field| first_source_property(source, *field).is_empty()))
                || source
                    .contributors
                    .iter()
                    .enumerate()
                    .any(|(index, contributor)| {
                        source.contributors[..index]
                            .iter()
                            .any(|earlier| earlier.role == contributor.role)
                    })
                || source
                    .contributors
                    .iter()
                    .any(|contributor| match &contributor.value {
                        BibliographyAuthor::People(people) => {
                            people.len() != 1
                                && !(source.kind == BibliographySourceKind::Book
                                    && people.len() == 2
                                    && match contributor.role {
                                        BibliographyContributorRole::Author => grammar.author_pair,
                                        BibliographyContributorRole::Editor => {
                                            !grammar.editors.is_empty()
                                        }
                                        BibliographyContributorRole::Translator => {
                                            !grammar.translators.is_empty()
                                        }
                                        _ => false,
                                    }
                                    && !apa_bibliography_names(
                                        source,
                                        BibliographyContributorRole::Author,
                                        true,
                                        &grammar,
                                    )
                                    .is_empty())
                        }
                        BibliographyAuthor::Corporate(_) => true,
                    })
                || (!measured_sparse_dates
                    && (!first_source_property(source, BibliographySourceField::Url).is_empty()
                        || !first_source_property(
                            source,
                            BibliographySourceField::InternetSiteTitle,
                        )
                        .is_empty())
                    && [
                        BibliographySourceField::MonthAccessed,
                        BibliographySourceField::DayAccessed,
                        BibliographySourceField::YearAccessed,
                    ]
                    .iter()
                    .any(|field| first_source_property(source, *field).is_empty())))
        {
            return Err(bibliography_error(
                "localized APA bibliography branch is still being implemented",
            ));
        }
        if grammar.semantic_rtl
            || grammar.numeric_cs_font.is_some()
            || grammar.literal_font.is_some()
        {
            return match source.kind {
                BibliographySourceKind::Book
                | BibliographySourceKind::Report
                | BibliographySourceKind::DocumentFromInternetSite => {
                    apa_rtl_authored_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::Art => apa_rtl_art_bibliography_paragraph(source, &grammar),
                BibliographySourceKind::ArticleInAPeriodical
                | BibliographySourceKind::JournalArticle => {
                    apa_rtl_periodical_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::Patent | BibliographySourceKind::Case => {
                    apa_rtl_legal_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::Performance => {
                    apa_rtl_performance_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::Interview => {
                    apa_rtl_interview_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::ElectronicSource | BibliographySourceKind::Misc => {
                    apa_rtl_general_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::BookSection
                | BibliographySourceKind::ConferenceProceedings => {
                    apa_rtl_chapter_conference_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::InternetSite => {
                    apa_rtl_site_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::SoundRecording => {
                    apa_rtl_sound_bibliography_paragraph(source, &grammar)
                }
                BibliographySourceKind::Film => {
                    apa_rtl_film_bibliography_paragraph(source, &grammar)
                }
            };
        }
        let mut paragraph = match source.kind {
            BibliographySourceKind::Book
            | BibliographySourceKind::Report
            | BibliographySourceKind::DocumentFromInternetSite => {
                apa_authored_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::Patent | BibliographySourceKind::Case => {
                apa_legal_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::ArticleInAPeriodical
            | BibliographySourceKind::JournalArticle => {
                apa_periodical_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::ElectronicSource | BibliographySourceKind::Misc => {
                apa_general_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::BookSection => {
                apa_chapter_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::ConferenceProceedings => {
                apa_conference_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::Film => apa_film_bibliography_paragraph(source, &grammar),
            BibliographySourceKind::InternetSite => {
                apa_site_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::SoundRecording => {
                apa_sound_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::Art => apa_art_bibliography_paragraph(source, &grammar),
            BibliographySourceKind::Performance => {
                apa_performance_bibliography_paragraph(source, &grammar)
            }
            BibliographySourceKind::Interview => {
                apa_interview_bibliography_paragraph(source, &grammar)
            }
        };
        if let Some(properties) = paragraph.properties.as_mut()
            && let Some(properties) = properties.rpr.as_mut()
        {
            properties.language = grammar.language.map(str::to_owned);
        }
        for run in &mut paragraph.runs {
            if let Some(properties) = run.properties.as_mut()
                && properties.language.is_some()
            {
                properties.language = grammar.language.map(str::to_owned);
            }
        }
        return Ok(paragraph);
    }
    let Some(BibliographyAuthor::People(people)) =
        source.contributors.first().map(|value| &value.value)
    else {
        return Err(bibliography_error(
            "catalogued bibliography contributor branch is still being implemented",
        ));
    };
    let [person] = people.as_slice() else {
        return Err(bibliography_error(
            "catalogued bibliography contributor count branch is still being implemented",
        ));
    };
    let last = person.last.join(" ");
    let first = person.first.join(" ");
    let middle = person.middle.join(" ");
    let title = first_source_property(source, BibliographySourceField::Title);
    let year = first_source_property(source, BibliographySourceField::Year);
    let city = first_source_property(source, BibliographySourceField::City);
    let publisher = first_source_property(source, BibliographySourceField::Publisher);
    if [
        last.as_str(),
        first.as_str(),
        middle.as_str(),
        title,
        year,
        city,
        publisher,
    ]
    .iter()
    .any(|value| !value.is_ascii())
        || [last.as_str(), first.as_str(), title, year, city, publisher]
            .iter()
            .any(|value| value.is_empty())
    {
        return Err(bibliography_error(
            "catalogued bibliography source-variable branch is still being implemented",
        ));
    }
    let given = if middle.is_empty() {
        first
    } else {
        format!("{first} {middle}")
    };
    let inverted = format!("{last}, {given}.");
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(style));
    match style {
        BibliographyStyle::Chicago => {
            paragraph.runs.push(bibliography_display_run(
                &format!("{inverted} {year}. "),
                false,
                false,
            ));
            paragraph
                .runs
                .push(bibliography_display_run(&format!("{title}."), false, true));
            paragraph.runs.push(bibliography_display_run(
                &format!(" {city}: {publisher}."),
                false,
                false,
            ));
        }
        BibliographyStyle::Iso690AuthorDate | BibliographyStyle::Gb7714 => {
            paragraph.runs.push(bibliography_display_run(
                &format!("{inverted} {year}."),
                true,
                false,
            ));
            paragraph
                .runs
                .push(bibliography_display_run(" ", false, false));
            paragraph
                .runs
                .push(bibliography_display_run(&format!("{title}. "), false, true));
            paragraph.runs.push(bibliography_display_run(
                &format!("{city}\u{00a0}: {publisher}, {year}."),
                false,
                false,
            ));
        }
        BibliographyStyle::GostName => {
            paragraph.runs.push(bibliography_display_run(
                &format!("{last} {given}"),
                true,
                false,
            ));
            paragraph.runs.push(bibliography_display_run(
                &format!(" {title} [Book].\u{00a0}- {city}\u{00a0}: {publisher}, {year}."),
                false,
                false,
            ));
        }
        BibliographyStyle::GostTitle => {
            paragraph
                .runs
                .push(bibliography_display_run(title, true, false));
            paragraph.runs.push(bibliography_display_run(&format!(" [Book]\u{00a0}/ auth. {last} {given}.\u{00a0}- {city}\u{00a0}: {publisher}, {year}."), false, false));
        }

        BibliographyStyle::HarvardAnglia => {
            let initials = person
                .first
                .iter()
                .chain(&person.middle)
                .flat_map(|part| part.split_whitespace())
                .filter_map(|part| part.chars().next())
                .map(|initial| format!("{initial}."))
                .collect::<Vec<_>>()
                .join(" ");
            paragraph.runs.push(bibliography_display_run(
                &format!("{last}, {initials}, {year}. "),
                false,
                false,
            ));
            paragraph
                .runs
                .push(bibliography_display_run(&format!("{title}. "), false, true));
            paragraph.runs.push(bibliography_display_run(
                &format!("{city}: {publisher}."),
                false,
                false,
            ));
        }
        BibliographyStyle::MlaSeventhEdition => {
            paragraph.runs.push(bibliography_display_run(
                &format!("{inverted} "),
                false,
                false,
            ));
            paragraph
                .runs
                .push(bibliography_display_run(title, false, true));
            paragraph.runs.push(bibliography_display_run(
                &format!(". {city}: {publisher}, {year}."),
                false,
                false,
            ));
        }
        BibliographyStyle::Sist02 => {
            paragraph.runs.push(bibliography_display_run(
                &format!("{last}{}{}", person.first.join(""), person.middle.join("")),
                true,
                false,
            ));
            paragraph
                .runs
                .push(bibliography_display_run(" ", false, false));
            paragraph
                .runs
                .push(bibliography_display_run(title, false, true));
            paragraph.runs.push(east_asian_bibliography_run("．", true));
            paragraph
                .runs
                .push(bibliography_display_run(city, false, false));
            paragraph
                .runs
                .push(east_asian_bibliography_run("，", false));
            paragraph
                .runs
                .push(bibliography_display_run(publisher, false, false));
            paragraph
                .runs
                .push(east_asian_bibliography_run("，", false));
            paragraph
                .runs
                .push(bibliography_display_run(year, false, false));
            paragraph
                .runs
                .push(east_asian_bibliography_run("．", false));
        }

        BibliographyStyle::Iso690Numeric => {
            let number = reference.ok_or_else(|| {
                bibliography_error("missing effective bibliography reference number")
            })?;
            paragraph.runs.push(bibliography_display_run(
                &format!("{number}. "),
                false,
                false,
            ));
            paragraph
                .runs
                .push(bibliography_display_run(&inverted, true, false));
            paragraph
                .runs
                .push(bibliography_display_run(" ", false, false));
            paragraph
                .runs
                .push(bibliography_display_run(&format!("{title}. "), false, true));
            paragraph.runs.push(bibliography_display_run(
                &format!("{city}\u{00a0}: {publisher}, {year}."),
                false,
                false,
            ));
        }
        BibliographyStyle::Turabian => {
            paragraph.runs.push(bibliography_display_run(
                &format!("{inverted} "),
                false,
                false,
            ));
            paragraph
                .runs
                .push(bibliography_display_run(&format!("{title}."), false, true));
            paragraph.runs.push(bibliography_display_run(
                &format!(" {city}: {publisher}, {year}."),
                false,
                false,
            ));
        }
        BibliographyStyle::Ieee => {
            let initials = person
                .first
                .iter()
                .chain(&person.middle)
                .flat_map(|part| part.split_whitespace())
                .filter_map(|part| part.chars().next())
                .map(|initial| format!("{initial}."))
                .collect::<Vec<_>>()
                .join(" ");
            paragraph.runs.push(bibliography_display_run(
                &format!("{initials} {last}, {title}, {city}: {publisher}, {year}. "),
                false,
                false,
            ));
        }
        _ => {
            return Err(bibliography_error(
                "catalogued bibliography formatter branch is still being implemented",
            ));
        }
    }
    Ok(paragraph)
}

fn bibliography_display_run(text: &str, bold: bool, italic: bool) -> rdocx_oxml::text::CT_R {
    let mut run = rdocx_oxml::text::CT_R::new(text);
    run.properties = Some(rdocx_oxml::CT_RPr {
        no_proof: Some(true),
        language: Some("en-US".into()),
        bold: bold.then_some(true),
        bold_cs: bold.then_some(true),
        italic: italic.then_some(true),
        italic_cs: italic.then_some(true),
        ..Default::default()
    });
    run
}

fn east_asian_bibliography_run(text: &str, italic: bool) -> rdocx_oxml::text::CT_R {
    let mut run = bibliography_display_run(text, false, italic);
    if let Some(properties) = run.properties.as_mut() {
        properties.font_ascii = Some("MS Gothic".into());
        properties.font_hansi = Some("MS Gothic".into());
        properties.font_east_asia = Some("MS Gothic".into());
        properties.font_cs = Some("MS Gothic".into());
        properties.font_hint = Some("eastAsia".into());
    }
    run
}

fn bibliography_paragraph_properties(style: BibliographyStyle) -> rdocx_oxml::CT_PPr {
    use rdocx_oxml::units::{HalfPoint, Twips};
    let hanging = matches!(
        style,
        BibliographyStyle::ApaSixthEdition
            | BibliographyStyle::Chicago
            | BibliographyStyle::MlaSeventhEdition
            | BibliographyStyle::Turabian
    );
    rdocx_oxml::CT_PPr {
        style_id: Some("Bibliography".into()),
        ind_left: hanging.then_some(Twips(720)),
        ind_hanging: hanging.then_some(Twips(720)),
        rpr: Some(rdocx_oxml::CT_RPr {
            no_proof: Some(true),
            sz: Some(HalfPoint(24)),
            sz_cs: Some(HalfPoint(24)),
            language: Some("en-US".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn apa_bibliography_names(
    source: &BibliographySource,
    role: BibliographyContributorRole,
    inverted: bool,
    grammar: &ApaBibliographyLanguage,
) -> String {
    source
        .contributors
        .iter()
        .filter(|contributor| contributor.role == role)
        .map(|contributor| {
            let names = match &contributor.value {
                BibliographyAuthor::Corporate(value) => vec![if value.ends_with('.') {
                    value.clone()
                } else {
                    format!("{value}.")
                }],
                BibliographyAuthor::People(people) => people
                    .iter()
                    .filter_map(|person| {
                        let first = person.first.first().map_or("", String::as_str);
                        let middle = person.middle.first().map_or("", String::as_str);
                        let last = person.last.first().map_or("", String::as_str);
                        if grammar.joined_names {
                            let name = [last, middle, first].join("");
                            return (!name.is_empty()).then_some(name);
                        }
                        if grammar.full_names {
                            let name = [first, middle, last]
                                .into_iter()
                                .filter(|part| !part.is_empty())
                                .collect::<Vec<_>>()
                                .join(" ");
                            return (!name.is_empty()).then_some(name);
                        }
                        let initials = [first, middle]
                            .into_iter()
                            .flat_map(str::split_whitespace)
                            .map(|part| {
                                part.split('-')
                                    .filter_map(|part| part.chars().next())
                                    .map(|initial| format!("{initial}{}", grammar.initial_mark))
                                    .collect::<Vec<_>>()
                                    .join("-")
                            })
                            .filter(|part| !part.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        if last.is_empty() {
                            let given = [first, middle]
                                .into_iter()
                                .filter(|part| !part.is_empty())
                                .collect::<Vec<_>>()
                                .join(" ");
                            return (!given.is_empty()).then(|| {
                                if given.ends_with('.') {
                                    given
                                } else {
                                    format!("{given}.")
                                }
                            });
                        }
                        Some(if initials.is_empty() {
                            if last.ends_with('.') {
                                last.to_owned()
                            } else {
                                format!("{last}.")
                            }
                        } else if inverted {
                            format!("{last}, {initials}")
                        } else {
                            format!("{initials} {last}")
                        })
                    })
                    .collect::<Vec<_>>(),
            };
            match names.as_slice() {
                [] => String::new(),
                [name] => name.clone(),
                names
                    if names.len() <= 7
                        && role == BibliographyContributorRole::Editor
                        && !inverted =>
                {
                    names.join(grammar.contributor_separator)
                }
                names if names.len() <= 7 => format!(
                    "{}{}{} {}",
                    names[..names.len() - 1].join(grammar.contributor_separator),
                    grammar.contributor_separator,
                    grammar.conjunction,
                    names.last().expect("nonempty names")
                ),
                names => format!(
                    "{}, . . . {}",
                    names[..6].join(", "),
                    names.last().expect("nonempty names")
                ),
            }
        })
        .collect::<String>()
}

fn bibliography_display_runs(text: &str, italic: bool) -> Vec<rdocx_oxml::text::CT_R> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut east_asian = None;
    for (offset, character) in text.char_indices() {
        let current = matches!(character, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}');
        if east_asian.is_some_and(|previous| previous != current) {
            let fragment = &text[start..offset];
            runs.push(if east_asian == Some(true) {
                east_asian_bibliography_run(fragment, italic)
            } else {
                bibliography_display_run(fragment, false, italic)
            });
            start = offset;
        }
        east_asian = Some(current);
    }
    if start < text.len() {
        runs.push(if east_asian == Some(true) {
            east_asian_bibliography_run(&text[start..], italic)
        } else {
            bibliography_display_run(&text[start..], false, italic)
        });
    }
    runs
}

fn apa_bibliography_edition(edition: &str, grammar: &ApaBibliographyLanguage) -> String {
    if grammar.edition_before {
        format!(
            "{}{}{edition}{}",
            grammar.edition, grammar.edition_separator, grammar.edition_suffix
        )
    } else {
        format!(
            "{edition}{}{}{}",
            grammar.edition_separator, grammar.edition, grammar.edition_suffix
        )
    }
}

fn apa_bibliography_volume(volume: &str, grammar: &ApaBibliographyLanguage) -> String {
    if grammar.volume_before {
        format!(
            "{}{}{volume}{}",
            grammar.volume, grammar.volume_separator, grammar.volume_suffix
        )
    } else {
        format!(
            "{volume}{}{}{}",
            grammar.volume_separator, grammar.volume, grammar.volume_suffix
        )
    }
}

fn apa_bibliography_date(source: &BibliographySource, grammar: &ApaBibliographyLanguage) -> String {
    let year = first_source_property(source, BibliographySourceField::Year);
    let date = if year.is_empty() {
        grammar.missing_year.unwrap_or_default().to_owned()
    } else {
        year.to_owned()
    };
    let month = first_source_property(source, BibliographySourceField::Month);
    let day = first_source_property(source, BibliographySourceField::Day);
    if grammar.publication_month_first && !year.is_empty() && !month.is_empty() {
        return if day.is_empty() {
            format!("{month}{}{year}", grammar.month_year)
        } else {
            format!("{month} {day}{}{year}", grammar.calendar_day_year)
        };
    }
    if grammar.day_first && !year.is_empty() && !month.is_empty() {
        return format!(
            "{}{day}{}{month}{}{year}{}",
            grammar.date_prefix, grammar.day_month, grammar.month_year, grammar.date_suffix
        );
    }
    if !year.is_empty() && !month.is_empty() {
        return apa_bibliography_year_date(year, month, day, grammar);
    }
    date
}

fn apa_bibliography_year_date(
    year: &str,
    month: &str,
    day: &str,
    grammar: &ApaBibliographyLanguage,
) -> String {
    if grammar.year_day_first {
        return format!(
            "{year}{}{day}{}{month}{}",
            grammar.year_month, grammar.day_month, grammar.date_suffix
        );
    }
    let mut date = format!("{year}{}{month}", grammar.year_month);
    if !day.is_empty() {
        date.push_str(&format!("{}{day}", grammar.month_day));
    }
    date.push_str(grammar.date_suffix);
    date
}

fn apa_bibliography_retrieval(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> String {
    let mut publication = String::new();
    let url = first_source_property(source, BibliographySourceField::Url);
    let site = first_source_property(source, BibliographySourceField::InternetSiteTitle);
    if !url.is_empty() || !site.is_empty() {
        let month = first_source_property(source, BibliographySourceField::MonthAccessed);
        let day = first_source_property(source, BibliographySourceField::DayAccessed);
        let year = first_source_property(source, BibliographySourceField::YearAccessed);
        publication.push_str(if month.is_empty() && year.is_empty() {
            grammar.retrieved_without_date.unwrap_or(grammar.retrieved)
        } else {
            grammar.retrieved
        });
        if grammar.calendar_year_first && !grammar.retrieval_date_after {
            publication.push_str(grammar.retrieval_date_separator);
            publication.push_str(&apa_bibliography_year_date(year, month, day, grammar));
        } else if grammar.day_first && !grammar.retrieval_date_after {
            let date = format!(
                "{day}{}{month}{}{year}",
                grammar.day_month, grammar.month_year
            );
            if !date.is_empty() {
                publication.push_str(grammar.retrieval_date_separator);
                publication.push_str(&date);
            }
        } else if !grammar.retrieval_date_after {
            if !month.is_empty() {
                publication.push_str(&format!("{}{month}", grammar.retrieval_date_separator));
                if !day.is_empty() {
                    publication.push_str(&format!(" {day}"));
                }
            }
            if !year.is_empty() {
                publication.push_str(if !month.is_empty() && !day.is_empty() {
                    grammar.calendar_day_year
                } else if !month.is_empty() && grammar.publication_month_first {
                    grammar.month_year
                } else {
                    " "
                });
                publication.push_str(year);
            }
        }
        publication.push_str(if month.is_empty() && year.is_empty() {
            grammar.from_without_date
        } else {
            grammar.from_with_date
        });
        if !site.is_empty() {
            publication.push_str(site);
            publication.push_str(if url.is_empty() { "." } else { ": " });
        }
        publication.push_str(url);
        if grammar.retrieval_date_after {
            publication.push_str(grammar.retrieval_after_prefix);
            publication.push_str(grammar.date_prefix);
            if grammar.calendar_year_first {
                publication.push_str(&apa_bibliography_year_date(year, month, day, grammar));
            } else if grammar.day_first {
                publication.push_str(&format!(
                    "{day}{}{month}{}{year}",
                    grammar.day_month, grammar.month_year
                ));
            } else {
                publication.push_str(&format!(
                    "{month}{}{day}{}{year}",
                    grammar.month_day, grammar.calendar_day_year
                ));
            }
        }
        publication.push_str(grammar.retrieval_suffix);
    }
    publication
}

fn apa_legal_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    let retrieval = apa_bibliography_retrieval(source, grammar);
    if source.kind == BibliographySourceKind::Patent {
        let inventor =
            apa_bibliography_names(source, BibliographyContributorRole::Inventor, true, grammar);
        let date = apa_bibliography_date(source, grammar);
        let mut prefix = if inventor.is_empty() {
            format!("({date}).")
        } else {
            format!("{inventor} ({date}).")
        };
        let country = first_source_property(source, BibliographySourceField::CountryRegion);
        let number = first_source_property(source, BibliographySourceField::PatentNumber);
        if country.is_empty() && number.is_empty() {
            prefix.push_str(&retrieval);
            paragraph
                .runs
                .extend(bibliography_display_runs(&prefix, false));
        } else {
            paragraph
                .runs
                .extend(bibliography_display_runs(&prefix, false));
            paragraph.runs.extend(bibliography_display_runs(
                &format!(
                    " {}{}{number}{}.",
                    if country.is_empty() {
                        String::new()
                    } else {
                        format!("{country} ")
                    },
                    grammar.patent,
                    grammar.patent_suffix
                ),
                true,
            ));
            paragraph
                .runs
                .extend(bibliography_display_runs(&retrieval, false));
        }
    } else {
        let title = first_source_property(source, BibliographySourceField::Title);
        let number = first_source_property(source, BibliographySourceField::CaseNumber);
        let court = first_source_property(source, BibliographySourceField::Court);
        let year = first_source_property(source, BibliographySourceField::Year);
        let month = first_source_property(source, BibliographySourceField::Month);
        let day = first_source_property(source, BibliographySourceField::Day);
        let mut text = title.to_owned();
        if !number.is_empty() {
            if !text.is_empty() {
                text.push_str(", ");
            }
            text.push_str(number);
        }
        let mut date = court.to_owned();
        if grammar.day_first || grammar.calendar_year_first {
            if !date.is_empty() {
                date.push(' ');
            }
            date.push_str(&apa_bibliography_date(source, grammar));
        } else {
            if !month.is_empty() {
                if !date.is_empty() {
                    date.push(' ');
                }
                date.push_str(month);
                if !day.is_empty() {
                    date.push_str(&format!(" {day}"));
                }
            }
            if !year.is_empty() {
                if !date.is_empty() {
                    date.push_str(if month.is_empty() {
                        " "
                    } else {
                        grammar.calendar_day_year
                    });
                }
                date.push_str(year);
            }
        }
        if !date.is_empty() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&format!("({date})"));
        }
        let invalid = text.is_empty();
        if invalid {
            text.push_str("Invalid source specified");
        }
        text.push('.');
        text.push_str(&retrieval);
        if invalid {
            let mut run = bibliography_display_run(&text, true, false);
            if let Some(properties) = run.properties.as_mut() {
                properties.language = None;
            }
            paragraph.runs.push(run);
        } else {
            paragraph
                .runs
                .extend(bibliography_display_runs(&text, false));
        }
    }
    paragraph
}

fn apa_general_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let mut prefix = apa_author_title_prefix(source, grammar);
    let publication = first_source_property(source, BibliographySourceField::PublicationTitle);
    let volume = first_source_property(source, BibliographySourceField::Volume);
    let issue = first_source_property(source, BibliographySourceField::Issue);
    let edition = first_source_property(source, BibliographySourceField::Edition);
    let pages = first_source_property(source, BibliographySourceField::Pages);
    let miscellaneous = source.kind == BibliographySourceKind::Misc;
    let mut heading = publication.to_owned();
    if !volume.is_empty() {
        if !heading.is_empty() {
            heading.push_str(", ");
        }
        heading.push_str(volume);
    }
    if miscellaneous {
        if !issue.is_empty() {
            heading.push_str(&format!("({issue})"));
        }
        if !edition.is_empty() {
            if !heading.is_empty() {
                heading.push_str(", ");
            }
            heading.push_str(edition);
        }
    }
    let mut suffix = String::new();
    if !miscellaneous && !edition.is_empty() {
        suffix.push_str(&format!("({edition})"));
    }
    if !pages.is_empty() {
        if !heading.is_empty() || !suffix.is_empty() {
            suffix.push_str(", ");
        }
        suffix.push_str(pages);
    }
    if !heading.is_empty() || !suffix.is_empty() {
        suffix.push('.');
    }
    let mut secondary = Vec::new();
    for (role, label) in [
        (BibliographyContributorRole::Editor, grammar.editor),
        (BibliographyContributorRole::Translator, grammar.translator),
        (BibliographyContributorRole::Compiler, grammar.compiler),
    ] {
        let name = apa_bibliography_names(source, role, false, grammar);
        if !name.is_empty() {
            secondary.push(format!("{name}, {label}"));
        }
    }
    if !secondary.is_empty() {
        let names = if secondary.len() == 1 {
            secondary[0].clone()
        } else {
            format!(
                "{}{}{} {}",
                secondary[..secondary.len() - 1].join(grammar.contributor_separator),
                grammar.contributor_separator,
                grammar.conjunction,
                secondary.last().unwrap()
            )
        };
        suffix.push_str(&format!(" ({names})"));
    }
    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    let publisher = first_source_property(source, BibliographySourceField::Publisher);
    if !location.is_empty() || !publisher.is_empty() {
        suffix.push(' ');
        suffix.push_str(&location);
        if !location.is_empty() && !publisher.is_empty() {
            suffix.push_str(": ");
        }
        suffix.push_str(publisher);
        suffix.push('.');
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    if heading.is_empty() {
        if !suffix.is_empty() && !suffix.starts_with(' ') {
            prefix.push(' ');
        }
        prefix.push_str(&suffix);
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
    } else {
        prefix.push(' ');
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        paragraph
            .runs
            .extend(bibliography_display_runs(&heading, true));
        paragraph
            .runs
            .extend(bibliography_display_runs(&suffix, false));
    }
    paragraph
}

fn apa_interview_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let creator = apa_bibliography_names(
        source,
        BibliographyContributorRole::Interviewee,
        true,
        grammar,
    );
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    let broadcast = first_source_property(source, BibliographySourceField::BroadcastTitle);
    let mut prefix = if creator.is_empty() {
        format!("({date}).")
    } else {
        format!("{creator} ({date}).")
    };
    if !title.is_empty() {
        prefix.push_str(&format!(" {title}."));
    }
    let interviewer = apa_bibliography_names(
        source,
        BibliographyContributorRole::Interviewer,
        false,
        grammar,
    );
    let mut suffix = String::new();
    if broadcast.is_empty() {
        let pages = first_source_property(source, BibliographySourceField::Pages);
        if !pages.is_empty() {
            prefix.push_str(&format!(" {pages}."));
        }
    }
    let mut secondary = Vec::new();
    if !interviewer.is_empty() {
        secondary.push(format!("{interviewer}, {}", grammar.interviewer));
    }
    if broadcast.is_empty() {
        for (role, label) in [
            (BibliographyContributorRole::Editor, grammar.site_editor),
            (BibliographyContributorRole::Translator, "Translator"),
        ] {
            let name = apa_bibliography_names(source, role, false, grammar);
            if !name.is_empty() {
                secondary.push(format!("{name}, {label}"));
            }
        }
    }
    if !secondary.is_empty() {
        let names = if secondary.len() == 1 {
            secondary[0].clone()
        } else {
            format!(
                "{}{}{} {}",
                secondary[..secondary.len() - 1].join(grammar.contributor_separator),
                grammar.contributor_separator,
                grammar.conjunction,
                secondary.last().unwrap()
            )
        };
        suffix.push_str(&format!(" ({names})"));
    }
    if broadcast.is_empty() {
        let location = [
            BibliographySourceField::City,
            BibliographySourceField::StateProvince,
            BibliographySourceField::CountryRegion,
        ]
        .into_iter()
        .map(|field| first_source_property(source, field))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        let publisher = first_source_property(source, BibliographySourceField::Publisher);
        if !location.is_empty() || !publisher.is_empty() {
            suffix.push(' ');
            suffix.push_str(&location);
            if !location.is_empty() && !publisher.is_empty() {
                suffix.push_str(": ");
            }
            suffix.push_str(publisher);
            suffix.push('.');
        }
    } else {
        let broadcaster = first_source_property(source, BibliographySourceField::Broadcaster);
        if !broadcaster.is_empty() {
            suffix.push_str(&format!(" {broadcaster}."));
        }
        let location = [
            BibliographySourceField::Station,
            BibliographySourceField::City,
        ]
        .into_iter()
        .map(|field| first_source_property(source, field))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        if !location.is_empty() {
            suffix.push_str(&format!(" {location}."));
        }
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    if broadcast.is_empty() {
        prefix.push_str(&suffix);
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
    } else {
        prefix.push(' ');
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        paragraph
            .runs
            .extend(bibliography_display_runs(&format!("{broadcast}."), true));
        paragraph
            .runs
            .extend(bibliography_display_runs(&suffix, false));
    }
    paragraph
}

fn apa_author_title_prefix(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> String {
    let author = apa_bibliography_names(source, BibliographyContributorRole::Author, true, grammar);
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    if author.is_empty() {
        if title.is_empty() {
            format!("({date}).")
        } else {
            format!("{title}. ({date}).")
        }
    } else {
        format!(
            "{author} ({date}).{}",
            if title.is_empty() {
                String::new()
            } else {
                format!(" {title}.")
            }
        )
    }
}

fn apa_chapter_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let mut prefix = apa_author_title_prefix(source, grammar);
    let author = apa_bibliography_names(
        source,
        BibliographyContributorRole::BookAuthor,
        false,
        grammar,
    );
    let editor =
        apa_bibliography_names(source, BibliographyContributorRole::Editor, false, grammar);
    let book = first_source_property(source, BibliographySourceField::BookTitle);
    let mut names = author;
    if !editor.is_empty() {
        if !names.is_empty() {
            names.push_str(&format!(
                "{}{} ",
                grammar.contributor_separator, grammar.conjunction
            ));
        }
        names.push_str(&format!("{editor} ({})", grammar.editor));
    }
    if !names.is_empty() || !book.is_empty() {
        prefix.push(' ');
        if !grammar.in_after && !grammar.in_label.is_empty() {
            prefix.push_str(grammar.in_label);
            prefix.push_str(grammar.in_before_separator);
        }
        prefix.push_str(&names);
        if !names.is_empty() {
            if grammar.in_after && !grammar.in_label.is_empty() {
                prefix.push_str(&format!("{}{}", grammar.in_separator, grammar.in_label));
            }
            prefix.push_str(if book.is_empty() { "." } else { ", " });
        }
    }
    let mut suffix = String::new();
    if !book.is_empty() {
        let translator = apa_bibliography_names(
            source,
            BibliographyContributorRole::Translator,
            false,
            grammar,
        );
        let edition = first_source_property(source, BibliographySourceField::Edition);
        let volume = first_source_property(source, BibliographySourceField::Volume);
        let pages = first_source_property(source, BibliographySourceField::Pages);
        let mut details = Vec::new();
        if !translator.is_empty() {
            details.push(format!("{translator}, {}", grammar.translator));
        }
        if !edition.is_empty() {
            details.push(apa_bibliography_edition(edition, grammar));
        }
        if !volume.is_empty() {
            details.push(apa_bibliography_volume(volume, grammar));
        }
        if !pages.is_empty() {
            details.push(format!("{}{pages}", grammar.pages));
        }
        if !details.is_empty() {
            suffix.push_str(&format!(" ({})", details.join(", ")));
        }
        suffix.push('.');
    }
    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    let publisher = first_source_property(source, BibliographySourceField::Publisher);
    if !location.is_empty() || !publisher.is_empty() {
        suffix.push(' ');
        suffix.push_str(&location);
        if !location.is_empty() && !publisher.is_empty() {
            suffix.push_str(": ");
        }
        suffix.push_str(publisher);
        suffix.push('.');
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    if book.is_empty() {
        prefix.push_str(&suffix);
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
    } else {
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        paragraph.runs.extend(bibliography_display_runs(book, true));
        paragraph
            .runs
            .extend(bibliography_display_runs(&suffix, false));
    }
    paragraph
}

fn apa_conference_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let mut prefix = apa_author_title_prefix(source, grammar);
    let editor =
        apa_bibliography_names(source, BibliographyContributorRole::Editor, false, grammar);
    let conference = first_source_property(source, BibliographySourceField::ConferenceName);
    let volume = first_source_property(source, BibliographySourceField::Volume);
    let pages = first_source_property(source, BibliographySourceField::Pages);
    if !editor.is_empty() {
        prefix.push(' ');
        if !grammar.in_after && !grammar.in_label.is_empty() {
            prefix.push_str(grammar.in_label);
            prefix.push_str(grammar.in_before_separator);
        }
        prefix.push_str(&format!(
            "{editor} ({}){}",
            grammar.editor,
            if conference.is_empty() { "." } else { "," }
        ));
    }
    if !conference.is_empty() || !volume.is_empty() {
        prefix.push(' ');
    }
    let city = first_source_property(source, BibliographySourceField::City);
    let publisher = first_source_property(source, BibliographySourceField::Publisher);
    let mut suffix = String::new();
    if !pages.is_empty() {
        suffix.push_str(&format!(", {}{pages}.", grammar.pages));
    }
    if !city.is_empty() || !publisher.is_empty() {
        suffix.push(' ');
        suffix.push_str(city);
        if !city.is_empty() && !publisher.is_empty() {
            suffix.push_str(": ");
        }
        suffix.push_str(publisher);
        suffix.push('.');
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    if conference.is_empty() && volume.is_empty() {
        prefix.push_str(&suffix);
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
    } else {
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        if !conference.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(&format!("{conference}."), true));
            if !volume.is_empty() {
                paragraph.runs.extend(bibliography_display_runs(
                    &if grammar.in_after && !grammar.in_label.is_empty() {
                        format!("{}{} ", grammar.in_separator, grammar.in_label)
                    } else {
                        " ".to_owned()
                    },
                    false,
                ));
            }
        }
        if !volume.is_empty() {
            if conference.is_empty() && grammar.in_after && !grammar.in_label.is_empty() {
                paragraph.runs.extend(bibliography_display_runs(
                    &format!("{} ", grammar.in_label),
                    false,
                ));
            }
            paragraph.runs.extend(bibliography_display_runs(
                &format!("{volume}{}", if pages.is_empty() { "." } else { "" }),
                true,
            ));
        }
        paragraph
            .runs
            .extend(bibliography_display_runs(&suffix, false));
    }
    paragraph
}

fn apa_film_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let mut creators = Vec::new();
    for (role, label) in [
        (BibliographyContributorRole::ProducerName, grammar.producer),
        (BibliographyContributorRole::Writer, grammar.writer),
        (BibliographyContributorRole::Director, grammar.director),
    ] {
        let name = apa_bibliography_names(source, role, true, grammar);
        if !name.is_empty() {
            creators.push(format!("{name} ({label})"));
        }
    }
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    let mut suffix = String::new();
    if creators.is_empty() && !title.is_empty() {
        paragraph
            .runs
            .extend(bibliography_display_runs(title, true));
        suffix.push_str(&format!(" ({date})."));
    } else {
        let prefix = if creators.is_empty() {
            format!("({date}).")
        } else {
            let names = if creators.len() == 1 {
                creators[0].clone()
            } else {
                format!(
                    "{}{}{} {}",
                    creators[..creators.len() - 1].join(grammar.contributor_separator),
                    grammar.contributor_separator,
                    grammar.conjunction,
                    creators.last().unwrap()
                )
            };
            format!(
                "{names}. ({date}).{}",
                if title.is_empty() { "" } else { " " }
            )
        };
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        if !title.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(title, true));
        }
    }
    suffix.push_str(&format!(" [{}].", grammar.motion_picture));
    let country = first_source_property(source, BibliographySourceField::CountryRegion);
    let distributor = first_source_property(source, BibliographySourceField::Distributor);
    if !country.is_empty() || !distributor.is_empty() {
        suffix.push(' ');
        suffix.push_str(country);
        if !country.is_empty() && !distributor.is_empty() {
            suffix.push_str(": ");
        }
        suffix.push_str(distributor);
        suffix.push('.');
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    if title.is_empty() && paragraph.runs.len() == 1 {
        suffix.insert_str(0, &paragraph.runs.pop().unwrap().text());
    }
    paragraph
        .runs
        .extend(bibliography_display_runs(&suffix, false));
    paragraph
}

fn apa_site_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let creator =
        apa_bibliography_names(source, BibliographyContributorRole::Author, true, grammar);
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    let version = first_source_property(source, BibliographySourceField::Version);
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    let mut suffix = String::new();
    if creator.is_empty() && !title.is_empty() {
        paragraph
            .runs
            .extend(bibliography_display_runs(title, true));
        suffix.push_str(&format!(". ({date})."));
    } else {
        let prefix = if creator.is_empty() {
            format!("({date}).")
        } else {
            format!(
                "{creator} ({date}).{}",
                if title.is_empty() { "" } else { " " }
            )
        };
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        if !title.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(title, true));
            if !version.is_empty() {
                suffix.push_str(&format!(", {version}"));
            }
            suffix.push('.');
        }
    }
    let editor =
        apa_bibliography_names(source, BibliographyContributorRole::Editor, false, grammar);
    let producer = apa_bibliography_names(
        source,
        BibliographyContributorRole::ProducerName,
        false,
        grammar,
    );
    let company = first_source_property(source, BibliographySourceField::ProductionCompany);
    let mut secondary = Vec::new();
    if !editor.is_empty() {
        secondary.push(format!("{editor}, {}", grammar.site_editor));
    }
    if !producer.is_empty() {
        secondary.push(format!("{producer}, {}", grammar.producer));
    }
    if !company.is_empty() {
        secondary.push(company.to_owned());
    }
    if !secondary.is_empty() {
        let joined = if secondary.len() == 1 {
            secondary[0].clone()
        } else {
            format!(
                "{}{}{} {}",
                secondary[..secondary.len() - 1].join(grammar.contributor_separator),
                grammar.contributor_separator,
                grammar.conjunction,
                secondary.last().unwrap()
            )
        };
        suffix.push_str(&format!(" ({joined})"));
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    if paragraph.runs.len() == 1 && title.is_empty() {
        let prefix = paragraph.runs.pop().unwrap().text();
        suffix.insert_str(0, &prefix);
    }
    paragraph
        .runs
        .extend(bibliography_display_runs(&suffix, false));
    paragraph
}

fn apa_sound_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let creator =
        apa_bibliography_names(source, BibliographyContributorRole::Composer, true, grammar);
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    let performer = apa_bibliography_names(
        source,
        BibliographyContributorRole::Performer,
        false,
        grammar,
    );
    let mut prefix = if creator.is_empty() {
        if title.is_empty() {
            format!("({date}).")
        } else {
            format!("{title}. ({date}).")
        }
    } else {
        format!(
            "{creator} ({date}).{}",
            if title.is_empty() {
                String::new()
            } else {
                format!(" {title}")
            }
        )
    };
    if !performer.is_empty() {
        prefix.push_str(&if grammar.recorded_after {
            format!(
                " [{}{performer}{}{}].",
                grammar.recorded_prefix, grammar.recorded_separator, grammar.recorded_by
            )
        } else {
            format!(
                " [{}{}{performer}].",
                grammar.recorded_by, grammar.recorded_before_separator
            )
        });
    } else if !creator.is_empty() && !title.is_empty() {
        prefix.push('.');
    }
    let album = first_source_property(source, BibliographySourceField::AlbumTitle);
    let medium = first_source_property(source, BibliographySourceField::Medium);
    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    let producer = apa_bibliography_names(
        source,
        BibliographyContributorRole::ProducerName,
        false,
        grammar,
    );
    let mut suffix = if medium.is_empty() {
        if album.is_empty() {
            String::new()
        } else {
            ".".to_owned()
        }
    } else {
        format!(" [{medium}].")
    };
    if !location.is_empty() || !producer.is_empty() {
        suffix.push(' ');
        suffix.push_str(&location);
        if !location.is_empty() && !producer.is_empty() {
            suffix.push_str(": ");
        }
        suffix.push_str(&producer);
        suffix.push('.');
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    if album.is_empty() {
        prefix.push_str(&suffix);
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
    } else {
        prefix.push(' ');
        if !grammar.on_after && !grammar.on_label.is_empty() {
            prefix.push_str(grammar.on_label);
            prefix.push_str(grammar.on_before_separator);
        }
        if grammar.on_after && !grammar.on_label.is_empty() {
            suffix.insert_str(0, &format!("{}{}", grammar.on_separator, grammar.on_label));
        }
        if grammar.quote_album {
            prefix.push('"');
            suffix.insert(0, '"');
        }
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        paragraph
            .runs
            .extend(bibliography_display_runs(album, true));
        paragraph
            .runs
            .extend(bibliography_display_runs(&suffix, false));
    }
    paragraph
}

fn apa_art_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let creator =
        apa_bibliography_names(source, BibliographyContributorRole::Artist, true, grammar);
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    let publication = first_source_property(source, BibliographySourceField::PublicationTitle);
    let mut prefix = if creator.is_empty() {
        format!("({date}).")
    } else {
        format!(
            "{creator} ({date}).{}",
            if title.is_empty() { "" } else { " " }
        )
    };
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    if !publication.is_empty() {
        if !title.is_empty() {
            prefix.push_str(&format!("{title}. "));
        }
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        paragraph
            .runs
            .extend(bibliography_display_runs(&format!("{publication}."), true));
    } else {
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        if !title.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(&format!("{title}."), true));
        }
    }
    let location = [
        BibliographySourceField::Institution,
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    let mut suffix = if location.is_empty() {
        if title.is_empty() && publication.is_empty() {
            String::new()
        } else {
            " ".to_owned()
        }
    } else {
        format!(" {location}.")
    };
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    paragraph
        .runs
        .extend(bibliography_display_runs(&suffix, false));
    paragraph
}

fn apa_performance_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let creator =
        apa_bibliography_names(source, BibliographyContributorRole::Writer, true, grammar);
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    let mut suffix = String::new();
    if creator.is_empty() && !title.is_empty() {
        paragraph
            .runs
            .extend(bibliography_display_runs(&format!("{title}."), true));
        suffix.push_str(&format!(" ({date})."));
    } else {
        let prefix = if creator.is_empty() {
            format!("({date}).")
        } else {
            format!(
                "{creator} ({date}).{}",
                if title.is_empty() { "" } else { " " }
            )
        };
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        if !title.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(&format!("{title}."), true));
        }
    }
    let director = apa_bibliography_names(
        source,
        BibliographyContributorRole::Director,
        false,
        grammar,
    );
    let performer = apa_bibliography_names(
        source,
        BibliographyContributorRole::Performer,
        false,
        grammar,
    );
    let mut secondary = Vec::new();
    if !director.is_empty() {
        secondary.push(format!("{director}, {}", grammar.director));
    }
    if !performer.is_empty() {
        secondary.push(format!("{performer}, {}", grammar.performer));
    }
    if !secondary.is_empty() {
        suffix.push_str(&format!(
            " ({})",
            secondary.join(&format!(
                "{}{} ",
                grammar.contributor_separator, grammar.conjunction
            ))
        ));
    }
    let location = [
        BibliographySourceField::Theater,
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    if !location.is_empty() {
        suffix.push_str(&format!(" {location}."));
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    paragraph
        .runs
        .extend(bibliography_display_runs(&suffix, false));
    paragraph
}

fn apa_bibliography_secondary(
    source: &BibliographySource,
    editor_fallback: bool,
    grammar: &ApaBibliographyLanguage,
) -> String {
    let count = |role| {
        source
            .contributors
            .iter()
            .filter(|value| value.role == role)
            .map(|value| match &value.value {
                BibliographyAuthor::People(people) => people.len(),
                BibliographyAuthor::Corporate(_) => 1,
            })
            .sum::<usize>()
    };
    let editor =
        apa_bibliography_names(source, BibliographyContributorRole::Editor, false, grammar);
    let translator = apa_bibliography_names(
        source,
        BibliographyContributorRole::Translator,
        false,
        grammar,
    );
    let mut secondary = Vec::new();
    if !editor.is_empty() && !editor_fallback {
        secondary.push(format!(
            "{editor}, {}",
            if count(BibliographyContributorRole::Editor) > 1 {
                grammar.editors
            } else {
                grammar.editor
            }
        ));
    }
    if !translator.is_empty() {
        secondary.push(format!(
            "{translator}, {}",
            if count(BibliographyContributorRole::Translator) > 1 {
                grammar.translators
            } else {
                grammar.translator
            }
        ));
    }
    if secondary.is_empty() {
        String::new()
    } else {
        format!(
            " ({})",
            secondary.join(&if count(BibliographyContributorRole::Translator) > 1 {
                grammar.contributor_separator.to_owned()
            } else {
                format!(
                    "{}{}{}",
                    grammar.contributor_separator,
                    grammar.conjunction,
                    if grammar.conjunction.is_empty() {
                        ""
                    } else {
                        " "
                    }
                )
            })
        )
    }
}

fn apa_periodical_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    let author = apa_bibliography_names(source, BibliographyContributorRole::Author, true, grammar);
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    let mut prefix = if author.is_empty() {
        if title.is_empty() {
            format!("({date}).")
        } else {
            format!("{title}. ({date}).")
        }
    } else {
        format!(
            "{author} ({date}).{}",
            if title.is_empty() {
                String::new()
            } else {
                format!(" {title}.")
            }
        )
    };
    prefix.push_str(&apa_bibliography_secondary(source, false, grammar));
    let periodical = first_source_property(
        source,
        if source.kind == BibliographySourceKind::JournalArticle {
            BibliographySourceField::JournalName
        } else {
            BibliographySourceField::PeriodicalTitle
        },
    );
    let volume = first_source_property(source, BibliographySourceField::Volume);
    let issue = first_source_property(source, BibliographySourceField::Issue);
    let pages = first_source_property(source, BibliographySourceField::Pages);
    let mut heading = periodical.to_owned();
    if !volume.is_empty() {
        if !heading.is_empty() {
            heading.push_str(", ");
        }
        heading.push_str(volume);
    }
    let mut suffix = String::new();
    if !issue.is_empty() {
        suffix.push_str(&format!("({issue})"));
    }
    if !pages.is_empty() {
        if !heading.is_empty() || !suffix.is_empty() {
            suffix.push_str(", ");
        }
        if source.kind == BibliographySourceKind::ArticleInAPeriodical {
            suffix.push_str(grammar.pages);
        }
        suffix.push_str(pages);
    }
    if !heading.is_empty() || !suffix.is_empty() {
        suffix.push('.');
    }
    suffix.push_str(&apa_bibliography_retrieval(source, grammar));
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    if !heading.is_empty() {
        prefix.push(' ');
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
        paragraph
            .runs
            .extend(bibliography_display_runs(&heading, true));
        paragraph
            .runs
            .extend(bibliography_display_runs(&suffix, false));
    } else {
        if !suffix.is_empty() && !suffix.starts_with(' ') {
            prefix.push(' ');
        }
        prefix.push_str(&suffix);
        paragraph
            .runs
            .extend(bibliography_display_runs(&prefix, false));
    }
    paragraph
}

fn append_apa_semantic_run(
    paragraph: &mut rdocx_oxml::text::CT_P,
    grammar: &ApaBibliographyLanguage,
    text: &str,
    italic: bool,
    rtl: bool,
) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    let italic = italic && !grammar.no_italic;
    let properties = rdocx_oxml::CT_RPr {
        no_proof: Some(true),
        font_cs: if rtl {
            grammar.rtl_cs_font.map(str::to_owned)
        } else {
            None
        },
        font_hint: if grammar.language_east_asia.is_some() {
            Some("eastAsia".into())
        } else {
            grammar.literal_font.is_none().then(|| "cs".into())
        },
        language: grammar.language.map(str::to_owned),
        language_bidi: grammar.language_bidi.map(str::to_owned),
        language_east_asia: grammar.language_east_asia.map(str::to_owned),
        italic: italic.then_some(true),
        italic_cs: italic.then_some(true),
        rtl: (rtl && grammar.semantic_rtl).then_some(true),
        ..Default::default()
    };
    if matches!(
        grammar.language_bidi,
        Some(
            "ug-CN" | "tzm-Arab-MA" | "ur-PK" | "ur-IN" | "yi-Hebr" | "he-IL" | "dv-MV" | "syr-SY"
        )
    ) && !rtl
    {
        // Own paired Title/Last source probes distinguish a measured source-script span, its
        // numeric/neutral suffix, and the next Latin span. Generated operands
        // retain their separate caller-provided direction.
        let source_script = match grammar.language_bidi {
            Some("yi-Hebr" | "he-IL") => '\u{590}'..='\u{5ff}',
            Some("syr-SY") => '\u{700}'..='\u{74f}',
            Some("dv-MV") => '\u{780}'..='\u{7bf}',
            _ => '\u{600}'..='\u{6ff}',
        };
        let mut rtl_source = false;
        for (offset, character) in text.char_indices() {
            if source_script.contains(&character) {
                rtl_source = true;
            } else if character.is_ascii_alphabetic() {
                rtl_source = false;
            } else if character == ' '
                && text[offset + character.len_utf8()..]
                    .chars()
                    .next()
                    .is_some_and(|next| source_script.contains(&next))
            {
                rtl_source = true;
            }
            let mut actual = properties.clone();
            actual.rtl = rtl_source.then_some(true);
            if matches!(grammar.language_bidi, Some("dv-MV" | "syr-SY")) {
                // Completed own source and generated neutral controls share the own cs font.
                // Keep the measured Latin suffix free of an explicit cs font.
                actual.font_cs = if rtl_source {
                    grammar.rtl_cs_font.map(str::to_owned)
                } else {
                    None
                };
            }
            append_apa_semantic_text(paragraph, &character.to_string(), actual)?;
        }
        return Ok(());
    }
    if rtl && grammar.language_bidi == Some("syr-SY") {
        // Own completed Syriac controls distinguish neutral generated glue from
        // translated Arabic labels. Original source operands enter separately.
        let previous_arabic = paragraph
            .runs
            .iter()
            .rev()
            .find_map(|run| {
                run.properties
                    .as_ref()
                    .filter(|actual| actual.rtl == Some(true))
            })
            .is_some_and(|actual| actual.language_bidi.is_none());
        let first_arabic = text
            .char_indices()
            .find(|(_, character)| ('\u{600}'..='\u{6ff}').contains(character))
            .map(|(offset, _)| offset);
        let label_start = if text.trim_start_matches(' ').starts_with('(') {
            first_arabic.unwrap_or(0)
        } else {
            text.len() - text.trim_start_matches(' ').len()
        };
        let continuation =
            first_arabic.is_none() && previous_arabic && !text.starts_with([' ', '(']);
        let trailing_glue = text.ends_with(&format!("{} ", grammar.conjunction))
            || (text == grammar.contributor_separator
                && paragraph.runs.last().is_some_and(|run| {
                    run.text().ends_with(')')
                        && run.properties.as_ref().is_some_and(|actual| {
                            actual.rtl == Some(true) && actual.language_bidi.is_none()
                        })
                }));
        for (offset, character) in text.char_indices() {
            let mut actual = properties.clone();
            if (first_arabic.is_some() && offset >= label_start || continuation)
                && !(trailing_glue && offset + character.len_utf8() == text.len())
            {
                actual.font_cs = None;
                actual.language_bidi = None;
            }
            append_apa_semantic_text(paragraph, &character.to_string(), actual)?;
        }
        return Ok(());
    }
    if let Some(font) = grammar.literal_font {
        // Keep date-unit literals separate from numeric and source operands.
        for character in text.chars() {
            let mut actual = properties.clone();
            // Own Japanese/Korean operands distinguish CJK and fullwidth comma
            // from Hangul. Keep that measured EastAsia font boundary separate from
            // the locale's generated-label font and from ASCII punctuation.
            let character_font = if matches!(
                grammar.language_east_asia,
                Some("ii-CN" | "zh-CN" | "zh-SG")
            ) && "专员处导录检电编访译辑页汉".contains(character)
            {
                Some("Microsoft JhengHei")
            } else if grammar.language_east_asia == Some("ii-CN")
                && ('\u{a000}'..='\u{a48f}').contains(&character)
            {
                Some("Microsoft Yi Baiti")
            } else if (grammar.language_east_asia.is_some() && character == '\u{ff0c}')
                || (matches!(grammar.language_east_asia, Some("ja-JP" | "ko-KR"))
                    && ('\u{3040}'..='\u{9fff}').contains(&character))
            {
                Some("MS Gothic")
            } else if grammar
                .literal_script
                .is_some_and(|(first, last)| (first..=last).contains(&character))
            {
                Some(font)
            } else {
                None
            };
            if let Some(font) = character_font {
                actual.font_ascii = Some(font.to_owned());
                actual.font_hansi = Some(font.to_owned());
                actual.font_cs = Some(font.to_owned());
                actual.font_east_asia = grammar.language_east_asia.map(|_| font.to_owned());
            }
            append_apa_semantic_text(paragraph, &character.to_string(), actual)?;
        }
        return Ok(());
    }
    if let Some(font) = grammar.numeric_cs_font {
        let (first_script, last_script) = grammar.literal_script.unwrap_or(('\u{900}', '\u{97f}'));
        // These measured ASCII and own-script operations are not a full script algorithm.
        // Source spans and generated glue enter separately before text is joined.
        if !text.chars().all(|character| {
            character.is_ascii()
                || matches!(character, '\u{a0}' | '\u{600}'..='\u{6ff}')
                || (first_script..=last_script).contains(&character)
                || (grammar.language_bidi == Some("ml-IN")
                    && matches!(character, '\u{200c}' | '\u{200d}'))
        }) {
            return Err(bibliography_error(
                "bibliography complex-script source operation is still being implemented",
            ));
        }
        // Preserve source XML whitespace. This operation normalizes only emitted cache text.
        let normalized = text.replace(['\t', '\n', '\r'], " ");
        let text = if rtl {
            if paragraph
                .runs
                .last()
                .is_some_and(|run| run.text().ends_with(' '))
            {
                normalized.strip_prefix(' ').unwrap_or(&normalized)
            } else {
                &normalized
            }
        } else {
            normalized.trim_start_matches(' ')
        };
        let mut numeric = rtl
            && paragraph
                .runs
                .last()
                .and_then(|run| run.properties.as_ref())
                .is_some_and(|properties| properties.complex_script == Some(true));
        let mut arabic = false;
        let mut after_comma = false;
        for (offset, character) in text.char_indices() {
            if matches!(character, '\u{600}'..='\u{6ff}') {
                arabic = true;
                numeric = false;
                after_comma = false;
            } else if (first_script..=last_script).contains(&character) {
                arabic = false;
                numeric = true;
                after_comma = false;
            } else if character.is_ascii_digit() {
                numeric = !arabic;
                after_comma = false;
            } else if matches!(character, ',' | ';' | '{' | '}' | '\u{a0}') {
                arabic = false;
                numeric = false;
                after_comma = true;
            } else if character.is_ascii_alphabetic() || character == '&' {
                arabic = false;
                numeric = false;
                after_comma = false;
            } else if matches!(character, '+' | '-') {
                arabic = false;
                numeric = true;
                after_comma = false;
            } else if offset == 0 && matches!(character, '(' | '[') {
                numeric = text[character.len_utf8()..]
                    .chars()
                    .find(|next| *next != ' ')
                    .is_some_and(|next| next.is_ascii_digit());
            } else if arabic && character == ' ' {
                if text[offset + 1..]
                    .chars()
                    .find(|next| *next != ' ')
                    .is_some_and(|next| next.is_ascii_alphabetic())
                {
                    arabic = false;
                    numeric = false;
                }
            } else if rtl && character == ' ' {
                numeric = !after_comma
                    || text[offset + 1..].chars().find(|next| *next != ' ') == Some('&');
            } else if !matches!(
                character,
                ' ' | '(' | ')' | '[' | ']' | '.' | ':' | '/' | '-'
            ) && !(grammar.language_bidi == Some("ml-IN")
                && matches!(character, '\u{200c}' | '\u{200d}'))
            {
                return Err(bibliography_error(
                    "bibliography complex-script neutral operation is still being implemented",
                ));
            }
            let mut actual = properties.clone();
            if arabic {
                actual.language = None;
                actual.language_bidi = None;
                actual.rtl = Some(true);
            } else if numeric {
                actual.font_cs = Some(font.to_owned());
                actual.complex_script = Some(true);
            }
            // Own source and generated Va controls share this exact measured grapheme.
            // This does not classify font coverage for other Odia operands.
            if grammar.language_bidi == Some("or-IN")
                && ((character == '\u{b35}' && text[offset..].starts_with("ଵା"))
                    || (character == '\u{b3e}' && text[..offset].ends_with('\u{b35}')))
            {
                actual.font_ascii = Some("Noto Sans Oriya".to_owned());
                actual.font_hansi = Some("Noto Sans Oriya".to_owned());
                actual.font_cs = Some("Noto Sans Oriya".to_owned());
                actual.complex_script = Some(true);
            }
            append_apa_semantic_text(paragraph, &character.to_string(), actual)?;
        }
        Ok(())
    } else {
        append_apa_semantic_text(paragraph, text, properties)
    }
}

fn append_apa_semantic_text(
    paragraph: &mut rdocx_oxml::text::CT_P,
    text: &str,
    properties: rdocx_oxml::CT_RPr,
) -> Result<()> {
    use rdocx_oxml::text::{CT_R, RunContent};
    if let Some(last) = paragraph.runs.last_mut()
        && last.properties.as_ref() == Some(&properties)
    {
        let [RunContent::Text(content)] = last.content.as_mut_slice() else {
            return Err(bibliography_error("invalid semantic bibliography text run"));
        };
        content.text.push_str(text);
        content.preserve_space = content.text.starts_with(' ') || content.text.ends_with(' ');
    } else {
        let mut run = CT_R::new(text);
        run.properties = Some(properties);
        paragraph.runs.push(run);
    }
    Ok(())
}

fn append_apa_semantic_secondary(
    paragraph: &mut rdocx_oxml::text::CT_P,
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<()> {
    let secondary = [
        (BibliographyContributorRole::Editor, grammar.editor),
        (BibliographyContributorRole::Translator, grammar.translator),
    ]
    .into_iter()
    .map(|(role, label)| (apa_bibliography_names(source, role, false, grammar), label))
    .filter(|(name, _)| !name.is_empty())
    .collect::<Vec<_>>();
    if !secondary.is_empty() {
        append_apa_semantic_run(paragraph, grammar, " (", false, true)?;
        for (index, (name, label)) in secondary.iter().enumerate() {
            if grammar.rtl_labels {
                append_apa_semantic_run(paragraph, grammar, name, false, false)?;
                append_apa_semantic_run(
                    paragraph,
                    grammar,
                    &format!("{}{label}", grammar.contributor_separator),
                    false,
                    true,
                )?;
                if index + 1 < secondary.len() {
                    append_apa_semantic_run(
                        paragraph,
                        grammar,
                        &format!(
                            "{}{}{}",
                            grammar.contributor_separator,
                            grammar.conjunction,
                            if grammar.conjunction.is_empty() {
                                ""
                            } else {
                                " "
                            }
                        ),
                        false,
                        true,
                    )?;
                }
                continue;
            }
            let (label_text, terminal) = label
                .strip_suffix('.')
                .map_or((*label, ""), |text| (text, "."));
            append_apa_semantic_run(
                paragraph,
                grammar,
                &format!("{name}, {label_text}"),
                false,
                false,
            )?;
            append_apa_semantic_run(
                paragraph,
                grammar,
                terminal,
                false,
                grammar.numeric_cs_font.is_none()
                    || label_text.chars().any(|character| {
                        let (first, last) =
                            grammar.literal_script.unwrap_or(('\u{900}', '\u{97f}'));
                        (first..=last).contains(&character)
                    }),
            )?;
            if index + 1 < secondary.len() {
                if grammar.numeric_cs_font.is_some() {
                    append_apa_semantic_run(
                        paragraph,
                        grammar,
                        grammar.contributor_separator.trim_end(),
                        false,
                        false,
                    )?;
                    append_apa_semantic_run(paragraph, grammar, " ", false, true)?;
                    append_apa_semantic_run(paragraph, grammar, grammar.conjunction, false, false)?;
                    append_apa_semantic_run(paragraph, grammar, " ", false, true)?;
                } else {
                    append_apa_semantic_run(
                        paragraph,
                        grammar,
                        &format!(
                            "{}{}{}",
                            grammar.contributor_separator,
                            grammar.conjunction,
                            if grammar.conjunction.is_empty() {
                                ""
                            } else {
                                " "
                            }
                        ),
                        false,
                        true,
                    )?;
                }
            }
        }
        append_apa_semantic_run(
            paragraph,
            grammar,
            ")",
            false,
            grammar.numeric_cs_font.is_none()
                || secondary.last().is_some_and(|(_, label)| {
                    label.chars().any(|character| {
                        let (first, last) =
                            grammar.literal_script.unwrap_or(('\u{900}', '\u{97f}'));
                        (first..=last).contains(&character)
                    })
                }),
        )?;
    }
    Ok(())
}

fn append_apa_semantic_title(
    paragraph: &mut rdocx_oxml::text::CT_P,
    grammar: &ApaBibliographyLanguage,
    title: &str,
    italic: bool,
    quoted: bool,
    terminal: bool,
) -> Result<()> {
    if title.is_empty() {
        return Ok(());
    }
    if quoted {
        append_apa_semantic_run(paragraph, grammar, "“", italic, true)?;
    }
    append_apa_semantic_run(paragraph, grammar, title, italic, false)?;
    if quoted {
        append_apa_semantic_run(
            paragraph,
            grammar,
            if terminal { ".”" } else { "”" },
            italic,
            true,
        )?;
    }
    Ok(())
}

fn apa_semantic_creator_paragraph(
    source: &BibliographySource,
    role: BibliographyContributorRole,
    grammar: &ApaBibliographyLanguage,
    title_italic: bool,
) -> Result<rdocx_oxml::text::CT_P> {
    use rdocx_oxml::text::CT_P;
    let mut paragraph = CT_P::new();
    let mut properties = bibliography_paragraph_properties(BibliographyStyle::ApaSixthEdition);
    properties.bidi = grammar.semantic_rtl.then_some(true);
    if let Some(mark) = properties.rpr.as_mut() {
        mark.language = grammar.language.map(str::to_owned);
        mark.language_bidi = grammar.language_bidi.map(str::to_owned);
        mark.language_east_asia = grammar.language_east_asia.map(str::to_owned);
    }
    paragraph.properties = Some(properties);
    let author = apa_bibliography_names(source, role, true, grammar);
    let date = apa_bibliography_date(source, grammar);
    let title = if source.kind == BibliographySourceKind::Patent {
        ""
    } else {
        first_source_property(source, BibliographySourceField::Title)
    };
    // The final initial mark is generated name grammar. Internal source punctuation
    // remains inside the author operand, including the measured BN discriminator.
    let generated_initial = grammar.numeric_cs_font.is_none()
        && !grammar.full_names
        && !grammar.joined_names
        && source.contributors.iter().any(|contributor| {
            contributor.role == role
                && matches!(&contributor.value, BibliographyAuthor::People(people)
                if people.last().is_some_and(|person| !person.last.is_empty()
                    && (!person.first.is_empty() || !person.middle.is_empty())))
        });
    let author_text = if generated_initial {
        author.strip_suffix(grammar.initial_mark).unwrap_or(&author)
    } else {
        &author
    };
    append_apa_semantic_run(&mut paragraph, grammar, author_text, false, false)?;
    append_apa_semantic_run(
        &mut paragraph,
        grammar,
        &format!(
            "{} ({date}).{}",
            if generated_initial {
                if grammar.initial_mark == '\'' && role != BibliographyContributorRole::Composer {
                    "'.".to_owned()
                } else {
                    grammar.initial_mark.to_string()
                }
            } else if (grammar.full_names || grammar.joined_names)
                && role != BibliographyContributorRole::Composer
            {
                ".".to_owned()
            } else {
                String::new()
            },
            if title.is_empty() { "" } else { " " }
        ),
        false,
        true,
    )?;
    append_apa_semantic_title(
        &mut paragraph,
        grammar,
        title,
        title_italic,
        grammar.curly_titles
            && matches!(
                source.kind,
                BibliographySourceKind::Book
                    | BibliographySourceKind::Report
                    | BibliographySourceKind::DocumentFromInternetSite
                    | BibliographySourceKind::Performance
                    | BibliographySourceKind::InternetSite
            ),
        matches!(
            source.kind,
            BibliographySourceKind::Report
                | BibliographySourceKind::DocumentFromInternetSite
                | BibliographySourceKind::Performance
        ),
    )?;
    Ok(paragraph)
}

fn append_apa_semantic_retrieval(
    paragraph: &mut rdocx_oxml::text::CT_P,
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<()> {
    let retrieval = apa_bibliography_retrieval(source, grammar);
    if grammar.rtl_labels {
        let url = first_source_property(source, BibliographySourceField::Url);
        let site = first_source_property(source, BibliographySourceField::InternetSiteTitle);
        let operand = if site.is_empty() {
            url.to_owned()
        } else if url.is_empty() {
            format!("{site}.")
        } else {
            format!("{site}: {url}")
        };
        if !operand.is_empty() && grammar.retrieval_date_after {
            // The lexical consumer already assembles the source-valued date.
            // Match its exact generated prefix and operand, never a substring
            // that could also occur inside a source URL or site title.
            let no_date = first_source_property(source, BibliographySourceField::MonthAccessed)
                .is_empty()
                && first_source_property(source, BibliographySourceField::YearAccessed).is_empty();
            let prefix = format!(
                "{}{}",
                if no_date {
                    grammar.retrieved_without_date.unwrap_or(grammar.retrieved)
                } else {
                    grammar.retrieved
                },
                if no_date {
                    grammar.from_without_date
                } else {
                    grammar.from_with_date
                }
            );
            let suffix = retrieval
                .strip_prefix(&prefix)
                .and_then(|rest| rest.strip_prefix(&operand))
                .ok_or_else(|| bibliography_error("invalid RTL post-URL retrieval operand"))?;
            append_apa_semantic_run(paragraph, grammar, &prefix, false, true)?;
            append_apa_semantic_run(paragraph, grammar, &operand, false, false)?;
            append_apa_semantic_run(paragraph, grammar, suffix, false, true)?;
        } else if !operand.is_empty() {
            let before_suffix = retrieval
                .strip_suffix(grammar.retrieval_suffix)
                .ok_or_else(|| bibliography_error("invalid RTL retrieval suffix"))?;
            let literal = before_suffix
                .strip_suffix(&operand)
                .ok_or_else(|| bibliography_error("invalid RTL retrieval operand"))?;
            append_apa_semantic_run(paragraph, grammar, literal, false, true)?;
            append_apa_semantic_run(paragraph, grammar, &operand, false, false)?;
            append_apa_semantic_run(paragraph, grammar, grammar.retrieval_suffix, false, true)?;
        }
    } else if let Some(retrieval) = retrieval.strip_prefix(' ') {
        append_apa_semantic_run(paragraph, grammar, " ", false, true)?;
        append_apa_semantic_run(paragraph, grammar, retrieval, false, false)?;
    } else {
        append_apa_semantic_run(paragraph, grammar, &retrieval, false, false)?;
    }
    Ok(())
}

fn append_apa_semantic_source_list(
    paragraph: &mut rdocx_oxml::text::CT_P,
    grammar: &ApaBibliographyLanguage,
    operands: &[&str],
) -> Result<()> {
    for (index, operand) in operands.iter().filter(|text| !text.is_empty()).enumerate() {
        if index != 0 {
            append_apa_semantic_run(
                paragraph,
                grammar,
                grammar.location_separator,
                false,
                grammar.rtl_labels
                    && !matches!(
                        grammar.language_bidi,
                        Some(
                            "ug-CN"
                                | "ur-PK"
                                | "ur-IN"
                                | "he-IL"
                                | "fa-IR"
                                | "sd-Arab-PK"
                                | "pa-Arab-PK"
                                | "ps-AF"
                        )
                    ),
            )?;
        }
        append_apa_semantic_run(paragraph, grammar, operand, false, false)?;
    }
    Ok(())
}

fn append_apa_semantic_labeled_roles(
    paragraph: &mut rdocx_oxml::text::CT_P,
    grammar: &ApaBibliographyLanguage,
    roles: &[(String, &str)],
) -> Result<()> {
    if roles.is_empty() {
        return Ok(());
    }
    append_apa_semantic_run(paragraph, grammar, " (", false, true)?;
    for (index, (name, label)) in roles.iter().enumerate() {
        append_apa_semantic_run(paragraph, grammar, name, false, false)?;
        if !label.is_empty() {
            append_apa_semantic_run(
                paragraph,
                grammar,
                &format!("{}{label}", grammar.contributor_separator),
                false,
                true,
            )?;
        }
        if index + 1 != roles.len() {
            append_apa_semantic_run(
                paragraph,
                grammar,
                &format!(
                    "{}{}",
                    grammar.contributor_separator,
                    if index + 2 == roles.len() {
                        format!(
                            "{}{}",
                            grammar.conjunction,
                            if grammar.conjunction.is_empty() {
                                ""
                            } else {
                                " "
                            }
                        )
                    } else {
                        String::new()
                    }
                ),
                false,
                true,
            )?;
        }
    }
    append_apa_semantic_run(paragraph, grammar, ")", false, true)
}

fn apa_rtl_site_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph =
        apa_semantic_creator_paragraph(source, BibliographyContributorRole::Author, grammar, true)?;
    let title = first_source_property(source, BibliographySourceField::Title);
    let version = first_source_property(source, BibliographySourceField::Version);
    if !title.is_empty() {
        if !version.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}{version}", grammar.contributor_separator),
                false,
                grammar.language_bidi != Some("ug-CN"),
            )?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    let editor =
        apa_bibliography_names(source, BibliographyContributorRole::Editor, false, grammar);
    let producer = apa_bibliography_names(
        source,
        BibliographyContributorRole::ProducerName,
        false,
        grammar,
    );
    let company = first_source_property(source, BibliographySourceField::ProductionCompany);
    let mut roles = Vec::new();
    if !editor.is_empty() {
        roles.push(format!("{editor}, {}", grammar.site_editor));
    }
    if !producer.is_empty() {
        roles.push(format!("{producer}, {}", grammar.producer));
    }
    if !company.is_empty() {
        roles.push(company.to_owned());
    }
    if grammar.rtl_labels {
        let mut labeled = Vec::new();
        if !editor.is_empty() {
            labeled.push((editor, grammar.site_editor));
        }
        if !producer.is_empty() {
            labeled.push((producer, grammar.producer));
        }
        if !company.is_empty() {
            labeled.push((company.to_owned(), ""));
        }
        append_apa_semantic_labeled_roles(&mut paragraph, grammar, &labeled)?;
    } else if !roles.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
        if roles.len() > 1 {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &roles[..roles.len() - 1].join(grammar.contributor_separator),
                false,
                false,
            )?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!(
                    "{}{}{}",
                    grammar.contributor_separator,
                    grammar.conjunction,
                    if grammar.conjunction.is_empty() {
                        ""
                    } else {
                        " "
                    }
                ),
                false,
                true,
            )?;
        }
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &roles[roles.len() - 1],
            false,
            false,
        )?;
        append_apa_semantic_run(&mut paragraph, grammar, ")", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_sound_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph = apa_semantic_creator_paragraph(
        source,
        BibliographyContributorRole::Composer,
        grammar,
        false,
    )?;
    let performer = apa_bibliography_names(
        source,
        BibliographyContributorRole::Performer,
        false,
        grammar,
    );
    let rtl_recorded = grammar.rtl_labels && grammar.language_bidi != Some("ps-AF");
    if !performer.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " [", false, true)?;
        if grammar.recorded_after {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                grammar.recorded_prefix,
                false,
                false,
            )?;
            append_apa_semantic_run(&mut paragraph, grammar, &performer, false, false)?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}{}", grammar.recorded_separator, grammar.recorded_by),
                false,
                rtl_recorded || grammar.numeric_cs_font.is_some(),
            )?;
        } else {
            let recorded = if rtl_recorded {
                format!("{} ", grammar.recorded_by)
            } else {
                format!("{} {performer}", grammar.recorded_by)
            };
            append_apa_semantic_run(&mut paragraph, grammar, &recorded, false, rtl_recorded)?;
            if rtl_recorded {
                append_apa_semantic_run(&mut paragraph, grammar, &performer, false, false)?;
            }
        }
        append_apa_semantic_run(&mut paragraph, grammar, "].", false, true)?;
    }
    let album = first_source_property(source, BibliographySourceField::AlbumTitle);
    if !album.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        if !grammar.on_after && !grammar.on_label.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                grammar.on_label,
                false,
                grammar.rtl_labels,
            )?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                grammar.on_before_separator,
                false,
                true,
            )?;
        }
        if grammar.quote_album {
            append_apa_semantic_run(&mut paragraph, grammar, "“", false, false)?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, album, true, false)?;
        if grammar.quote_album {
            append_apa_semantic_run(&mut paragraph, grammar, "”", false, false)?;
        }
        if grammar.on_after && !grammar.on_label.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}{}", grammar.on_separator, grammar.on_label),
                false,
                grammar.rtl_labels || grammar.numeric_cs_font.is_some(),
            )?;
        }
    }
    let medium = first_source_property(source, BibliographySourceField::Medium);
    if !medium.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " [", false, true)?;
        append_apa_semantic_run(&mut paragraph, grammar, medium, false, false)?;
        append_apa_semantic_run(&mut paragraph, grammar, "].", false, true)?;
    }
    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    let producer = apa_bibliography_names(
        source,
        BibliographyContributorRole::ProducerName,
        false,
        grammar,
    );
    if !location.is_empty() || !producer.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_source_list(&mut paragraph, grammar, &location)?;
        if !location.is_empty() && !producer.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, ": ", false, false)?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, &producer, false, false)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_film_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph = rdocx_oxml::text::CT_P::new();
    let mut properties = bibliography_paragraph_properties(BibliographyStyle::ApaSixthEdition);
    properties.bidi = grammar.semantic_rtl.then_some(true);
    if let Some(mark) = properties.rpr.as_mut() {
        mark.language = grammar.language.map(str::to_owned);
        mark.language_bidi = grammar.language_bidi.map(str::to_owned);
        mark.language_east_asia = grammar.language_east_asia.map(str::to_owned);
    }
    paragraph.properties = Some(properties);
    let roles = [(BibliographyContributorRole::ProducerName, grammar.producer), (BibliographyContributorRole::Writer, grammar.writer), (BibliographyContributorRole::Director, grammar.director)]
        .into_iter().map(|(role,label)| {
            let name = apa_bibliography_names(source, role, true, grammar);
            let generated_initial = grammar.numeric_cs_font.is_none() && !grammar.full_names
        && !grammar.joined_names && source.contributors.iter().any(|contributor| contributor.role == role && matches!(&contributor.value, BibliographyAuthor::People(people) if people.last().is_some_and(|person| !person.last.is_empty() && (!person.first.is_empty() || !person.middle.is_empty()))));
            (name,label,generated_initial)
        }).filter(|(name,_,_)| !name.is_empty()).collect::<Vec<_>>();
    for (index, (name, label, generated_initial)) in roles.iter().enumerate() {
        let name_text = if *generated_initial {
            name.strip_suffix(grammar.initial_mark).unwrap_or(name)
        } else {
            name
        };
        append_apa_semantic_run(&mut paragraph, grammar, name_text, false, false)?;
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!(
                "{} (",
                if *generated_initial {
                    grammar.initial_mark.to_string()
                } else {
                    String::new()
                }
            ),
            false,
            true,
        )?;
        append_apa_semantic_run(&mut paragraph, grammar, label, false, grammar.rtl_labels)?;
        if index + 1 < roles.len() {
            let final_join = index + 2 == roles.len();
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!(
                    "){}{}",
                    grammar.contributor_separator,
                    if final_join {
                        format!(
                            "{}{}",
                            grammar.conjunction,
                            if grammar.conjunction.is_empty() {
                                ""
                            } else {
                                " "
                            }
                        )
                    } else {
                        String::new()
                    }
                ),
                false,
                final_join || grammar.rtl_labels || grammar.numeric_cs_font.is_some(),
            )?;
        } else {
            append_apa_semantic_run(&mut paragraph, grammar, ").", false, true)?;
        }
    }
    let date = apa_bibliography_date(source, grammar);
    let title = first_source_property(source, BibliographySourceField::Title);
    append_apa_semantic_run(
        &mut paragraph,
        grammar,
        &format!(
            "{}({date}).{}",
            if roles.is_empty() { "" } else { " " },
            if title.is_empty() { "" } else { " " }
        ),
        false,
        true,
    )?;
    append_apa_semantic_title(
        &mut paragraph,
        grammar,
        title,
        true,
        grammar.curly_titles,
        false,
    )?;
    if !title.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " [", false, true)?;
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            grammar.motion_picture,
            false,
            grammar.rtl_labels,
        )?;
        append_apa_semantic_run(&mut paragraph, grammar, "].", false, true)?;
    }
    let country = first_source_property(source, BibliographySourceField::CountryRegion);
    let distributor = first_source_property(source, BibliographySourceField::Distributor);
    if !country.is_empty() || !distributor.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_run(&mut paragraph, grammar, country, false, false)?;
        if !country.is_empty() && !distributor.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, ": ", false, false)?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, distributor, false, false)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_chapter_conference_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph = apa_semantic_creator_paragraph(
        source,
        BibliographyContributorRole::Author,
        grammar,
        false,
    )?;
    let title = first_source_property(source, BibliographySourceField::Title);
    if !title.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    let editor =
        apa_bibliography_names(source, BibliographyContributorRole::Editor, false, grammar);
    let book_author = if source.kind == BibliographySourceKind::BookSection {
        apa_bibliography_names(
            source,
            BibliographyContributorRole::BookAuthor,
            false,
            grammar,
        )
    } else {
        String::new()
    };
    if !book_author.is_empty() || !editor.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        if !grammar.in_after && !grammar.in_label.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}{}", grammar.in_label, grammar.in_before_separator),
                false,
                grammar.rtl_labels,
            )?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, &book_author, false, false)?;
        if !book_author.is_empty() && !editor.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!(
                    "{}{}{}",
                    grammar.contributor_separator,
                    grammar.conjunction,
                    if grammar.conjunction.is_empty() {
                        ""
                    } else {
                        " "
                    }
                ),
                false,
                true,
            )?;
        }
        if !editor.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, &editor, false, false)?;
            append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
            let (label, terminal) = grammar
                .editor
                .strip_suffix('.')
                .map_or((grammar.editor, ""), |label| (label, "."));
            append_apa_semantic_run(&mut paragraph, grammar, label, false, grammar.rtl_labels)?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{terminal})"),
                false,
                true,
            )?;
        }
        if grammar.in_after && source.kind == BibliographySourceKind::BookSection {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}{}", grammar.in_separator, grammar.in_label),
                false,
                grammar.rtl_labels || grammar.numeric_cs_font.is_some(),
            )?;
        }
        if grammar.numeric_cs_font.is_some() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                grammar.contributor_separator.trim_end(),
                false,
                false,
            )?;
            append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        } else {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                grammar.contributor_separator,
                false,
                true,
            )?;
        }
    }
    let container = first_source_property(
        source,
        if source.kind == BibliographySourceKind::BookSection {
            BibliographySourceField::BookTitle
        } else {
            BibliographySourceField::ConferenceName
        },
    );
    append_apa_semantic_title(
        &mut paragraph,
        grammar,
        container,
        true,
        grammar.curly_titles && source.kind == BibliographySourceKind::ConferenceProceedings,
        true,
    )?;
    let volume = first_source_property(source, BibliographySourceField::Volume);
    let pages = first_source_property(source, BibliographySourceField::Pages);
    if source.kind == BibliographySourceKind::BookSection {
        let translator = apa_bibliography_names(
            source,
            BibliographyContributorRole::Translator,
            false,
            grammar,
        );
        let edition = first_source_property(source, BibliographySourceField::Edition);
        let mut details = Vec::new();
        if !translator.is_empty() {
            details.push(format!("{translator}, {}", grammar.translator));
        }
        if !edition.is_empty() {
            details.push(apa_bibliography_edition(edition, grammar));
        }
        if !volume.is_empty() {
            details.push(apa_bibliography_volume(volume, grammar));
        }
        if !details.is_empty() || !pages.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
            if grammar.rtl_labels {
                append_apa_semantic_run(&mut paragraph, grammar, &translator, false, false)?;
                let mut labels = Vec::new();
                if !translator.is_empty() {
                    labels.push(grammar.translator.to_owned());
                }
                if !edition.is_empty() {
                    labels.push(apa_bibliography_edition(edition, grammar));
                }
                if !volume.is_empty() {
                    labels.push(apa_bibliography_volume(volume, grammar));
                }
                let prefix = if translator.is_empty() {
                    ""
                } else {
                    grammar.contributor_separator
                };
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    &format!("{prefix}{}", labels.join(grammar.contributor_separator)),
                    false,
                    true,
                )?;
            } else {
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    &details.join(", "),
                    false,
                    false,
                )?;
            }
            if !pages.is_empty() {
                if !details.is_empty() {
                    append_apa_semantic_run(
                        &mut paragraph,
                        grammar,
                        grammar.contributor_separator,
                        false,
                        grammar.rtl_labels,
                    )?;
                }
                let label = grammar.pages.trim_end();
                let (label, terminal) = label
                    .strip_suffix('.')
                    .map_or((label, ""), |label| (label, "."));
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    label,
                    false,
                    grammar.rtl_labels && grammar.language_bidi != Some("ug-CN"),
                )?;
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    &format!("{terminal} {pages}"),
                    false,
                    true,
                )?;
            }
            append_apa_semantic_run(&mut paragraph, grammar, ")", false, true)?;
        }
        if !container.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
        }
    } else {
        if !container.is_empty() && !grammar.curly_titles {
            append_apa_semantic_run(&mut paragraph, grammar, ".", true, true)?;
        }
        if !volume.is_empty() && grammar.in_after && !grammar.in_label.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}{}", grammar.in_separator, grammar.in_label),
                false,
                grammar.rtl_labels || grammar.numeric_cs_font.is_some(),
            )?;
        }
        if !volume.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
            append_apa_semantic_run(&mut paragraph, grammar, volume, true, true)?;
        }
        if !pages.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                grammar.contributor_separator,
                false,
                true,
            )?;
            let label = grammar.pages.trim_end();
            let (label, terminal) = label
                .strip_suffix('.')
                .map_or((label, ""), |label| (label, "."));
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                label,
                false,
                grammar.rtl_labels && grammar.language_bidi != Some("ug-CN"),
            )?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{terminal} {pages}."),
                false,
                true,
            )?;
        }
    }
    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .filter(|field| {
        source.kind == BibliographySourceKind::BookSection
            || *field == BibliographySourceField::City
    })
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    let publisher = first_source_property(source, BibliographySourceField::Publisher);
    if !location.is_empty() || !publisher.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_source_list(&mut paragraph, grammar, &location)?;
        if !location.is_empty() && !publisher.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, ": ", false, false)?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, publisher, false, false)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_general_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph = apa_semantic_creator_paragraph(
        source,
        BibliographyContributorRole::Author,
        grammar,
        false,
    )?;
    let title = first_source_property(source, BibliographySourceField::Title);
    if !title.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    let publication = first_source_property(source, BibliographySourceField::PublicationTitle);
    let volume = first_source_property(source, BibliographySourceField::Volume);
    let issue = first_source_property(
        source,
        if source.kind == BibliographySourceKind::Misc {
            BibliographySourceField::Issue
        } else {
            BibliographySourceField::Edition
        },
    );
    let edition = first_source_property(source, BibliographySourceField::Edition);
    let pages = first_source_property(source, BibliographySourceField::Pages);
    if !publication.is_empty() || !volume.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_title(
            &mut paragraph,
            grammar,
            publication,
            true,
            grammar.curly_titles,
            false,
        )?;
        if !volume.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!(
                    "{}{volume}",
                    if publication.is_empty() {
                        ""
                    } else {
                        grammar.contributor_separator
                    }
                ),
                true,
                grammar.language_bidi != Some("ug-CN"),
            )?;
        }
    }
    if !issue.is_empty() {
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!("({issue})"),
            source.kind == BibliographySourceKind::Misc,
            grammar.language_bidi != Some("ug-CN"),
        )?;
    }
    if source.kind == BibliographySourceKind::Misc && !edition.is_empty() {
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!("{}{edition}", grammar.contributor_separator),
            true,
            grammar.language_bidi != Some("ug-CN"),
        )?;
    }
    if !pages.is_empty() {
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!("{}{pages}", grammar.contributor_separator),
            false,
            grammar.language_bidi != Some("ug-CN"),
        )?;
    }
    if !publication.is_empty() || !volume.is_empty() || !issue.is_empty() || !pages.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    let roles = [
        (BibliographyContributorRole::Editor, grammar.editor),
        (BibliographyContributorRole::Translator, grammar.translator),
        (BibliographyContributorRole::Compiler, grammar.compiler),
    ]
    .into_iter()
    .map(|(role, label)| (apa_bibliography_names(source, role, false, grammar), label))
    .filter(|(name, _)| !name.is_empty())
    .collect::<Vec<_>>();
    if grammar.rtl_labels {
        append_apa_semantic_labeled_roles(&mut paragraph, grammar, &roles)?;
    } else if !roles.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
        if roles.len() > 1 {
            let (preceding, last) = roles.split_at(roles.len() - 1);
            let mut preceding_text = preceding
                .iter()
                .map(|(name, label)| format!("{name}, {label}"))
                .collect::<Vec<_>>()
                .join(grammar.contributor_separator);
            let terminal = if preceding
                .last()
                .is_some_and(|(_, label)| label.ends_with('.'))
            {
                preceding_text.pop();
                "."
            } else {
                ""
            };
            append_apa_semantic_run(&mut paragraph, grammar, &preceding_text, false, false)?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!(
                    "{terminal}{}{}{}",
                    grammar.contributor_separator,
                    grammar.conjunction,
                    if grammar.conjunction.is_empty() {
                        ""
                    } else {
                        " "
                    }
                ),
                false,
                true,
            )?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}, {}", last[0].0, last[0].1),
                false,
                false,
            )?;
        } else {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{}, {}", roles[0].0, roles[0].1),
                false,
                false,
            )?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, ")", false, true)?;
    }
    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    let publisher = first_source_property(source, BibliographySourceField::Publisher);
    if !location.is_empty() || !publisher.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_source_list(&mut paragraph, grammar, &location)?;
        if !location.is_empty() && !publisher.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, ": ", false, false)?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, publisher, false, false)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_performance_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let title = first_source_property(source, BibliographySourceField::Title);
    let mut paragraph =
        apa_semantic_creator_paragraph(source, BibliographyContributorRole::Writer, grammar, true)?;
    if !title.is_empty() && !grammar.curly_titles {
        append_apa_semantic_run(&mut paragraph, grammar, ".", true, true)?;
    }
    let roles = [
        (BibliographyContributorRole::Director, grammar.director),
        (BibliographyContributorRole::Performer, grammar.performer),
    ]
    .into_iter()
    .map(|(role, label)| (apa_bibliography_names(source, role, false, grammar), label))
    .filter(|(name, _)| !name.is_empty())
    .collect::<Vec<_>>();
    if grammar.rtl_labels {
        append_apa_semantic_labeled_roles(&mut paragraph, grammar, &roles)?;
    } else if !roles.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
        for (index, (name, label)) in roles.iter().enumerate() {
            if index != 0 {
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    &format!(
                        "{}{}{}",
                        grammar.contributor_separator,
                        grammar.conjunction,
                        if grammar.conjunction.is_empty() {
                            ""
                        } else {
                            " "
                        }
                    ),
                    false,
                    true,
                )?;
            }
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{name}, {label}"),
                false,
                false,
            )?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, ")", false, true)?;
    }
    let location = [
        BibliographySourceField::Theater,
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    if !location.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_source_list(&mut paragraph, grammar, &location)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_interview_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let title = first_source_property(source, BibliographySourceField::Title);
    let mut paragraph = apa_semantic_creator_paragraph(
        source,
        BibliographyContributorRole::Interviewee,
        grammar,
        false,
    )?;
    if !title.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    let broadcast = first_source_property(source, BibliographySourceField::BroadcastTitle);
    if !broadcast.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_title(
            &mut paragraph,
            grammar,
            broadcast,
            true,
            grammar.curly_titles,
            true,
        )?;
        if !grammar.curly_titles {
            append_apa_semantic_run(&mut paragraph, grammar, ".", true, true)?;
        }
    }
    let interviewer = apa_bibliography_names(
        source,
        BibliographyContributorRole::Interviewer,
        false,
        grammar,
    );
    if grammar.rtl_labels {
        let roles = if interviewer.is_empty() {
            Vec::new()
        } else {
            vec![(interviewer.clone(), grammar.interviewer)]
        };
        append_apa_semantic_labeled_roles(&mut paragraph, grammar, &roles)?;
    } else if !interviewer.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!("{interviewer}, {}", grammar.interviewer),
            false,
            false,
        )?;
        append_apa_semantic_run(&mut paragraph, grammar, ")", false, true)?;
    }
    let broadcaster = first_source_property(source, BibliographySourceField::Broadcaster);
    if !broadcaster.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_run(&mut paragraph, grammar, broadcaster, false, false)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    let location = [
        BibliographySourceField::Station,
        BibliographySourceField::City,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    if !location.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_source_list(&mut paragraph, grammar, &location)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_legal_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph = if source.kind == BibliographySourceKind::Patent {
        apa_semantic_creator_paragraph(
            source,
            BibliographyContributorRole::Inventor,
            grammar,
            false,
        )?
    } else {
        let mut paragraph = rdocx_oxml::text::CT_P::new();
        let mut properties = bibliography_paragraph_properties(BibliographyStyle::ApaSixthEdition);
        properties.bidi = grammar.semantic_rtl.then_some(true);
        if let Some(mark) = properties.rpr.as_mut() {
            mark.language = grammar.language.map(str::to_owned);
            mark.language_bidi = grammar.language_bidi.map(str::to_owned);
            mark.language_east_asia = grammar.language_east_asia.map(str::to_owned);
        }
        paragraph.properties = Some(properties);
        paragraph
    };
    if source.kind == BibliographySourceKind::Patent {
        let country = first_source_property(source, BibliographySourceField::CountryRegion);
        let number = first_source_property(source, BibliographySourceField::PatentNumber);
        if !country.is_empty() || !number.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, " ", true, true)?;
            append_apa_semantic_run(&mut paragraph, grammar, country, true, false)?;
            if !country.is_empty() && !number.is_empty() {
                append_apa_semantic_run(&mut paragraph, grammar, " ", true, true)?;
            }
            if grammar.rtl_labels {
                append_apa_semantic_run(&mut paragraph, grammar, grammar.patent, true, true)?;
                append_apa_semantic_run(&mut paragraph, grammar, number, true, false)?;
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    grammar.patent_suffix,
                    true,
                    true,
                )?;
            } else {
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    &format!("{}{number}{}", grammar.patent, grammar.patent_suffix),
                    true,
                    false,
                )?;
            }
            append_apa_semantic_run(&mut paragraph, grammar, ".", true, true)?;
        }
    } else {
        let title = first_source_property(source, BibliographySourceField::Title);
        let number = first_source_property(source, BibliographySourceField::CaseNumber);
        append_apa_semantic_run(&mut paragraph, grammar, title, false, false)?;
        if !number.is_empty() {
            if !title.is_empty() {
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    grammar.contributor_separator,
                    false,
                    grammar.rtl_labels
                        && !matches!(
                            grammar.language_bidi,
                            Some(
                                "ug-CN"
                                    | "ur-PK"
                                    | "ur-IN"
                                    | "he-IL"
                                    | "fa-IR"
                                    | "sd-Arab-PK"
                                    | "pa-Arab-PK"
                                    | "ps-AF"
                            )
                        ),
                )?;
            }
            append_apa_semantic_run(&mut paragraph, grammar, number, false, false)?;
        }
        let court = first_source_property(source, BibliographySourceField::Court);
        let month = first_source_property(source, BibliographySourceField::Month);
        let day = first_source_property(source, BibliographySourceField::Day);
        let year = first_source_property(source, BibliographySourceField::Year);
        if !court.is_empty() || !year.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
            append_apa_semantic_run(&mut paragraph, grammar, court, false, false)?;
            if (grammar.day_first || grammar.calendar_year_first)
                && !year.is_empty()
                && !month.is_empty()
            {
                let date = apa_bibliography_date(source, grammar);
                append_apa_semantic_run(
                    &mut paragraph,
                    grammar,
                    &format!("{}{date}", if court.is_empty() { "" } else { " " }),
                    false,
                    true,
                )?;
            } else {
                if !month.is_empty() {
                    append_apa_semantic_run(
                        &mut paragraph,
                        grammar,
                        &format!("{}{month}", if court.is_empty() { "" } else { " " }),
                        false,
                        true,
                    )?;
                    if !day.is_empty() {
                        append_apa_semantic_run(
                            &mut paragraph,
                            grammar,
                            &format!(" {day}"),
                            false,
                            true,
                        )?;
                    }
                }
                if !year.is_empty() {
                    append_apa_semantic_run(
                        &mut paragraph,
                        grammar,
                        &format!(
                            "{}{year}",
                            if month.is_empty() {
                                " "
                            } else {
                                grammar.calendar_day_year
                            }
                        ),
                        false,
                        true,
                    )?;
                }
            }
            append_apa_semantic_run(&mut paragraph, grammar, ")", false, true)?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_art_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph = apa_semantic_creator_paragraph(
        source,
        BibliographyContributorRole::Artist,
        grammar,
        first_source_property(source, BibliographySourceField::PublicationTitle).is_empty(),
    )?;
    let title = first_source_property(source, BibliographySourceField::Title);
    let publication = first_source_property(source, BibliographySourceField::PublicationTitle);
    if !title.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, ".", publication.is_empty(), true)?;
    }
    if !publication.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_title(
            &mut paragraph,
            grammar,
            publication,
            true,
            grammar.curly_titles,
            true,
        )?;
        if !grammar.curly_titles {
            append_apa_semantic_run(&mut paragraph, grammar, ".", true, true)?;
        }
    }
    let location = [
        BibliographySourceField::Institution,
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    if !location.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_source_list(&mut paragraph, grammar, &location)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_periodical_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph = apa_semantic_creator_paragraph(
        source,
        BibliographyContributorRole::Author,
        grammar,
        false,
    )?;
    let title = first_source_property(source, BibliographySourceField::Title);
    if !title.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_secondary(&mut paragraph, source, grammar)?;
    let periodical = first_source_property(
        source,
        if source.kind == BibliographySourceKind::JournalArticle {
            BibliographySourceField::JournalName
        } else {
            BibliographySourceField::PeriodicalTitle
        },
    );
    let volume = first_source_property(source, BibliographySourceField::Volume);
    let issue = first_source_property(source, BibliographySourceField::Issue);
    let pages = first_source_property(source, BibliographySourceField::Pages);
    if !periodical.is_empty() || !volume.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_title(
            &mut paragraph,
            grammar,
            periodical,
            true,
            grammar.curly_titles,
            false,
        )?;
        if !volume.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!(
                    "{}{volume}",
                    if periodical.is_empty() {
                        ""
                    } else {
                        grammar.contributor_separator
                    }
                ),
                true,
                grammar.language_bidi != Some("ug-CN"),
            )?;
        }
    }
    if !issue.is_empty() {
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!("({issue})"),
            false,
            grammar.language_bidi != Some("ug-CN"),
        )?;
    }
    if !pages.is_empty() {
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            grammar.contributor_separator,
            false,
            grammar.language_bidi != Some("ug-CN"),
        )?;
        if source.kind == BibliographySourceKind::ArticleInAPeriodical {
            let label = grammar.pages.trim_end();
            let (label, terminal) = label
                .strip_suffix('.')
                .map_or((label, ""), |text| (text, "."));
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                label,
                false,
                grammar.rtl_labels && grammar.language_bidi != Some("ug-CN"),
            )?;
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!("{terminal} "),
                false,
                true,
            )?;
        }
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            pages,
            false,
            grammar.language_bidi != Some("ug-CN")
                || source.kind == BibliographySourceKind::ArticleInAPeriodical,
        )?;
    }
    if !periodical.is_empty() || !volume.is_empty() || !issue.is_empty() || !pages.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_rtl_authored_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> Result<rdocx_oxml::text::CT_P> {
    let mut paragraph =
        apa_semantic_creator_paragraph(source, BibliographyContributorRole::Author, grammar, true)?;
    let title = first_source_property(source, BibliographySourceField::Title);
    let edition = first_source_property(source, BibliographySourceField::Edition);
    let volume = first_source_property(source, BibliographySourceField::Volume);
    if source.kind == BibliographySourceKind::Book
        && (grammar.rtl_labels || grammar.literal_font.is_some() || grammar.edition_before)
        && (!edition.is_empty() || !volume.is_empty())
    {
        let mut details = Vec::new();
        if !edition.is_empty() {
            details.push(apa_bibliography_edition(edition, grammar));
        }
        if !volume.is_empty() {
            details.push(apa_bibliography_volume(volume, grammar));
        }
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!(" ({}).", details.join(grammar.contributor_separator)),
            false,
            true,
        )?;
    } else if source.kind == BibliographySourceKind::Book && !edition.is_empty() {
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &format!(" ({edition} "),
            false,
            true,
        )?;
        append_apa_semantic_run(&mut paragraph, grammar, grammar.edition, false, false)?;
        if !volume.is_empty() {
            append_apa_semantic_run(
                &mut paragraph,
                grammar,
                &format!(", {}", apa_bibliography_volume(volume, grammar)),
                false,
                false,
            )?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, ").", false, true)?;
    } else if source.kind == BibliographySourceKind::Book && !volume.is_empty() {
        append_apa_semantic_run(&mut paragraph, grammar, " (", false, true)?;
        append_apa_semantic_run(
            &mut paragraph,
            grammar,
            &apa_bibliography_volume(volume, grammar),
            false,
            false,
        )?;
        append_apa_semantic_run(&mut paragraph, grammar, ").", false, true)?;
    } else if !title.is_empty()
        && !(grammar.curly_titles
            && matches!(
                source.kind,
                BibliographySourceKind::Report | BibliographySourceKind::DocumentFromInternetSite
            ))
    {
        append_apa_semantic_run(&mut paragraph, grammar, ".", true, true)?;
    }
    if source.kind != BibliographySourceKind::Report {
        append_apa_semantic_secondary(&mut paragraph, source, grammar)?;
    }
    if source.kind == BibliographySourceKind::Report {
        let institution = [
            BibliographySourceField::Institution,
            BibliographySourceField::Department,
        ]
        .into_iter()
        .map(|field| first_source_property(source, field))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
        if !institution.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
            append_apa_semantic_source_list(&mut paragraph, grammar, &institution)?;
            append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
        }
    }

    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .filter(|field| {
        source.kind != BibliographySourceKind::Report || *field == BibliographySourceField::City
    })
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    let publisher = first_source_property(source, BibliographySourceField::Publisher);
    if source.kind != BibliographySourceKind::DocumentFromInternetSite
        && (!location.is_empty() || !publisher.is_empty())
    {
        append_apa_semantic_run(&mut paragraph, grammar, " ", false, true)?;
        append_apa_semantic_source_list(&mut paragraph, grammar, &location)?;
        if !location.is_empty() && !publisher.is_empty() {
            append_apa_semantic_run(&mut paragraph, grammar, ": ", false, false)?;
        }
        append_apa_semantic_run(&mut paragraph, grammar, publisher, false, false)?;
        append_apa_semantic_run(&mut paragraph, grammar, ".", false, true)?;
    }
    append_apa_semantic_retrieval(&mut paragraph, source, grammar)?;
    Ok(paragraph)
}

fn apa_authored_bibliography_paragraph(
    source: &BibliographySource,
    grammar: &ApaBibliographyLanguage,
) -> rdocx_oxml::text::CT_P {
    use rdocx_oxml::text::CT_P;
    let mut author_label =
        apa_bibliography_names(source, BibliographyContributorRole::Author, true, grammar);
    let editor_count = source
        .contributors
        .iter()
        .filter(|value| value.role == BibliographyContributorRole::Editor)
        .map(|value| match &value.value {
            BibliographyAuthor::People(people) => people.len(),
            BibliographyAuthor::Corporate(_) => 1,
        })
        .sum::<usize>();
    let editor_label = if editor_count > 1 {
        grammar.editors
    } else {
        grammar.editor
    };
    let editor_fallback = source.kind != BibliographySourceKind::Report
        && author_label.is_empty()
        && !apa_bibliography_names(source, BibliographyContributorRole::Editor, true, grammar)
            .is_empty();
    if editor_fallback {
        author_label = format!(
            "{} ({editor_label}).",
            apa_bibliography_names(source, BibliographyContributorRole::Editor, true, grammar)
        );
    }
    let title = first_source_property(source, BibliographySourceField::Title);
    let date = apa_bibliography_date(source, grammar);
    let year = date.as_str();

    let edition = first_source_property(source, BibliographySourceField::Edition);
    let volume = first_source_property(source, BibliographySourceField::Volume);
    let mut parenthetical = Vec::new();
    if source.kind == BibliographySourceKind::Book && !edition.is_empty() {
        parenthetical.push(apa_bibliography_edition(edition, grammar));
    }
    if source.kind == BibliographySourceKind::Book && !volume.is_empty() {
        parenthetical.push(apa_bibliography_volume(volume, grammar));
    }
    let title_display = if parenthetical.is_empty() {
        format!("{title}.")
    } else {
        title.to_owned()
    };
    let mut paragraph = CT_P::new();
    paragraph.properties = Some(bibliography_paragraph_properties(
        BibliographyStyle::ApaSixthEdition,
    ));
    let mut publication = if parenthetical.is_empty() {
        String::new()
    } else {
        format!(" ({}).", parenthetical.join(", "))
    };
    if author_label.is_empty() && source.kind == BibliographySourceKind::Report {
        paragraph.runs.extend(bibliography_display_runs(
            &format!("({year}).{}", if title.is_empty() { "" } else { " " }),
            false,
        ));
        if !title.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(&title_display, true));
        }
    } else if author_label.is_empty() {
        if !title.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(&title_display, true));
            publication.push_str(&format!(" ({year})."));
        } else {
            publication.insert_str(0, &format!("({year})."));
        }
    } else {
        paragraph.runs.extend(bibliography_display_runs(
            &format!(
                "{author_label} ({year}).{}",
                if title.is_empty() { "" } else { " " }
            ),
            false,
        ));
        if !title.is_empty() {
            paragraph
                .runs
                .extend(bibliography_display_runs(&title_display, true));
        }
    }
    let location = [
        BibliographySourceField::City,
        BibliographySourceField::StateProvince,
        BibliographySourceField::CountryRegion,
    ]
    .into_iter()
    .filter(|field| {
        source.kind != BibliographySourceKind::Report || *field == BibliographySourceField::City
    })
    .map(|field| first_source_property(source, field))
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    let publisher = first_source_property(source, BibliographySourceField::Publisher);

    if source.kind != BibliographySourceKind::Report {
        publication.push_str(&apa_bibliography_secondary(
            source,
            editor_fallback,
            grammar,
        ));
    }
    if source.kind == BibliographySourceKind::Report {
        let institution = [
            BibliographySourceField::Institution,
            BibliographySourceField::Department,
        ]
        .into_iter()
        .map(|field| first_source_property(source, field))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        if !institution.is_empty() {
            publication.push_str(&format!(" {institution}."));
        } else if publisher.is_empty() {
            let thesis = first_source_property(source, BibliographySourceField::ThesisType);
            if !thesis.is_empty() {
                publication.push_str(&format!(" {thesis}"));
                if !location.is_empty() {
                    publication.push_str(&format!(", {location}"));
                }
                publication.push('.');
            }
        }
    }
    let report_thesis = source.kind == BibliographySourceKind::Report
        && publisher.is_empty()
        && !first_source_property(source, BibliographySourceField::ThesisType).is_empty()
        && first_source_property(source, BibliographySourceField::Institution).is_empty()
        && first_source_property(source, BibliographySourceField::Department).is_empty();
    if !report_thesis
        && matches!(
            source.kind,
            BibliographySourceKind::Book | BibliographySourceKind::Report
        )
        && (!location.is_empty()
            || !publisher.is_empty()
            || (!title.is_empty()
                && (!author_label.is_empty() || source.kind == BibliographySourceKind::Report)))
    {
        publication.push(' ');
        publication.push_str(&location);
        if !location.is_empty() && !publisher.is_empty() {
            publication.push_str(": ");
        }
        publication.push_str(publisher);
        if !location.is_empty() || !publisher.is_empty() {
            publication.push('.');
        }
    }
    let retrieval = apa_bibliography_retrieval(source, grammar);
    if source.kind == BibliographySourceKind::DocumentFromInternetSite
        && !author_label.is_empty()
        && !title.is_empty()
        && publication.is_empty()
        && retrieval.is_empty()
    {
        publication.push(' ');
    }
    publication.push_str(&retrieval);
    if !publication.is_empty() {
        paragraph
            .runs
            .extend(bibliography_display_runs(&publication, false));
    }
    paragraph
}

fn ensure_bibliography_style(document: &mut Document) -> Result<()> {
    let style_relationship =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
    let style_content_type =
        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
    let edges = document
        .package
        .get_part_rels(&document.doc_part_name)
        .into_iter()
        .flat_map(|relationships| &relationships.items)
        .filter(|edge| edge.rel_type == style_relationship)
        .collect::<Vec<_>>();
    if edges.len() > 1 {
        return Err(bibliography_error("ambiguous bibliography style part"));
    }
    let existing = edges
        .first()
        .map(|edge| internal_target(&document.package, &document.doc_part_name, edge))
        .transpose()?;
    let part = document.reserve_document_part_bundle(
        existing.as_deref(),
        "/word/styles.xml",
        style_relationship,
        style_content_type,
    )?;
    let mut xml = document.package.get_part(&part).map(<[u8]>::to_vec)
        .unwrap_or_else(|| b"<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"></w:styles>".to_vec());
    let parsed = SourceXml::parse(&xml)?;
    let root = &parsed.nodes[0];
    if root.namespace != "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
        || root.local != "styles"
    {
        return Err(bibliography_error(
            "bibliography styles root has wrong namespace",
        ));
    }
    let existing_definitions = root
        .children
        .iter()
        .filter(|&&index| {
            let node = &parsed.nodes[index];
            node.namespace == root.namespace
                && node.local == "style"
                && node.attributes.iter().any(|(namespace, local, value)| {
                    namespace == &root.namespace && local == "styleId" && value == "Bibliography"
                })
        })
        .count();
    if existing_definitions > 1 {
        return Err(bibliography_error("ambiguous Bibliography style identity"));
    }
    if existing_definitions == 1 {
        return Ok(());
    }
    if document.styles.get_by_id("Bibliography").is_some() {
        return Err(bibliography_error(
            "unsupported namespace binding in projected Bibliography style",
        ));
    }
    let style = b"<w:style xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" w:type=\"paragraph\" w:styleId=\"Bibliography\"><w:name w:val=\"Bibliography\"/><w:basedOn w:val=\"Normal\"/><w:next w:val=\"Normal\"/><w:uiPriority w:val=\"37\"/><w:unhideWhenUsed/></w:style>";
    if root.empty {
        let start = &xml[root.full.start..root.full.end];
        let close = start
            .windows(2)
            .rposition(|bytes| bytes == b"/>")
            .ok_or_else(|| bibliography_error("empty bibliography styles root changed"))?;
        let mut replacement = start[..close].to_vec();
        replacement.push(b'>');
        replacement.extend_from_slice(style);
        replacement.extend_from_slice(format!("</{}>", root.name).as_bytes());
        xml.splice(root.full.clone(), replacement);
    } else {
        xml.splice(root.content.end..root.content.end, style.iter().copied());
    }
    SourceXml::parse(&xml)?;
    document.package.set_part(&part, xml);
    Ok(())
}

impl Document {
    /// Atomically update citation and bibliography caches from saved source and field context.
    /// Imported source XML, including LCID spelling and RefOrder occurrences, retains its bytes.
    /// A field with no effective locale requires [`Self::update_bibliography_with_default_locale`].
    pub fn update_bibliography(&mut self) -> Result<BibliographyUpdateReport> {
        self.update_bibliography_transaction(None)
    }

    /// Atomically update caches with the caller's actual application default locale.
    /// This context applies only when source and field selectors do not specify a nonzero locale.
    /// It does not persist a document default or rewrite source metadata.
    pub fn update_bibliography_with_default_locale(
        &mut self,
        locale: u32,
    ) -> Result<BibliographyUpdateReport> {
        self.update_bibliography_transaction(Some(locale))
    }

    fn update_bibliography_transaction(
        &mut self,
        locale: Option<u32>,
    ) -> Result<BibliographyUpdateReport> {
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        let state = BibliographyUpdateState::new(&candidate, locale)?;
        let report = crate::field::update_bibliography_caches(&mut candidate, &state)?;
        if report.updated_citations == 0 && report.rebuilt_bibliographies == 0 {
            return Ok(report);
        }
        if report.rebuilt_bibliographies > 0 {
            ensure_bibliography_style(&mut candidate)?;
        }
        publish_source_collection(self, candidate)?;
        Ok(report)
    }
}
