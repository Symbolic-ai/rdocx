//! OPC digital-signature discovery, verification, and coverage reporting.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use ring::signature::{RSA_PKCS1_2048_8192_SHA256, UnparsedPublicKey};
use sha2::{Digest, Sha256};
use x509_cert::Certificate;
use x509_cert::der::Decode;

use crate::content_types::ContentTypes;
use crate::error::{OpcError, Result};
use crate::package::OpcPackage;
use crate::relationship::{Relationship, Relationships, rel_types};

const DSIG_NS: &str = "http://www.w3.org/2000/09/xmldsig#";
const C14N_EXCLUSIVE: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
const DIGEST_SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
const SIGNATURE_RSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const RELATIONSHIP_TRANSFORM: &str =
    "http://schemas.openxmlformats.org/package/2006/RelationshipTransform";
const OPC_SIGNATURE_NS: &str = "http://schemas.openxmlformats.org/package/2006/digital-signature";
const RELATIONSHIPS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

#[derive(Debug, Clone)]
pub(crate) struct SignatureSource {
    pub(crate) content_types_xml: Vec<u8>,
    pub(crate) content_types: ContentTypes,
}

/// The signer identity embedded in an X.509 certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerCertificateIdentity {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
}

/// One relationship authenticated by an OPC relationship transform.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoveredRelationship {
    /// `/` for package relationships, otherwise the owning part name.
    pub source_part: String,
    pub relationship_id: String,
}

/// A precise reason why a signature is invalid or incomplete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureIssue {
    MissingPart {
        uri: String,
    },
    ChangedReference {
        uri: String,
        expected_digest: String,
        actual_digest: String,
    },
    DuplicateReference {
        uri: String,
    },
    MissingRelationship {
        source_part: String,
        relationship_id: String,
    },
    DuplicateRelationship {
        source_part: String,
        relationship_id: String,
    },
    ExternalRelationship {
        source_part: String,
        relationship_id: String,
        target: String,
    },
    MissingRelationshipTarget {
        source_part: String,
        relationship_id: String,
        target: String,
    },
    UncoveredPart {
        part_name: String,
    },
    UncoveredRelationship {
        source_part: String,
        relationship_id: String,
    },
    SignatureValueMismatch,
    OrphanSignaturePart {
        part_name: String,
    },
}

/// Verification and declared-coverage result for one signature part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureReport {
    pub signature_part: String,
    pub signer: Option<SignerCertificateIdentity>,
    /// True only when `SignedInfo`, its references, and authenticated manifest references verify.
    pub cryptographically_valid: bool,
    /// True only when every non-signature package part and relationship is covered.
    pub coverage_complete: bool,
    pub covered_parts: Vec<String>,
    pub covered_relationships: Vec<CoveredRelationship>,
    pub issues: Vec<SignatureIssue>,
}

#[derive(Debug, Clone)]
struct XmlName {
    qname: String,
    prefix: String,
    local: String,
    namespace: String,
}

#[derive(Debug, Clone)]
struct XmlAttribute {
    name: XmlName,
    value: String,
}

#[derive(Debug, Clone)]
enum XmlNode {
    Element(XmlElement),
    Text(String),
    ProcessingInstruction { target: String, content: String },
}

#[derive(Debug, Clone)]
struct XmlElement {
    name: XmlName,
    attributes: Vec<XmlAttribute>,
    children: Vec<XmlNode>,
    namespaces: BTreeMap<String, String>,
}

#[derive(Debug)]
enum ReferenceTransform {
    ExclusiveCanonicalization,
    Relationships(Vec<String>),
}

#[derive(Debug)]
struct ReferenceSpec {
    uri: String,
    digest: Vec<u8>,
    transforms: Vec<ReferenceTransform>,
}

pub(crate) fn verify_signatures(package: &OpcPackage) -> Result<Vec<SignatureReport>> {
    let mut signature_parts = discover_signature_parts(package)?;
    let discovered: HashSet<String> = signature_parts.iter().cloned().collect();
    let mut orphaned: Vec<String> = package
        .parts
        .keys()
        .filter(|name| is_signature_xml_part(name) && !discovered.contains(*name))
        .cloned()
        .collect();
    orphaned.sort();
    signature_parts.sort();

    let mut reports = Vec::with_capacity(signature_parts.len() + orphaned.len());
    for part_name in signature_parts {
        let xml = package.parts.get(&part_name).ok_or_else(|| {
            OpcError::InvalidSignatureXml(format!("missing signature part {part_name}"))
        })?;
        reports.push(verify_signature_part(package, &part_name, xml)?);
    }
    reports.extend(orphaned.into_iter().map(|part_name| SignatureReport {
        signature_part: part_name.clone(),
        signer: None,
        cryptographically_valid: false,
        coverage_complete: false,
        covered_parts: Vec::new(),
        covered_relationships: Vec::new(),
        issues: vec![SignatureIssue::OrphanSignaturePart { part_name }],
    }));
    Ok(reports)
}

fn discover_signature_parts(package: &OpcPackage) -> Result<Vec<String>> {
    let origins = package
        .package_rels
        .get_all_by_type(rel_types::DIGITAL_SIGNATURE_ORIGIN);
    let mut parts = Vec::new();
    for origin in origins {
        if is_external(origin) {
            return Err(OpcError::InvalidSignatureXml(format!(
                "external signature origin relationship {}",
                origin.id
            )));
        }
        let origin_part = OpcPackage::resolve_rel_target("/", &origin.target);
        if !package.parts.contains_key(&origin_part) {
            return Err(OpcError::InvalidSignatureXml(format!(
                "signature origin target is missing: {origin_part}"
            )));
        }
        let origin_rels = package.part_rels.get(&origin_part).ok_or_else(|| {
            OpcError::InvalidSignatureXml(format!(
                "signature origin has no relationships: {origin_part}"
            ))
        })?;
        for signature in origin_rels.get_all_by_type(rel_types::DIGITAL_SIGNATURE) {
            if is_external(signature) {
                return Err(OpcError::InvalidSignatureXml(format!(
                    "external signature relationship {}",
                    signature.id
                )));
            }
            let part_name = OpcPackage::resolve_rel_target(&origin_part, &signature.target);
            if parts.contains(&part_name) {
                return Err(OpcError::InvalidSignatureXml(format!(
                    "duplicate signature target {part_name}"
                )));
            }
            parts.push(part_name);
        }
    }
    Ok(parts)
}

fn verify_signature_part(
    package: &OpcPackage,
    signature_part: &str,
    xml: &[u8],
) -> Result<SignatureReport> {
    let root = parse_xml(xml)?;
    require_name(&root, DSIG_NS, "Signature")?;
    let signed_info = direct_child(&root, DSIG_NS, "SignedInfo")?;
    require_algorithm(
        direct_child(signed_info, DSIG_NS, "CanonicalizationMethod")?,
        "canonicalization",
        C14N_EXCLUSIVE,
    )?;
    require_algorithm(
        direct_child(signed_info, DSIG_NS, "SignatureMethod")?,
        "signature method",
        SIGNATURE_RSA_SHA256,
    )?;

    let certificate_element = descendant(&root, DSIG_NS, "X509Certificate")
        .ok_or_else(|| invalid_signature("missing X509Certificate"))?;
    let certificate_der = decode_base64(&element_text(certificate_element), "X509Certificate")?;
    let certificate = Certificate::from_der(&certificate_der)
        .map_err(|error| OpcError::InvalidSigningCertificate(error.to_string()))?;
    let subject_public_key_info = certificate.tbs_certificate().subject_public_key_info();
    let key_algorithm = subject_public_key_info.algorithm.oid.to_string();
    if key_algorithm != "1.2.840.113549.1.1.1" {
        return Err(unsupported("key algorithm", &key_algorithm));
    }
    let public_key = subject_public_key_info
        .subject_public_key
        .as_bytes()
        .ok_or_else(|| {
            OpcError::InvalidSigningCertificate("non-octet-aligned public key".into())
        })?;
    let signer = SignerCertificateIdentity {
        subject: certificate.tbs_certificate().subject().to_string(),
        issuer: certificate.tbs_certificate().issuer().to_string(),
        serial_number: certificate.tbs_certificate().serial_number().to_string(),
    };

    let mut issues = Vec::new();
    let mut covered_parts = BTreeSet::new();
    let mut covered_relationships = BTreeSet::new();
    let mut seen_references = HashSet::new();
    let mut all_reference_digests_valid = true;
    let mut authenticated_roots = Vec::new();

    for reference_element in direct_children(signed_info, DSIG_NS, "Reference") {
        let reference = parse_reference(reference_element)?;
        if !seen_references.insert(reference.uri.clone()) {
            issues.push(SignatureIssue::DuplicateReference {
                uri: reference.uri.clone(),
            });
            all_reference_digests_valid = false;
            continue;
        }
        let actual = apply_reference(
            package,
            &root,
            &reference,
            &mut covered_parts,
            &mut covered_relationships,
            &mut issues,
        )?;
        let digest_valid = compare_digest(&reference, actual, &mut issues);
        all_reference_digests_valid &= digest_valid;
        if digest_valid && let Some(target) = same_document_target(&root, &reference.uri)? {
            authenticated_roots.push(target);
        }
    }

    let mut manifests = Vec::new();
    for target in authenticated_roots {
        collect_matching_elements(target, DSIG_NS, "Manifest", &mut manifests);
    }
    let mut processed_manifests = HashSet::new();
    let mut manifest_index = 0;
    while let Some(manifest) = manifests.get(manifest_index).copied() {
        manifest_index += 1;
        if !processed_manifests.insert(manifest as *const XmlElement) {
            continue;
        }
        for reference_element in direct_children(manifest, DSIG_NS, "Reference") {
            let reference = parse_reference(reference_element)?;
            if !seen_references.insert(reference.uri.clone()) {
                issues.push(SignatureIssue::DuplicateReference {
                    uri: reference.uri.clone(),
                });
                all_reference_digests_valid = false;
                continue;
            }
            let actual = apply_reference(
                package,
                &root,
                &reference,
                &mut covered_parts,
                &mut covered_relationships,
                &mut issues,
            )?;
            let digest_valid = compare_digest(&reference, actual, &mut issues);
            all_reference_digests_valid &= digest_valid;
            if digest_valid && let Some(target) = same_document_target(&root, &reference.uri)? {
                collect_matching_elements(target, DSIG_NS, "Manifest", &mut manifests);
            }
        }
    }

    let signed_info_bytes = canonicalize(signed_info);
    let signature_value = decode_base64(
        &element_text(direct_child(&root, DSIG_NS, "SignatureValue")?),
        "SignatureValue",
    )?;
    let signed_info_valid = UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public_key)
        .verify(&signed_info_bytes, &signature_value)
        .is_ok();
    if !signed_info_valid {
        issues.push(SignatureIssue::SignatureValueMismatch);
    }

    add_uncovered_issues(package, &covered_parts, &covered_relationships, &mut issues);
    let coverage_complete = !issues.iter().any(is_coverage_issue);
    let cryptographically_valid = signed_info_valid && all_reference_digests_valid;

    Ok(SignatureReport {
        signature_part: signature_part.to_string(),
        signer: Some(signer),
        cryptographically_valid,
        coverage_complete,
        covered_parts: covered_parts.into_iter().collect(),
        covered_relationships: covered_relationships.into_iter().collect(),
        issues,
    })
}

fn parse_reference(element: &XmlElement) -> Result<ReferenceSpec> {
    let uri = attribute(element, "", "URI")
        .ok_or_else(|| invalid_signature("Reference has no URI"))?
        .to_string();
    let digest_method = direct_child(element, DSIG_NS, "DigestMethod")?;
    require_algorithm(digest_method, "digest method", DIGEST_SHA256)?;
    let digest = decode_base64(
        &element_text(direct_child(element, DSIG_NS, "DigestValue")?),
        "DigestValue",
    )?;
    let mut transforms = Vec::new();
    if let Some(container) = optional_direct_child(element, DSIG_NS, "Transforms")? {
        for transform in direct_children(container, DSIG_NS, "Transform") {
            let algorithm = attribute(transform, "", "Algorithm")
                .ok_or_else(|| invalid_signature("Transform has no Algorithm"))?;
            match algorithm {
                C14N_EXCLUSIVE => transforms.push(ReferenceTransform::ExclusiveCanonicalization),
                RELATIONSHIP_TRANSFORM => {
                    let mut ids = Vec::new();
                    for reference in element_children(transform) {
                        if !has_name(reference, OPC_SIGNATURE_NS, "RelationshipReference") {
                            return Err(unsupported(
                                "relationship transform child",
                                &reference.name.qname,
                            ));
                        }
                        let id = attribute(reference, "", "SourceId").ok_or_else(|| {
                            invalid_signature("RelationshipReference has no SourceId")
                        })?;
                        if ids.iter().any(|existing| existing == id) {
                            return Err(invalid_signature(&format!(
                                "duplicate relationship SourceId {id}"
                            )));
                        }
                        ids.push(id.to_string());
                    }
                    transforms.push(ReferenceTransform::Relationships(ids));
                }
                other => return Err(unsupported("transform", other)),
            }
        }
    }
    Ok(ReferenceSpec {
        uri,
        digest,
        transforms,
    })
}

fn apply_reference(
    package: &OpcPackage,
    root: &XmlElement,
    reference: &ReferenceSpec,
    covered_parts: &mut BTreeSet<String>,
    covered_relationships: &mut BTreeSet<CoveredRelationship>,
    issues: &mut Vec<SignatureIssue>,
) -> Result<Option<Vec<u8>>> {
    if let Some(element) = same_document_target(root, &reference.uri)? {
        if !matches!(
            reference.transforms.as_slice(),
            [ReferenceTransform::ExclusiveCanonicalization]
        ) {
            return Err(unsupported("same-document transform chain", &reference.uri));
        }
        return Ok(Some(canonicalize(element)));
    }

    let part_name = reference_part_name(&reference.uri)?;
    if let Some((source_part, relationships)) = relationships_for_uri(package, &part_name) {
        let ids = match reference.transforms.as_slice() {
            [
                ReferenceTransform::Relationships(ids),
                ReferenceTransform::ExclusiveCanonicalization,
            ]
            | [ReferenceTransform::Relationships(ids)] => ids,
            _ => return Err(unsupported("relationship transform chain", &reference.uri)),
        };
        let transformed = relationship_transform(
            package,
            &source_part,
            relationships,
            ids,
            covered_relationships,
            issues,
        );
        return Ok(transformed);
    }

    if !reference.transforms.is_empty() {
        return Err(unsupported("part transform chain", &reference.uri));
    }
    let bytes = if part_name == "/[Content_Types].xml" {
        Some(content_types_bytes(package)?)
    } else {
        package.parts.get(&part_name).cloned()
    };
    match bytes {
        Some(bytes) => {
            covered_parts.insert(part_name);
            Ok(Some(bytes))
        }
        None => {
            issues.push(SignatureIssue::MissingPart {
                uri: reference.uri.clone(),
            });
            Ok(None)
        }
    }
}

fn same_document_target<'a>(root: &'a XmlElement, uri: &str) -> Result<Option<&'a XmlElement>> {
    let Some(id) = uri.strip_prefix('#') else {
        return Ok(None);
    };
    let elements = find_all_by_id(root, id);
    match elements.as_slice() {
        [] => Err(invalid_signature(&format!(
            "missing same-document reference #{id}"
        ))),
        [element] => Ok(Some(*element)),
        _ => Err(invalid_signature(&format!(
            "duplicate same-document Id {id}"
        ))),
    }
}

fn relationship_transform(
    package: &OpcPackage,
    source_part: &str,
    relationships: &Relationships,
    ids: &[String],
    covered: &mut BTreeSet<CoveredRelationship>,
    issues: &mut Vec<SignatureIssue>,
) -> Option<Vec<u8>> {
    let mut selected = Vec::new();
    let mut valid = true;
    for id in ids {
        let matches: Vec<&Relationship> = relationships
            .items
            .iter()
            .filter(|relationship| relationship.id == *id)
            .collect();
        match matches.as_slice() {
            [] => {
                issues.push(SignatureIssue::MissingRelationship {
                    source_part: source_part.to_string(),
                    relationship_id: id.clone(),
                });
                valid = false;
            }
            [relationship] if is_external(relationship) => {
                issues.push(SignatureIssue::ExternalRelationship {
                    source_part: source_part.to_string(),
                    relationship_id: id.clone(),
                    target: relationship.target.clone(),
                });
                valid = false;
            }
            [relationship] => {
                let target = OpcPackage::resolve_rel_target(source_part, &relationship.target);
                if !package.parts.contains_key(&target) {
                    issues.push(SignatureIssue::MissingRelationshipTarget {
                        source_part: source_part.to_string(),
                        relationship_id: id.clone(),
                        target,
                    });
                    valid = false;
                    continue;
                }
                selected.push(*relationship);
                covered.insert(CoveredRelationship {
                    source_part: source_part.to_string(),
                    relationship_id: id.clone(),
                });
            }
            _ => {
                issues.push(SignatureIssue::DuplicateRelationship {
                    source_part: source_part.to_string(),
                    relationship_id: id.clone(),
                });
                valid = false;
            }
        }
    }
    if !valid {
        return None;
    }
    selected.sort_by(|left, right| left.id.cmp(&right.id));
    let mut output = format!("<Relationships xmlns=\"{RELATIONSHIPS_NS}\">");
    for relationship in selected {
        output.push_str("<Relationship Id=\"");
        escape_attribute(&relationship.id, &mut output);
        output.push_str("\" Target=\"");
        escape_attribute(&relationship.target, &mut output);
        output.push_str("\" TargetMode=\"Internal\" Type=\"");
        escape_attribute(&relationship.rel_type, &mut output);
        output.push_str("\"></Relationship>");
    }
    output.push_str("</Relationships>");
    Some(output.into_bytes())
}

fn compare_digest(
    reference: &ReferenceSpec,
    actual_bytes: Option<Vec<u8>>,
    issues: &mut Vec<SignatureIssue>,
) -> bool {
    let Some(actual_bytes) = actual_bytes else {
        return false;
    };
    let actual = Sha256::digest(&actual_bytes).to_vec();
    if actual == reference.digest {
        return true;
    }
    issues.push(SignatureIssue::ChangedReference {
        uri: reference.uri.clone(),
        expected_digest: BASE64.encode(&reference.digest),
        actual_digest: BASE64.encode(actual),
    });
    false
}

fn add_uncovered_issues(
    package: &OpcPackage,
    covered_parts: &BTreeSet<String>,
    covered_relationships: &BTreeSet<CoveredRelationship>,
    issues: &mut Vec<SignatureIssue>,
) {
    let infrastructure = signature_infrastructure_parts(package);
    if !covered_parts.contains("/[Content_Types].xml") {
        issues.push(SignatureIssue::UncoveredPart {
            part_name: "/[Content_Types].xml".to_string(),
        });
    }
    let mut parts: Vec<&String> = package
        .parts
        .keys()
        .filter(|name| !infrastructure.contains(*name))
        .collect();
    parts.sort();
    for part_name in parts {
        if !covered_parts.contains(part_name) {
            issues.push(SignatureIssue::UncoveredPart {
                part_name: part_name.clone(),
            });
        }
    }

    add_uncovered_relationships("/", &package.package_rels, covered_relationships, issues);
    let mut sources: Vec<&String> = package.part_rels.keys().collect();
    sources.sort();
    for source in sources {
        add_uncovered_relationships(
            source,
            &package.part_rels[source],
            covered_relationships,
            issues,
        );
    }
}

fn signature_infrastructure_parts(package: &OpcPackage) -> BTreeSet<String> {
    let mut parts = BTreeSet::new();
    for origin in package
        .package_rels
        .get_all_by_type(rel_types::DIGITAL_SIGNATURE_ORIGIN)
    {
        if is_external(origin) {
            continue;
        }
        let origin_part = OpcPackage::resolve_rel_target("/", &origin.target);
        parts.insert(origin_part.clone());
        if let Some(relationships) = package.part_rels.get(&origin_part) {
            for signature in relationships.get_all_by_type(rel_types::DIGITAL_SIGNATURE) {
                if !is_external(signature) {
                    parts.insert(OpcPackage::resolve_rel_target(
                        &origin_part,
                        &signature.target,
                    ));
                }
            }
        }
    }
    parts
}

fn add_uncovered_relationships(
    source: &str,
    relationships: &Relationships,
    covered: &BTreeSet<CoveredRelationship>,
    issues: &mut Vec<SignatureIssue>,
) {
    let mut items: Vec<&Relationship> = relationships
        .items
        .iter()
        .filter(|relationship| {
            relationship.rel_type != rel_types::DIGITAL_SIGNATURE_ORIGIN
                && relationship.rel_type != rel_types::DIGITAL_SIGNATURE
        })
        .collect();
    items.sort_by(|left, right| left.id.cmp(&right.id));
    for relationship in items {
        let item = CoveredRelationship {
            source_part: source.to_string(),
            relationship_id: relationship.id.clone(),
        };
        if !covered.contains(&item) {
            issues.push(SignatureIssue::UncoveredRelationship {
                source_part: source.to_string(),
                relationship_id: relationship.id.clone(),
            });
        }
    }
}

pub(crate) fn content_types_bytes(package: &OpcPackage) -> Result<Vec<u8>> {
    if let Some(source) = &package.signature_source
        && source.content_types == package.content_types
    {
        return Ok(source.content_types_xml.clone());
    }
    package.content_types.to_xml()
}

fn relationships_for_uri<'a>(
    package: &'a OpcPackage,
    part_name: &str,
) -> Option<(String, &'a Relationships)> {
    if part_name == "/_rels/.rels" {
        return Some(("/".to_string(), &package.package_rels));
    }
    let marker = "/_rels/";
    let marker_index = part_name.rfind(marker)?;
    let filename = part_name
        .get(marker_index + marker.len()..)?
        .strip_suffix(".rels")?;
    let directory = &part_name[..marker_index + 1];
    let source = format!("{directory}{filename}");
    package.part_rels.get(&source).map(|rels| (source, rels))
}

fn reference_part_name(uri: &str) -> Result<String> {
    let path = uri.split('?').next().unwrap_or(uri);
    if path.is_empty() || !path.starts_with('/') {
        return Err(invalid_signature(&format!(
            "non-package reference URI {uri}"
        )));
    }
    percent_decode(path)
}

fn percent_decode(value: &str) -> Result<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).and_then(|byte| hex(*byte));
            let low = bytes.get(index + 2).and_then(|byte| hex(*byte));
            let (Some(high), Some(low)) = (high, low) else {
                return Err(invalid_signature(&format!(
                    "invalid percent escape in {value}"
                )));
            };
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| invalid_signature("reference URI is not UTF-8"))
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_xml(xml: &[u8]) -> Result<XmlElement> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<XmlElement> = Vec::new();
    let mut root = None;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(start)) => {
                let inherited = stack.last().map(|element| &element.namespaces);
                stack.push(parse_element_start(&reader, &start, inherited)?);
            }
            Ok(Event::Empty(start)) => {
                let inherited = stack.last().map(|element| &element.namespaces);
                let element = parse_element_start(&reader, &start, inherited)?;
                attach_element(element, &mut stack, &mut root)?;
            }
            Ok(Event::End(_)) => {
                let element = stack
                    .pop()
                    .ok_or_else(|| invalid_signature("unexpected closing element"))?;
                attach_element(element, &mut stack, &mut root)?;
            }
            Ok(Event::Text(text)) => {
                if let Some(parent) = stack.last_mut() {
                    let decoded = text
                        .xml_content(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|error| invalid_signature(&error.to_string()))?;
                    let unescaped = quick_xml::escape::unescape(&decoded)
                        .map_err(|error| invalid_signature(&error.to_string()))?;
                    parent.children.push(XmlNode::Text(unescaped.into_owned()));
                }
            }
            Ok(Event::CData(text)) => {
                if let Some(parent) = stack.last_mut() {
                    let decoded = text
                        .xml_content(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|error| invalid_signature(&error.to_string()))?;
                    parent.children.push(XmlNode::Text(decoded.into_owned()));
                }
            }
            Ok(Event::PI(instruction)) => {
                if let Some(parent) = stack.last_mut() {
                    let target = reader
                        .decoder()
                        .decode(instruction.target())
                        .map_err(|error| invalid_signature(&error.to_string()))?;
                    let content = reader
                        .decoder()
                        .decode(instruction.content())
                        .map_err(|error| invalid_signature(&error.to_string()))?;
                    parent.children.push(XmlNode::ProcessingInstruction {
                        target: normalize_xml_line_endings(&target),
                        content: normalize_xml_line_endings(&content),
                    });
                }
            }
            Ok(Event::Eof) => break,
            Ok(Event::Decl(_) | Event::Comment(_) | Event::DocType(_)) => {}
            Ok(Event::GeneralRef(reference)) => {
                return Err(invalid_signature(&format!(
                    "unresolved general entity {}",
                    String::from_utf8_lossy(reference.as_ref())
                )));
            }
            Err(error) => return Err(error.into()),
        }
        buffer.clear();
    }
    if !stack.is_empty() {
        return Err(invalid_signature("unclosed XML element"));
    }
    root.ok_or_else(|| invalid_signature("signature XML has no root element"))
}

fn parse_element_start(
    reader: &Reader<&[u8]>,
    start: &BytesStart<'_>,
    inherited: Option<&BTreeMap<String, String>>,
) -> Result<XmlElement> {
    let mut namespaces = inherited.cloned().unwrap_or_default();
    namespaces.insert("xml".to_string(), XML_NS.to_string());
    let mut decoded_attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let key = std::str::from_utf8(attribute.key.as_ref())?.to_string();
        let value = attribute
            .decoded_and_normalized_value(quick_xml::XmlVersion::Implicit1_0, reader.decoder())?
            .into_owned();
        if key == "xmlns" {
            namespaces.insert(String::new(), value);
        } else if let Some(prefix) = key.strip_prefix("xmlns:") {
            namespaces.insert(prefix.to_string(), value);
        } else {
            decoded_attributes.push((key, value));
        }
    }
    let qname = std::str::from_utf8(start.name().as_ref())?.to_string();
    let name = resolve_name(&qname, &namespaces, true)?;
    let attributes = decoded_attributes
        .into_iter()
        .map(|(qname, value)| {
            Ok(XmlAttribute {
                name: resolve_name(&qname, &namespaces, false)?,
                value,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(XmlElement {
        name,
        attributes,
        children: Vec::new(),
        namespaces,
    })
}

fn resolve_name(
    qname: &str,
    namespaces: &BTreeMap<String, String>,
    use_default: bool,
) -> Result<XmlName> {
    let (prefix, local) = qname
        .split_once(':')
        .map_or(("", qname), |(prefix, local)| (prefix, local));
    let namespace = if prefix.is_empty() && !use_default {
        String::new()
    } else {
        namespaces.get(prefix).cloned().unwrap_or_default()
    };
    if !prefix.is_empty() && namespace.is_empty() {
        return Err(invalid_signature(&format!(
            "unbound namespace prefix {prefix}"
        )));
    }
    Ok(XmlName {
        qname: qname.to_string(),
        prefix: prefix.to_string(),
        local: local.to_string(),
        namespace,
    })
}

fn attach_element(
    element: XmlElement,
    stack: &mut [XmlElement],
    root: &mut Option<XmlElement>,
) -> Result<()> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(XmlNode::Element(element));
    } else if root.replace(element).is_some() {
        return Err(invalid_signature("signature XML has multiple roots"));
    }
    Ok(())
}

fn canonicalize(element: &XmlElement) -> Vec<u8> {
    let mut output = String::new();
    write_canonical(element, &BTreeMap::new(), &mut output);
    output.into_bytes()
}

fn write_canonical(
    element: &XmlElement,
    rendered_namespaces: &BTreeMap<String, String>,
    output: &mut String,
) {
    output.push('<');
    output.push_str(&element.name.qname);

    let mut visible = BTreeSet::new();
    visible.insert(element.name.prefix.clone());
    for attribute in &element.attributes {
        if !attribute.name.prefix.is_empty() && attribute.name.prefix != "xml" {
            visible.insert(attribute.name.prefix.clone());
        }
    }
    let mut child_namespaces = rendered_namespaces.clone();
    for prefix in visible {
        let uri = if prefix.is_empty() {
            element.name.namespace.as_str()
        } else {
            element
                .namespaces
                .get(&prefix)
                .map(String::as_str)
                .unwrap_or("")
        };
        if rendered_namespaces.get(&prefix).map(String::as_str) != Some(uri) {
            if prefix.is_empty() {
                output.push_str(" xmlns=\"");
            } else {
                output.push_str(" xmlns:");
                output.push_str(&prefix);
                output.push_str("=\"");
            }
            escape_attribute(uri, output);
            output.push('"');
            child_namespaces.insert(prefix, uri.to_string());
        }
    }

    let mut attributes: Vec<&XmlAttribute> = element.attributes.iter().collect();
    attributes.sort_by(|left, right| {
        (left.name.namespace.as_str(), left.name.local.as_str())
            .cmp(&(right.name.namespace.as_str(), right.name.local.as_str()))
    });
    for attribute in attributes {
        output.push(' ');
        output.push_str(&attribute.name.qname);
        output.push_str("=\"");
        escape_attribute(&attribute.value, output);
        output.push('"');
    }
    output.push('>');
    for child in &element.children {
        match child {
            XmlNode::Element(child) => write_canonical(child, &child_namespaces, output),
            XmlNode::Text(text) => escape_text(text, output),
            XmlNode::ProcessingInstruction { target, content } => {
                output.push_str("<?");
                output.push_str(target);
                escape_processing_instruction(content, output);
                output.push_str("?>");
            }
        }
    }
    output.push_str("</");
    output.push_str(&element.name.qname);
    output.push('>');
}

fn escape_processing_instruction(value: &str, output: &mut String) {
    for character in value.chars() {
        if character == '\r' {
            output.push_str("&#xD;");
        } else {
            output.push(character);
        }
    }
}

fn normalize_xml_line_endings(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\r' {
            if characters.peek() == Some(&'\n') {
                characters.next();
            }
            normalized.push('\n');
        } else {
            normalized.push(character);
        }
    }
    normalized
}

fn escape_attribute(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '"' => output.push_str("&quot;"),
            '\t' => output.push_str("&#x9;"),
            '\n' => output.push_str("&#xA;"),
            '\r' => output.push_str("&#xD;"),
            _ => output.push(character),
        }
    }
}

fn escape_text(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '\r' => output.push_str("&#xD;"),
            _ => output.push(character),
        }
    }
}

fn require_algorithm(element: &XmlElement, kind: &'static str, expected: &str) -> Result<()> {
    let actual = attribute(element, "", "Algorithm")
        .ok_or_else(|| invalid_signature(&format!("{} has no Algorithm", element.name.qname)))?;
    if actual != expected {
        return Err(unsupported(kind, actual));
    }
    Ok(())
}

fn require_name(element: &XmlElement, namespace: &str, local: &str) -> Result<()> {
    if has_name(element, namespace, local) {
        Ok(())
    } else {
        Err(invalid_signature(&format!(
            "expected {{{namespace}}}{local}, found {}",
            element.name.qname
        )))
    }
}

fn direct_child<'a>(
    element: &'a XmlElement,
    namespace: &str,
    local: &str,
) -> Result<&'a XmlElement> {
    optional_direct_child(element, namespace, local)?
        .ok_or_else(|| invalid_signature(&format!("missing {{{namespace}}}{local}")))
}

fn optional_direct_child<'a>(
    element: &'a XmlElement,
    namespace: &str,
    local: &str,
) -> Result<Option<&'a XmlElement>> {
    let matches: Vec<&XmlElement> = element_children(element)
        .filter(|child| has_name(child, namespace, local))
        .collect();
    match matches.as_slice() {
        [] => Ok(None),
        [child] => Ok(Some(*child)),
        _ => Err(invalid_signature(&format!(
            "duplicate {{{namespace}}}{local}"
        ))),
    }
}

fn direct_children<'a>(
    element: &'a XmlElement,
    namespace: &'a str,
    local: &'a str,
) -> impl Iterator<Item = &'a XmlElement> {
    element_children(element).filter(move |child| has_name(child, namespace, local))
}

fn element_children(element: &XmlElement) -> impl Iterator<Item = &XmlElement> {
    element.children.iter().filter_map(|child| match child {
        XmlNode::Element(element) => Some(element),
        XmlNode::Text(_) | XmlNode::ProcessingInstruction { .. } => None,
    })
}

fn descendant<'a>(element: &'a XmlElement, namespace: &str, local: &str) -> Option<&'a XmlElement> {
    element_children(element).find_map(|child| {
        has_name(child, namespace, local)
            .then_some(child)
            .or_else(|| descendant(child, namespace, local))
    })
}

fn collect_matching_elements<'a>(
    element: &'a XmlElement,
    namespace: &str,
    local: &str,
    found: &mut Vec<&'a XmlElement>,
) {
    if has_name(element, namespace, local) {
        found.push(element);
    }
    collect_descendants(element, namespace, local, found);
}

fn collect_descendants<'a>(
    element: &'a XmlElement,
    namespace: &str,
    local: &str,
    found: &mut Vec<&'a XmlElement>,
) {
    for child in element_children(element) {
        if has_name(child, namespace, local) {
            found.push(child);
        }
        collect_descendants(child, namespace, local, found);
    }
}

fn find_all_by_id<'a>(element: &'a XmlElement, id: &str) -> Vec<&'a XmlElement> {
    let mut found = Vec::new();
    collect_by_id(element, id, &mut found);
    found
}

fn collect_by_id<'a>(element: &'a XmlElement, id: &str, found: &mut Vec<&'a XmlElement>) {
    if attribute(element, "", "Id") == Some(id) {
        found.push(element);
    }
    for child in element_children(element) {
        collect_by_id(child, id, found);
    }
}

fn has_name(element: &XmlElement, namespace: &str, local: &str) -> bool {
    element.name.namespace == namespace && element.name.local == local
}

fn attribute<'a>(element: &'a XmlElement, namespace: &str, local: &str) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find(|attribute| attribute.name.namespace == namespace && attribute.name.local == local)
        .map(|attribute| attribute.value.as_str())
}

fn element_text(element: &XmlElement) -> String {
    let mut text = String::new();
    collect_text(element, &mut text);
    text.trim().to_string()
}

fn collect_text(element: &XmlElement, output: &mut String) {
    for child in &element.children {
        match child {
            XmlNode::Element(child) => collect_text(child, output),
            XmlNode::Text(text) => output.push_str(text),
            XmlNode::ProcessingInstruction { .. } => {}
        }
    }
}

fn decode_base64(value: &str, field: &str) -> Result<Vec<u8>> {
    let compact: String = value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    BASE64
        .decode(compact)
        .map_err(|error| invalid_signature(&format!("invalid {field}: {error}")))
}

fn is_external(relationship: &Relationship) -> bool {
    relationship
        .target_mode
        .as_deref()
        .is_some_and(|mode| mode.eq_ignore_ascii_case("external"))
}

fn is_signature_part(name: &str) -> bool {
    name.starts_with("/_xmlsignatures/")
}

fn is_signature_xml_part(name: &str) -> bool {
    is_signature_part(name) && name.ends_with(".xml")
}

fn is_coverage_issue(issue: &SignatureIssue) -> bool {
    !matches!(
        issue,
        SignatureIssue::SignatureValueMismatch | SignatureIssue::ChangedReference { .. }
    )
}

fn invalid_signature(message: &str) -> OpcError {
    OpcError::InvalidSignatureXml(message.to_string())
}

fn unsupported(kind: &'static str, algorithm: &str) -> OpcError {
    OpcError::UnsupportedSignatureAlgorithm {
        kind,
        algorithm: algorithm.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    const SIGNATURE_DS_BASE64: &str = "RMIFfzT/GCclCbe0UiSNTXzGrz/cuxmfyniUJLDTgpmJBv7NBSQ0LMJY71qcPE5jRhkdC7AUeWKQrIdFAjHVRqYrBDWEaNdUmiRcbdEw67c3nVF33Zmuj24FULFC58/UMhP7m3Rvl2MfWzPY+9EPoEcCfyDkvzZYaurSxnYf5E2L/3rI7Exv/QBrfVwv7bC+I/ReuoRTc9rToOMQUowsLLgcDC5MFz0onvHGUaM/azClme+AJHPPYky2k5aKWmiN+mcBIUrlZYUnx+tdnzcf1DVdCk1YpNc9xqWndFSIjiDd0gfeDWq++VsJBXme02aVMLLXciVrOlvAYaMVdawUHw==";
    const SIGNATURE_SIG_BASE64: &str = "PoIHdPAtYTKOlVcxhWvUEHrlYf58I53jEJFbeiVb1BrQq3MLln4Bq8uHjY6gPs043hDausEDwuf44Olw/l2twOPmyiDPotpCNZe+yg6QeWPYMcWMM2PedUsMflLaYxOOFCFSAolH6MWR6N7w8iIgUSohay5B7KyRiuOOTu2nvlcxZJhei0EEiQFnPlfLhijZDZ97V0I+JAAPy8DDvFF7NOAUOz15yIN1AVgKtzUdgdsCvqR6wKV1+2XEfDftBt462QHOhkW5lrHcMg/THY0yEHjw1VXN6hwBPshzTORXeJexr0lmcJJt1p1vyyjPuDEiM8/JYpDDF8xP0B44/uE4Cg==";
    const CERTIFICATE_DER_BASE64: &str = "MIIDZzCCAk+gAwIBAgIUHP/5/GIqsdVlsrP3bYWND71j/aswDQYJKoZIhvcNAQELBQAwQzEaMBgGA1UEAwwRRi0xNzEgVGVzdCBTaWduZXIxGDAWBgNVBAoMD1RlbnNvcmJlZSBUZXN0czELMAkGA1UEBhMCR0IwHhcNMjYwODIyMDE1NTAxWhcNMzYwODE5MDE1NTAxWjBDMRowGAYDVQQDDBFGLTE3MSBUZXN0IFNpZ25lcjEYMBYGA1UECgwPVGVuc29yYmVlIFRlc3RzMQswCQYDVQQGEwJHQjCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBALRYSR2vue+MSg8VlQKoYDwwZk+b3rs7F2j6u3e5Wbanf/ABy4378xFq1xB6M5B3T+kabTVeENWx+IE3d2r8Z7KxOxsHhZyhrcS4QMa2alL9ErE2JrrTNFsM+ciI02xgLIr+baSig2iV5uzAgcqe9jMMurpeyCmaZxIAhpy3bjKUgyWVBgmZsznfDLMbJfC4u4fifY5Lpsghb9ewaV4euP/8/TE5orJOC80kWyqgbkws+RzoA1KyyBf6EzNtUzBBhEH+iBkhZtYVXuZcaq5WlkcKvQLzhsxPWSjD5CPU0tohtJkGP3ugDczifJRXH5i55GZjUPB9SGKdV6Gsi3Ow3x0CAwEAAaNTMFEwHQYDVR0OBBYEFG8Kul280npBFhtYiONVazzJqGNTMB8GA1UdIwQYMBaAFG8Kul280npBFhtYiONVazzJqGNTMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQELBQADggEBAFd9eu72wlh9rtleRKvPGOjZN55GtQistxUqBWdNNSBEDIQDIvZ9gM3ix+IkM45zS5rX6ME8eqeAUIiP40izNnrOIk4PT6f68hL/dCrDmcxouU0RHI2OgHHz9Ir2n94Lz+dBWx+8Syafa48GfvfpKKXo4/qQQNk0idZ524ftYT7RpCL11z2QuzcBHnD7ZoubdRYo6i12WPGHo2mZu0BS1odKZajMvBPhdF4rH2TqGUgYyjeeHUb+8cnYz9OPgGB/6hvv+sBCqEHWDnsVYIJzhjDQS6l0v0YVeE7RIwHrxPvEF+dGAqLfDYoZ6mWe50jBtrOq9otI13ncnccEHm8Aovo=";
    const EC_CERTIFICATE_DER_BASE64: &str = "MIIB4DCCAYegAwIBAgIUf1e7Wg30URPA6mcy7mqRrxSGfkUwCgYIKoZIzj0EAwIwRjEdMBsGA1UEAwwURi0xNzEgRUMgVGVzdCBTaWduZXIxGDAWBgNVBAoMD1RlbnNvcmJlZSBUZXN0czELMAkGA1UEBhMCR0IwHhcNMjYwODIyMDIwMTE4WhcNMzYwODE5MDIwMTE4WjBGMR0wGwYDVQQDDBRGLTE3MSBFQyBUZXN0IFNpZ25lcjEYMBYGA1UECgwPVGVuc29yYmVlIFRlc3RzMQswCQYDVQQGEwJHQjBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABDzWARcxjZ++hPgwD2LB/Hz+cJ2fm0Fjb3uE3NNhH0gItnMAQJvS3KQBRUU28gVYra2iDQabYYcCGC5LNX2yXCmjUzBRMB0GA1UdDgQWBBTiKMGSnjWxKe/LNO0uLALvpD8QrTAfBgNVHSMEGDAWgBTiKMGSnjWxKe/LNO0uLALvpD8QrTAPBgNVHRMBAf8EBTADAQH/MAoGCCqGSM49BAMCA0cAMEQCIBd6DFidlSSE5BApuxqqpsXpx5bHWZJrpn89432E5LJoAiB0akoHnuZREqsmnrJ9GXehgUMXYqf7hiVdcB4JMXeE4w==";

    #[test]
    fn signature_parser_is_prefix_tolerant_and_algorithm_strict() {
        let package = signed_package("sig");
        let report = package.verify_signatures().unwrap().remove(0);
        assert!(report.cryptographically_valid);

        for (from, to, kind) in [
            (DIGEST_SHA256, "urn:test:sha1", "digest method"),
            (
                SIGNATURE_RSA_SHA256,
                "urn:test:rsa-sha1",
                "signature method",
            ),
            (C14N_EXCLUSIVE, "urn:test:c14n", "canonicalization"),
        ] {
            let mut changed = signed_package("sig");
            mutate_signature_xml(&mut changed, |xml| xml.replacen(from, to, 1));
            assert!(matches!(
                changed.verify_signatures(),
                Err(OpcError::UnsupportedSignatureAlgorithm { kind: actual, .. }) if actual == kind
            ));
        }

        let unsupported_transform = format!(
            r#"<ds:Reference xmlns:ds="{DSIG_NS}" URI="/word/document.xml"><ds:Transforms><ds:Transform Algorithm="urn:test:relationship"></ds:Transform></ds:Transforms><ds:DigestMethod Algorithm="{DIGEST_SHA256}"></ds:DigestMethod><ds:DigestValue>{}</ds:DigestValue></ds:Reference>"#,
            BASE64.encode([0_u8; 32])
        );
        let reference = parse_xml(unsupported_transform.as_bytes()).unwrap();
        assert!(matches!(
            parse_reference(&reference),
            Err(OpcError::UnsupportedSignatureAlgorithm {
                kind: "transform",
                ..
            })
        ));

        let mut ec = signed_package("sig");
        mutate_signature_xml(&mut ec, |xml| {
            xml.replace(CERTIFICATE_DER_BASE64, EC_CERTIFICATE_DER_BASE64)
        });
        assert!(matches!(
            ec.verify_signatures(),
            Err(OpcError::UnsupportedSignatureAlgorithm {
                kind: "key algorithm",
                ..
            })
        ));
    }

    #[test]
    fn referenced_object_processing_instruction_is_canonical_and_mutation_sensitive() {
        let xml = format!(
            r#"<ds:Signature xmlns:ds="{DSIG_NS}"><ds:Object Id="pi-object"><ds:Value>before</ds:Value><?audit alpha&beta <ok?><ds:Value>after</ds:Value></ds:Object></ds:Signature>"#
        );
        let root = parse_xml(xml.as_bytes()).unwrap();
        let object = same_document_target(&root, "#pi-object").unwrap().unwrap();
        let expected = format!(
            r#"<ds:Object xmlns:ds="{DSIG_NS}" Id="pi-object"><ds:Value>before</ds:Value><?audit alpha&beta <ok?><ds:Value>after</ds:Value></ds:Object>"#
        );
        assert_eq!(canonicalize(object), expected.as_bytes());

        let reference = ReferenceSpec {
            uri: "#pi-object".to_string(),
            digest: Sha256::digest(expected.as_bytes()).to_vec(),
            transforms: vec![ReferenceTransform::ExclusiveCanonicalization],
        };
        let actual = apply_reference(
            &OpcPackage::new(),
            &root,
            &reference,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut Vec::new(),
        )
        .unwrap();
        assert!(compare_digest(&reference, actual, &mut Vec::new()));

        let changed = parse_xml(xml.replace("alpha", "changed").as_bytes()).unwrap();
        let changed_actual = apply_reference(
            &OpcPackage::new(),
            &changed,
            &reference,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut Vec::new(),
        )
        .unwrap();
        let mut issues = Vec::new();
        assert!(!compare_digest(&reference, changed_actual, &mut issues));
        assert!(matches!(
            issues.as_slice(),
            [SignatureIssue::ChangedReference { uri, .. }] if uri == "#pi-object"
        ));

        let mut escaped = String::new();
        escape_processing_instruction(" data\rmore", &mut escaped);
        assert_eq!(escaped, " data&#xD;more");
    }

    #[test]
    fn relationship_transform_selects_exact_ids_in_canonical_order() {
        let mut package = OpcPackage::new();
        package.set_part("/a.xml", Vec::new());
        package.set_part("/b.xml", Vec::new());
        let relationships = Relationships::from_xml(
            br#"<Relationships><Relationship Id="rId2" Type="urn:b" Target="b.xml"/><Relationship Id="rId1" Type="urn:a" Target="a.xml"/></Relationships>"#,
        )
        .unwrap();
        let mut covered = BTreeSet::new();
        let mut issues = Vec::new();
        let transformed = relationship_transform(
            &package,
            "/",
            &relationships,
            &["rId2".to_string(), "rId1".to_string()],
            &mut covered,
            &mut issues,
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(transformed).unwrap(),
            format!(
                "<Relationships xmlns=\"{RELATIONSHIPS_NS}\"><Relationship Id=\"rId1\" Target=\"a.xml\" TargetMode=\"Internal\" Type=\"urn:a\"></Relationship><Relationship Id=\"rId2\" Target=\"b.xml\" TargetMode=\"Internal\" Type=\"urn:b\"></Relationship></Relationships>"
            )
        );
        assert!(issues.is_empty());

        let duplicate = Relationships::from_xml(
            br#"<Relationships><Relationship Id="rId1" Type="urn:a" Target="a.xml"/><Relationship Id="rId1" Type="urn:b" Target="b.xml"/></Relationships>"#,
        )
        .unwrap();
        let mut duplicate_issues = Vec::new();
        assert!(
            relationship_transform(
                &package,
                "/",
                &duplicate,
                &["rId1".to_string()],
                &mut BTreeSet::new(),
                &mut duplicate_issues,
            )
            .is_none()
        );
        assert!(matches!(
            duplicate_issues.as_slice(),
            [SignatureIssue::DuplicateRelationship { .. }]
        ));

        let external = Relationships::from_xml(
            br#"<Relationships><Relationship Id="rId1" Type="urn:a" Target="https://example.invalid" TargetMode="External"/></Relationships>"#,
        )
        .unwrap();
        let mut external_issues = Vec::new();
        assert!(
            relationship_transform(
                &package,
                "/",
                &external,
                &["rId1".to_string()],
                &mut BTreeSet::new(),
                &mut external_issues,
            )
            .is_none()
        );
        assert!(matches!(
            external_issues.as_slice(),
            [SignatureIssue::ExternalRelationship { .. }]
        ));
    }

    #[test]
    fn valid_signature_reports_complete_declared_coverage() {
        let report = signed_package("ds").verify_signatures().unwrap().remove(0);
        assert!(report.cryptographically_valid);
        assert!(report.coverage_complete);
        assert!(report.issues.is_empty());
        assert_eq!(
            report.covered_parts,
            vec!["/[Content_Types].xml", "/word/document.xml"]
        );
        assert_eq!(
            report.covered_relationships,
            vec![CoveredRelationship {
                source_part: "/".to_string(),
                relationship_id: "rId1".to_string(),
            }]
        );
        let signer = report.signer.unwrap();
        assert!(signer.subject.contains("F-171 Test Signer"));

        let mut relocated = signed_package("ds");
        let origin = relocated
            .parts
            .remove("/_xmlsignatures/origin.sigs")
            .unwrap();
        let signature = relocated.parts.remove("/_xmlsignatures/sig1.xml").unwrap();
        let mut origin_rels = relocated
            .part_rels
            .remove("/_xmlsignatures/origin.sigs")
            .unwrap();
        origin_rels.items[0].target = "signature.xml".to_string();
        relocated.set_part("/custom/security/origin.bin", origin);
        relocated.set_part("/custom/security/signature.xml", signature);
        relocated
            .part_rels
            .insert("/custom/security/origin.bin".to_string(), origin_rels);
        relocated
            .package_rels
            .items
            .iter_mut()
            .find(|relationship| relationship.rel_type == rel_types::DIGITAL_SIGNATURE_ORIGIN)
            .unwrap()
            .target = "custom/security/origin.bin".to_string();
        let relocated_report = relocated.verify_signatures().unwrap().remove(0);
        assert_eq!(
            relocated_report.signature_part,
            "/custom/security/signature.xml"
        );
        assert!(relocated_report.cryptographically_valid);
        assert!(relocated_report.coverage_complete);
    }

    #[test]
    fn modified_signed_part_is_named() {
        let mut package = signed_package("ds");
        package.set_part("/word/document.xml", b"changed".to_vec());
        let report = package.verify_signatures().unwrap().remove(0);
        assert!(!report.cryptographically_valid);
        assert!(report.coverage_complete);
        assert!(matches!(
            report.issues.as_slice(),
            [SignatureIssue::ChangedReference { uri, .. }] if uri == "/word/document.xml"
        ));
    }

    #[test]
    fn partial_or_malformed_coverage_never_reports_success() {
        let mut uncovered = signed_package("ds");
        uncovered.set_part("/word/uncovered.xml", Vec::new());
        let report = uncovered.verify_signatures().unwrap().remove(0);
        assert!(report.cryptographically_valid);
        assert!(!report.coverage_complete);
        assert!(report.issues.contains(&SignatureIssue::UncoveredPart {
            part_name: "/word/uncovered.xml".to_string(),
        }));

        let mut missing = signed_package("ds");
        missing.parts.remove("/word/document.xml");
        let report = missing.verify_signatures().unwrap().remove(0);
        assert!(!report.cryptographically_valid);
        assert!(report.issues.iter().any(|issue| matches!(
            issue,
            SignatureIssue::MissingPart { uri } if uri == "/word/document.xml"
        )));

        let mut duplicate = signed_package("ds");
        mutate_signature_xml(&mut duplicate, |xml| {
            let reference_start = xml
                .find("<ds:Reference URI=\"/word/document.xml\"")
                .unwrap();
            let relative_end = xml[reference_start..].find("</ds:Reference>").unwrap();
            let reference_end = reference_start + relative_end + "</ds:Reference>".len();
            let reference = xml[reference_start..reference_end].to_string();
            xml.replacen("</ds:Manifest>", &format!("{reference}</ds:Manifest>"), 1)
        });
        let report = duplicate.verify_signatures().unwrap().remove(0);
        assert!(!report.cryptographically_valid);
        assert!(report.issues.iter().any(|issue| matches!(
            issue,
            SignatureIssue::ChangedReference { uri, .. } if uri == "#idPackageObject"
        )));

        let mut duplicate_id = signed_package("ds");
        mutate_signature_xml(&mut duplicate_id, |xml| {
            xml.replacen(
                "</ds:Signature>",
                "<ds:Object Id=\"idPackageObject\"></ds:Object></ds:Signature>",
                1,
            )
        });
        assert!(matches!(
            duplicate_id.verify_signatures(),
            Err(OpcError::InvalidSignatureXml(message))
                if message.contains("duplicate same-document Id")
        ));

        let mut duplicate_signed_info = signed_package("ds");
        mutate_signature_xml(&mut duplicate_signed_info, |xml| {
            let start = xml.find("<ds:SignedInfo>").unwrap();
            let relative_end = xml[start..].find("</ds:SignedInfo>").unwrap();
            let end = start + relative_end + "</ds:SignedInfo>".len();
            let signed_info = xml[start..end].to_string();
            xml.replacen(&signed_info, &format!("{signed_info}{signed_info}"), 1)
        });
        assert!(matches!(
            duplicate_signed_info.verify_signatures(),
            Err(OpcError::InvalidSignatureXml(message)) if message.contains("duplicate")
        ));

        let mut infrastructure_relationship = signed_package("ds");
        let expected_relationship_id = infrastructure_relationship
            .part_rels
            .get_mut("/_xmlsignatures/origin.sigs")
            .unwrap()
            .add_external("urn:test:unsigned", "https://example.invalid");
        let report = infrastructure_relationship
            .verify_signatures()
            .unwrap()
            .remove(0);
        assert!(report.cryptographically_valid);
        assert!(!report.coverage_complete);
        assert!(report.issues.iter().any(|issue| matches!(
            issue,
            SignatureIssue::UncoveredRelationship {
                source_part,
                relationship_id,
            } if source_part == "/_xmlsignatures/origin.sigs"
                && relationship_id == &expected_relationship_id
        )));

        let mut external = signed_package("ds");
        external.package_rels.items[0].target_mode = Some("External".to_string());
        let report = external.verify_signatures().unwrap().remove(0);
        assert!(!report.cryptographically_valid);
        assert!(report.issues.iter().any(|issue| matches!(
            issue,
            SignatureIssue::ExternalRelationship { relationship_id, .. } if relationship_id == "rId1"
        )));

        let mut absent_target = signed_package("ds");
        absent_target.package_rels.items[0].target = "word/missing.xml".to_string();
        let report = absent_target.verify_signatures().unwrap().remove(0);
        assert!(!report.cryptographically_valid);
        assert!(report.issues.iter().any(|issue| matches!(
            issue,
            SignatureIssue::MissingRelationshipTarget { target, .. } if target == "/word/missing.xml"
        )));

        let mut orphan = signed_package("ds");
        orphan
            .package_rels
            .items
            .retain(|relationship| relationship.rel_type != rel_types::DIGITAL_SIGNATURE_ORIGIN);
        let report = orphan.verify_signatures().unwrap().remove(0);
        assert!(!report.cryptographically_valid);
        assert!(!report.coverage_complete);
        assert!(matches!(
            report.issues.as_slice(),
            [SignatureIssue::OrphanSignaturePart { .. }]
        ));
    }

    #[test]
    fn unsigned_manifest_cannot_complete_partial_coverage() {
        let mut package = signed_package("ds");
        let unsigned_part = b"unsigned but digest-correct".to_vec();
        package.set_part("/word/unsigned.xml", unsigned_part.clone());
        let unsigned_reference = reference_xml("ds", "/word/unsigned.xml", &unsigned_part, "");
        mutate_signature_xml(&mut package, |xml| {
            xml.replacen(
                "</ds:Signature>",
                &format!(
                    "<ds:Object><ds:Manifest>{unsigned_reference}</ds:Manifest></ds:Object></ds:Signature>"
                ),
                1,
            )
        });

        let report = package.verify_signatures().unwrap().remove(0);
        assert!(report.cryptographically_valid);
        assert!(!report.coverage_complete);
        assert!(
            !report
                .covered_parts
                .contains(&"/word/unsigned.xml".to_string())
        );
        assert!(report.issues.contains(&SignatureIssue::UncoveredPart {
            part_name: "/word/unsigned.xml".to_string(),
        }));
    }

    #[test]
    fn verification_does_not_change_package_bytes() {
        let fixture = signed_package("ds");
        let mut source = Cursor::new(Vec::new());
        fixture.write_to(&mut source).unwrap();
        let source_bytes = source.into_inner();
        if let Ok(path) = std::env::var("RDOCX_F171_ORACLE_OUTPUT") {
            std::fs::write(path, &source_bytes).unwrap();
        }
        let package = OpcPackage::from_reader(Cursor::new(source_bytes)).unwrap();
        let signature_before = package.parts["/_xmlsignatures/sig1.xml"].clone();
        let document_before = package.parts["/word/document.xml"].clone();
        let first = package.verify_signatures().unwrap();
        let second = package.verify_signatures().unwrap();
        assert_eq!(first, second);
        let mut saved = Cursor::new(Vec::new());
        package.write_to(&mut saved).unwrap();
        let reopened = OpcPackage::from_reader(Cursor::new(saved.into_inner())).unwrap();
        assert_eq!(reopened.parts["/_xmlsignatures/sig1.xml"], signature_before);
        assert_eq!(reopened.parts["/word/document.xml"], document_before);
        let report = reopened.verify_signatures().unwrap().remove(0);
        assert!(report.cryptographically_valid);
        assert!(report.coverage_complete);
    }

    fn signed_package(prefix: &str) -> OpcPackage {
        let mut package = OpcPackage::with_main_part(
            "word/document.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        );
        package.set_part(
            "/word/document.xml",
            br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body/></w:document>"#.to_vec(),
        );
        package.set_part("/_xmlsignatures/origin.sigs", Vec::new());
        package.content_types.add_override(
            "/_xmlsignatures/origin.sigs",
            "application/vnd.openxmlformats-package.digital-signature-origin",
        );
        package.content_types.add_override(
            "/_xmlsignatures/sig1.xml",
            "application/vnd.openxmlformats-package.digital-signature-xmlsignature+xml",
        );
        package.package_rels.add(
            rel_types::DIGITAL_SIGNATURE_ORIGIN,
            "_xmlsignatures/origin.sigs",
        );
        package
            .get_or_create_part_rels("/_xmlsignatures/origin.sigs")
            .add(rel_types::DIGITAL_SIGNATURE, "sig1.xml");

        let content_types = package.content_types.to_xml().unwrap();
        let document = package.parts["/word/document.xml"].clone();
        let relationship_bytes = format!(
            "<Relationships xmlns=\"{RELATIONSHIPS_NS}\"><Relationship Id=\"rId1\" Target=\"word/document.xml\" TargetMode=\"Internal\" Type=\"{}\"></Relationship></Relationships>",
            rel_types::DOCUMENT
        );
        let part_references = format!(
            "{}{}{}",
            reference_xml(prefix, "/%5BContent_Types%5D.xml", &content_types, ""),
            reference_xml(prefix, "/word/document.xml", &document, ""),
            reference_xml(
                prefix,
                "/_rels/.rels",
                relationship_bytes.as_bytes(),
                &format!(
                    "<{prefix}:Transforms><{prefix}:Transform Algorithm=\"{RELATIONSHIP_TRANSFORM}\"><mdssi:RelationshipReference SourceId=\"rId1\"></mdssi:RelationshipReference></{prefix}:Transform><{prefix}:Transform Algorithm=\"{C14N_EXCLUSIVE}\"></{prefix}:Transform></{prefix}:Transforms>"
                ),
            ),
        );
        let object = format!(
            "<{prefix}:Object Id=\"idPackageObject\"><{prefix}:Manifest>{part_references}</{prefix}:Manifest></{prefix}:Object>"
        );
        let canonical_object = object
            .replacen(
                &format!("<{prefix}:Object "),
                &format!("<{prefix}:Object xmlns:{prefix}=\"{DSIG_NS}\" "),
                1,
            )
            .replace(
                "<mdssi:RelationshipReference ",
                "<mdssi:RelationshipReference xmlns:mdssi=\"http://schemas.openxmlformats.org/package/2006/digital-signature\" ",
            );
        let object_digest = BASE64.encode(Sha256::digest(canonical_object.as_bytes()));
        let signed_info = format!(
            "<{prefix}:SignedInfo><{prefix}:CanonicalizationMethod Algorithm=\"{C14N_EXCLUSIVE}\"></{prefix}:CanonicalizationMethod><{prefix}:SignatureMethod Algorithm=\"{SIGNATURE_RSA_SHA256}\"></{prefix}:SignatureMethod><{prefix}:Reference URI=\"#idPackageObject\"><{prefix}:Transforms><{prefix}:Transform Algorithm=\"{C14N_EXCLUSIVE}\"></{prefix}:Transform></{prefix}:Transforms><{prefix}:DigestMethod Algorithm=\"{DIGEST_SHA256}\"></{prefix}:DigestMethod><{prefix}:DigestValue>{object_digest}</{prefix}:DigestValue></{prefix}:Reference></{prefix}:SignedInfo>"
        );
        let signature_value = match prefix {
            "ds" => SIGNATURE_DS_BASE64,
            "sig" => SIGNATURE_SIG_BASE64,
            _ => panic!("fixture has no precomputed signature for prefix {prefix}"),
        };
        let xml = format!(
            "<?xml version=\"1.0\"?><{prefix}:Signature xmlns:{prefix}=\"{DSIG_NS}\" xmlns:mdssi=\"http://schemas.openxmlformats.org/package/2006/digital-signature\">{signed_info}<{prefix}:SignatureValue>{signature_value}</{prefix}:SignatureValue><{prefix}:KeyInfo><{prefix}:X509Data><{prefix}:X509Certificate>{CERTIFICATE_DER_BASE64}</{prefix}:X509Certificate></{prefix}:X509Data></{prefix}:KeyInfo>{object}</{prefix}:Signature>"
        );
        package.set_part("/_xmlsignatures/sig1.xml", xml.into_bytes());
        package
    }

    fn reference_xml(prefix: &str, uri: &str, bytes: &[u8], transforms: &str) -> String {
        let digest = BASE64.encode(Sha256::digest(bytes));
        format!(
            "<{prefix}:Reference URI=\"{uri}\">{transforms}<{prefix}:DigestMethod Algorithm=\"{DIGEST_SHA256}\"></{prefix}:DigestMethod><{prefix}:DigestValue>{digest}</{prefix}:DigestValue></{prefix}:Reference>"
        )
    }

    fn mutate_signature_xml(package: &mut OpcPackage, mutate: impl FnOnce(String) -> String) {
        let xml = String::from_utf8(package.parts["/_xmlsignatures/sig1.xml"].clone()).unwrap();
        package.set_part("/_xmlsignatures/sig1.xml", mutate(xml).into_bytes());
    }
}
