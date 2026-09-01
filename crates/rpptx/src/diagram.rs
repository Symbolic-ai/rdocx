//! Private SmartArt validation and transient PresentationML expansion.

use std::collections::{HashMap, HashSet};

use oxml_core::units::{Angle, Emu};
use oxml_drawing::color::ColorChoice;
use oxml_drawing::fill::{Fill, SolidFill};
use oxml_drawing::geometry::CT_CustomGeometry2D;
use oxml_drawing::line::{CT_LineProperties, LineJoin};
use oxml_drawing::text::{CT_TextBody, Coordinate32Value};
use oxml_drawing::xfrm::{CT_Point2D, CT_PositiveSize2D, CT_Transform2D};
use oxml_layout::{Diagnostic, Rect};
use rpptx_oxml::connector::CT_ConnectionShape;
use rpptx_oxml::diagram::{
    CT_DiagramColorsDefinition, CT_DiagramData, CT_DiagramLayoutDefinition,
    CT_DiagramStyleDefinition, DiagramColorRenderLabel, DiagramConnectionKind, DiagramLayoutFamily,
    DiagramPointKind, DiagramRelationshipIds, DiagramRenderInstruction,
    DiagramRenderInstructionKind, DiagramShapeStyle,
};
use rpptx_oxml::graphic_frame::{CT_GraphicFrame, GraphicDataPayload};
use rpptx_oxml::shape_tree::{
    CT_GroupShape, CT_Shape, CT_ShapeTree, ShapeIdAllocator, ShapeTreeChild,
};
use sha2::{Digest, Sha256};

const MAX_NODES: usize = 1_024;
const MAX_CONNECTIONS: usize = 4_096;
const MAX_GRAPH_DEPTH: usize = 64;
const MAX_PROGRAM_WORK: usize = 65_536;
const EMU_PER_POINT: f64 = 12_700.0;
const CYCLE1_COMPATIBILITY_ID: &str = "urn:microsoft.com/office/officeart/2005/8/layout/cycle1";
const CYCLE1_COMPATIBILITY_SHA256: &str =
    "8a7e35b9099cff9fd646490ab9b36f8349d82e2568c92354f871f90315301461";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DiagramResources {
    pub(crate) relationships: DiagramRelationshipIds,
    pub(crate) data: Result<Box<CT_DiagramData>, String>,
    pub(crate) layout: Result<Box<CT_DiagramLayoutDefinition>, String>,
    pub(crate) style: Result<Box<CT_DiagramStyleDefinition>, String>,
    pub(crate) colors: Result<Box<CT_DiagramColorsDefinition>, String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScopedDiagramResources {
    pub(crate) slide: HashMap<String, DiagramResources>,
    pub(crate) layout: HashMap<String, DiagramResources>,
    pub(crate) master: HashMap<String, DiagramResources>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ExpansionResult {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) clips: HashMap<u32, Rect>,
}

#[derive(Clone, Debug)]
struct RenderNode<'a> {
    id: &'a str,
    text: Option<&'a CT_TextBody>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GraphEdge<'a> {
    source: &'a str,
    destination: &'a str,
    source_order: u32,
    destination_order: u32,
}

#[derive(Debug)]
struct PreparedDiagram {
    shapes: Vec<PreparedShape>,
    connectors: Vec<PreparedConnector>,
}

#[derive(Debug, Default)]
struct ProgramEvaluation {
    layout_nodes: usize,
    shapes: usize,
    presentation_mappings: usize,
    algorithms: usize,
    constraints: usize,
    rules: usize,
    conditions: usize,
}

#[derive(Debug, Default)]
struct ProgramBudget {
    used: usize,
}

impl ProgramBudget {
    fn charge(&mut self, kind: &str) -> Result<(), String> {
        self.used = self
            .used
            .checked_add(1)
            .ok_or_else(|| "SmartArt interpreter work counter overflow".to_owned())?;
        if self.used > MAX_PROGRAM_WORK {
            return Err(format!(
                "SmartArt interpreter {kind} exceeds total work bound {MAX_PROGRAM_WORK}"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct PreparedShape {
    rect: Rect,
    preset: &'static str,
    text: Option<CT_TextBody>,
    style_label: &'static str,
    color_index: usize,
    rotation_degrees: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
    adjustments: Vec<(&'static str, f64)>,
    custom_geometry: Option<String>,
}

#[derive(Clone, Debug)]
struct PreparedConnector {
    start: (f64, f64),
    end: (f64, f64),
    preset: &'static str,
    style_label: &'static str,
    custom_geometry: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum DiagramLineSpacing {
    Points(i32),
}

pub(crate) fn expand_tree(
    tree: &mut CT_ShapeTree,
    resources: &HashMap<String, DiagramResources>,
) -> ExpansionResult {
    let mut allocator = ShapeIdAllocator::scan(tree);
    let mut result = ExpansionResult::default();
    expand_children(&mut tree.children, resources, &mut allocator, &mut result);
    result
}

fn expand_children(
    children: &mut [ShapeTreeChild],
    resources: &HashMap<String, DiagramResources>,
    allocator: &mut ShapeIdAllocator,
    result: &mut ExpansionResult,
) {
    for child in children {
        if let ShapeTreeChild::GroupShape(group) = child {
            expand_children(&mut group.children, resources, allocator, result);
            continue;
        }
        if !matches!(child, ShapeTreeChild::GraphicFrame(_)) {
            continue;
        }
        let frame_id = child
            .non_visual_id()
            .unwrap_or_else(|| allocator.allocate());
        let frame_name = child
            .non_visual_name()
            .unwrap_or_else(|| format!("SmartArt {frame_id}"));
        let ShapeTreeChild::GraphicFrame(frame) = child else {
            continue;
        };
        let GraphicDataPayload::SmartArt(relationships) = frame.graphic_data.payload() else {
            continue;
        };
        let identity = resources
            .get(&relationships.data)
            .and_then(|resource| resource.layout.as_ref().ok())
            .and_then(|layout| layout.unique_id.as_deref())
            .unwrap_or("unknown")
            .to_owned();
        match resources
            .get(&relationships.data)
            .ok_or_else(|| {
                format!(
                    "missing SmartArt data relationship `{}`",
                    relationships.data
                )
            })
            .and_then(|resource| {
                validate_relationship_set(resource, relationships).map(|_| resource)
            })
            .and_then(|resource| render_group(frame, resource, frame_id, &frame_name, allocator))
        {
            Ok(group) => {
                if let Ok(bounds) = frame_bounds(frame) {
                    record_group_clip(&group, bounds, result);
                }
                *child = ShapeTreeChild::GroupShape(Box::new(group));
            }
            Err(reason) => {
                let (fallback, placeholder_error) =
                    match placeholder_group(frame, frame_id, &frame_name, allocator) {
                        Ok(placeholder) => {
                            if let Ok(bounds) = frame_bounds(frame) {
                                record_group_clip(&placeholder, bounds, result);
                            }
                            *child = ShapeTreeChild::GroupShape(Box::new(placeholder));
                            ("labelled placeholder", None)
                        }
                        Err(error) => ("bounds fallback", Some(error)),
                    };
                result.diagnostics.push(Diagnostic {
                    message: format!(
                        "unsupported SmartArt rendered as {fallback}: layout `{identity}`: {reason}{}",
                        placeholder_error
                            .as_deref()
                            .map(|error| format!("; labelled placeholder unavailable: {error}"))
                            .unwrap_or_default()
                    ),
                });
            }
        }
    }
}

fn record_group_clip(group: &CT_GroupShape, bounds: Rect, result: &mut ExpansionResult) {
    for child in &group.children {
        if let Some(id) = child.non_visual_id() {
            result.clips.insert(id, bounds);
        }
    }
}

fn validate_relationship_set(
    resources: &DiagramResources,
    relationships: &DiagramRelationshipIds,
) -> Result<(), String> {
    if resources.relationships.layout != relationships.layout
        || resources.relationships.style != relationships.style
        || resources.relationships.colors != relationships.colors
    {
        return Err("conflicting SmartArt relationship set".to_owned());
    }
    Ok(())
}

fn render_group(
    frame: &CT_GraphicFrame,
    resources: &DiagramResources,
    frame_id: u32,
    frame_name: &str,
    allocator: &mut ShapeIdAllocator,
) -> Result<CT_GroupShape, String> {
    let data = resources.data.as_ref().map_err(Clone::clone)?;
    let layout = resources.layout.as_ref().map_err(Clone::clone)?;
    let style = resources.style.as_ref().map_err(Clone::clone)?;
    let colors = resources.colors.as_ref().map_err(Clone::clone)?;
    let bounds = frame_bounds(frame)?;
    let prepared = prepare(data, layout, style, colors, bounds)?;
    let mut group = CT_GroupShape::new_empty(frame_id, frame_name);
    for connector in &prepared.connectors {
        let connector = styled_connector(allocator.allocate(), connector, style, colors)?;
        group.children.push(ShapeTreeChild::Connector(connector));
    }
    for prepared_shape in &prepared.shapes {
        let shape = styled_shape(allocator.allocate(), prepared_shape, style, colors)?;
        group.children.push(ShapeTreeChild::Shape(shape));
    }
    apply_frame_transform(&mut group, frame)?;
    Ok(group)
}

fn prepare<'a>(
    data: &'a CT_DiagramData,
    layout: &'a CT_DiagramLayoutDefinition,
    style: &'a CT_DiagramStyleDefinition,
    colors: &'a CT_DiagramColorsDefinition,
    bounds: Rect,
) -> Result<PreparedDiagram, String> {
    validate_bounds(bounds)?;
    if data.points().len() > MAX_NODES || data.connections().len() > MAX_CONNECTIONS {
        return Err(format!(
            "SmartArt graph exceeds bounded complexity ({MAX_NODES} nodes, {MAX_CONNECTIONS} connections)"
        ));
    }
    let (nodes, edges) = presentation_graph(data, layout)?;
    if nodes.is_empty() {
        return Err("SmartArt data model has no renderable data nodes".to_owned());
    }
    let root = validate_authentic_layout_program(layout, nodes.len())?;
    let prepared = match &layout.family {
        DiagramLayoutFamily::List => authentic_list(&nodes, bounds, root)?,
        DiagramLayoutFamily::Hierarchy => authentic_hierarchy(&nodes, &edges, bounds, root)?,
        DiagramLayoutFamily::Cycle => {
            validate_cycle1_compatibility_profile(layout, nodes.len())?;
            authentic_cycle(&nodes, bounds)?
        }
        DiagramLayoutFamily::Relationship => authentic_relationship(&nodes, bounds, root)?,
        DiagramLayoutFamily::Matrix => authentic_matrix(&nodes, bounds, root)?,
        DiagramLayoutFamily::Pyramid => authentic_pyramid(&nodes, bounds, root)?,
        DiagramLayoutFamily::Unsupported(name) => {
            return Err(format!("unsupported SmartArt layout family `{name}`"));
        }
    };
    for shape in &prepared.shapes {
        require_owned_label(shape.style_label, style, colors)?;
    }
    for connector in &prepared.connectors {
        require_owned_label(connector.style_label, style, colors)?;
    }
    validate_rects(
        &prepared
            .shapes
            .iter()
            .map(|shape| shape.rect)
            .collect::<Vec<_>>(),
        bounds,
    )?;
    Ok(prepared)
}

fn validate_authentic_layout_program(
    layout: &CT_DiagramLayoutDefinition,
    node_count: usize,
) -> Result<&DiagramRenderInstruction, String> {
    if let DiagramLayoutFamily::Unsupported(reason) = &layout.family {
        return Err(format!("unsupported SmartArt layout family `{reason}`"));
    }
    let root = layout
        .render_projection()
        .root
        .as_ref()
        .ok_or_else(|| "SmartArt layout has no projected instruction root".to_owned())?;
    validate_instruction_tree(root, 0)?;
    validate_parameter_cardinality(root, false)?;
    if !matches!(layout.family, DiagramLayoutFamily::Cycle) {
        validate_identity_instruction_semantics(&layout.family, root, node_count)?;
        validate_identity_instruction_topology(&layout.family, root)?;
        validate_identity_semantic_profile(&layout.family, root)?;
    }
    let evaluation = evaluate_program(root, node_count)?;
    if evaluation.layout_nodes == 0
        || evaluation.shapes == 0
        || evaluation.presentation_mappings == 0
        || evaluation.algorithms == 0
        || evaluation.constraints == 0
        || evaluation.rules == 0
    {
        return Err("SmartArt instruction program omits required executable semantics".to_owned());
    }
    for (name, label) in match layout.family {
        DiagramLayoutFamily::List => &[("parentText", "node1"), ("childText", "conFgAcc1")][..],
        DiagramLayoutFamily::Hierarchy => &[
            ("background", "node0"),
            ("text", "fgAcc0"),
            ("text2", "fgAcc2"),
        ][..],
        DiagramLayoutFamily::Cycle => &[("node", "revTx"), ("sibTrans", "node1")][..],
        DiagramLayoutFamily::Relationship => &[
            ("Parent", "node0"),
            ("Child1", "node1"),
            ("Accent2", "node1"),
        ][..],
        DiagramLayoutFamily::Matrix => &[("tile1", "node1"), ("centerTile", "fgShp")][..],
        DiagramLayoutFamily::Pyramid => &[("acctBkgd", "alignAcc1"), ("levelTx", "revTx")][..],
        DiagramLayoutFamily::Unsupported(_) => unreachable!("unsupported returned above"),
    } {
        require_layout_node(root, name, Some(label))?;
    }
    Ok(root)
}

fn validate_cycle1_compatibility_profile(
    layout: &CT_DiagramLayoutDefinition,
    node_count: usize,
) -> Result<(), String> {
    if node_count != 3 {
        return Err(
            "PowerPoint 16.104 cycle1 compatibility profile requires exactly three data nodes"
                .to_owned(),
        );
    }
    if layout.unique_id.as_deref() != Some(CYCLE1_COMPATIBILITY_ID) {
        return Err(
            "PowerPoint 16.104 cycle1 compatibility profile requires the pinned cycle1 identity"
                .to_owned(),
        );
    }
    let actual_sha256 = Sha256::digest(layout.to_xml())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if actual_sha256 != CYCLE1_COMPATIBILITY_SHA256 {
        return Err(format!(
            "PowerPoint 16.104 cycle1 compatibility profile requires layout SHA-256 {CYCLE1_COMPATIBILITY_SHA256}, found {actual_sha256}"
        ));
    }
    Ok(())
}

fn validate_instruction_tree(
    instruction: &DiagramRenderInstruction,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_GRAPH_DEPTH {
        return Err(format!(
            "SmartArt instruction tree exceeds depth bound {MAX_GRAPH_DEPTH}"
        ));
    }
    if let DiagramRenderInstructionKind::Unsupported(local) = &instruction.kind {
        return Err(format!("unsupported SmartArt instruction `{local}`"));
    }
    let allowed: &[&str] = match &instruction.kind {
        DiagramRenderInstructionKind::LayoutNode => &["name", "styleLbl", "moveWith"],
        DiagramRenderInstructionKind::Algorithm => &["type", "rev"],
        DiagramRenderInstructionKind::Parameter => &["type", "val"],
        DiagramRenderInstructionKind::ConstraintList
        | DiagramRenderInstructionKind::RuleList
        | DiagramRenderInstructionKind::Choose
        | DiagramRenderInstructionKind::Else
        | DiagramRenderInstructionKind::VariableList
        | DiagramRenderInstructionKind::AdjustmentList => &[],
        DiagramRenderInstructionKind::Constraint => &[
            "fact",
            "for",
            "forName",
            "op",
            "ptType",
            "refFor",
            "refForName",
            "refType",
            "type",
            "val",
        ],
        DiagramRenderInstructionKind::Rule => {
            &["fact", "for", "forName", "max", "op", "type", "val"]
        }
        DiagramRenderInstructionKind::ForEach => &[
            "axis",
            "cnt",
            "hideLastTrans",
            "name",
            "ptType",
            "ref",
            "st",
            "step",
        ],
        DiagramRenderInstructionKind::Condition => {
            &["arg", "axis", "func", "name", "op", "ptType", "val"]
        }
        DiagramRenderInstructionKind::Shape => {
            &["blip", "hideGeom", "lkTxEntry", "rot", "type", "zOrderOff"]
        }
        DiagramRenderInstructionKind::PresentationOf => &["axis", "cnt", "ptType", "st", "step"],
        DiagramRenderInstructionKind::Variable(_) => &["val"],
        DiagramRenderInstructionKind::Adjustment => &["idx", "val"],
        DiagramRenderInstructionKind::Unsupported(_) => unreachable!(),
    };
    for (name, _) in &instruction.attributes {
        if !allowed.contains(&name.as_str()) {
            return Err(format!(
                "unsupported SmartArt `{}` attribute `{name}`",
                instruction_kind_name(&instruction.kind)
            ));
        }
    }
    for child in &instruction.children {
        validate_instruction_tree(child, depth + 1)?;
    }
    Ok(())
}

fn instruction_kind_name(kind: &DiagramRenderInstructionKind) -> &str {
    match kind {
        DiagramRenderInstructionKind::LayoutNode => "layoutNode",
        DiagramRenderInstructionKind::Algorithm => "alg",
        DiagramRenderInstructionKind::Parameter => "param",
        DiagramRenderInstructionKind::ConstraintList => "constrLst",
        DiagramRenderInstructionKind::Constraint => "constr",
        DiagramRenderInstructionKind::RuleList => "ruleLst",
        DiagramRenderInstructionKind::Rule => "rule",
        DiagramRenderInstructionKind::ForEach => "forEach",
        DiagramRenderInstructionKind::Choose => "choose",
        DiagramRenderInstructionKind::Condition => "if",
        DiagramRenderInstructionKind::Else => "else",
        DiagramRenderInstructionKind::Shape => "shape",
        DiagramRenderInstructionKind::PresentationOf => "presOf",
        DiagramRenderInstructionKind::VariableList => "varLst",
        DiagramRenderInstructionKind::Variable(_) => "variable",
        DiagramRenderInstructionKind::AdjustmentList => "adjLst",
        DiagramRenderInstructionKind::Adjustment => "adj",
        DiagramRenderInstructionKind::Unsupported(_) => "unsupported",
    }
}

fn instruction_semantic_sha256(instruction: &DiagramRenderInstruction) -> String {
    fn append_field(hasher: &mut Sha256, value: &str) {
        hasher.update((value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }

    fn append_instruction(hasher: &mut Sha256, instruction: &DiagramRenderInstruction) {
        let kind = match &instruction.kind {
            DiagramRenderInstructionKind::Variable(name) => format!("variable:{name}"),
            DiagramRenderInstructionKind::Unsupported(local) => format!("unsupported:{local}"),
            kind => instruction_kind_name(kind).to_owned(),
        };
        append_field(hasher, &kind);

        let mut attributes = instruction.attributes.iter().collect::<Vec<_>>();
        attributes.sort_unstable();
        hasher.update((attributes.len() as u64).to_le_bytes());
        for (name, value) in attributes {
            append_field(hasher, name);
            append_field(hasher, value);
        }

        hasher.update((instruction.children.len() as u64).to_le_bytes());
        for child in &instruction.children {
            append_instruction(hasher, child);
        }
    }

    let mut hasher = Sha256::new();
    append_instruction(&mut hasher, instruction);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validate_identity_semantic_profile(
    family: &DiagramLayoutFamily,
    root: &DiagramRenderInstruction,
) -> Result<(), String> {
    let expected = match family {
        DiagramLayoutFamily::List => {
            "a2935c947647c92c89fca4ef0b563e09451aac0b69dcc30fd82f12700f674da5"
        }
        DiagramLayoutFamily::Hierarchy => {
            "03b184d5239e1122c4d7eb5b56c91acda18a60cb0ec842ec2f652a5d2dbf1fb2"
        }
        DiagramLayoutFamily::Relationship => {
            "d4bbc3b4294bd6622c670b261b75068c8803987a16fb0dade762d11299e12a36"
        }
        DiagramLayoutFamily::Matrix => {
            "3c951f804a30df5be8585cb7b51467e5aff9951064917d0a19c78ac554bb4de8"
        }
        DiagramLayoutFamily::Pyramid => {
            "4e7194aecc77a57994396f9c9577f7468705a5699add1d14893d3f9d8a82680a"
        }
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => {
            return Err("unsupported readable SmartArt semantic profile family".to_owned());
        }
    };
    let actual = instruction_semantic_sha256(root);
    if actual != expected {
        return Err(format!(
            "SmartArt `{}` instruction semantics do not match the exact supported profile: expected {expected}, found {actual}",
            family_name(family)
        ));
    }
    Ok(())
}

fn validate_parameter_cardinality(
    instruction: &DiagramRenderInstruction,
    parent_is_algorithm: bool,
) -> Result<(), String> {
    if instruction.kind == DiagramRenderInstructionKind::Parameter && !parent_is_algorithm {
        return Err("SmartArt parameter occurs outside its owning algorithm".to_owned());
    }
    if instruction.kind == DiagramRenderInstructionKind::Algorithm {
        let algorithm = attribute(&instruction.attributes, "type").unwrap_or("unknown");
        let mut types = HashSet::new();
        for parameter in &instruction.children {
            if parameter.kind != DiagramRenderInstructionKind::Parameter {
                return Err(format!(
                    "SmartArt `{algorithm}` algorithm contains non-parameter child `{}`",
                    instruction_kind_name(&parameter.kind)
                ));
            }
            let parameter_type = attribute(&parameter.attributes, "type")
                .ok_or_else(|| format!("SmartArt `{algorithm}` parameter has no type"))?;
            if attribute(&parameter.attributes, "val").is_none() {
                return Err(format!(
                    "SmartArt `{algorithm}` parameter `{parameter_type}` has no value"
                ));
            }
            if !types.insert(parameter_type) {
                return Err(format!(
                    "SmartArt `{algorithm}` has duplicate parameter `{parameter_type}`"
                ));
            }
        }
    }
    let owns_parameters = instruction.kind == DiagramRenderInstructionKind::Algorithm;
    for child in &instruction.children {
        validate_parameter_cardinality(child, owns_parameters)?;
    }
    Ok(())
}

fn validate_identity_instruction_semantics(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
    node_count: usize,
) -> Result<(), String> {
    validate_identity_instruction_semantics_with_owner(
        family,
        instruction,
        node_count,
        None,
        "layoutNode[0]",
    )
}

fn validate_identity_instruction_semantics_with_owner(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
    node_count: usize,
    owner: Option<&str>,
    path: &str,
) -> Result<(), String> {
    let instruction_owner = if instruction.kind == DiagramRenderInstructionKind::LayoutNode {
        Some(
            attribute(&instruction.attributes, "name").unwrap_or(if owner.is_none() {
                "#root"
            } else {
                "#anonymous"
            }),
        )
    } else {
        owner
    };
    match &instruction.kind {
        DiagramRenderInstructionKind::LayoutNode => {
            validate_layout_node_semantics(family, instruction)?;
        }
        DiagramRenderInstructionKind::Algorithm => {
            let algorithm = attribute(&instruction.attributes, "type")
                .ok_or_else(|| "SmartArt algorithm has no type".to_owned())?;
            if instruction.attributes.len() != 1 {
                return Err(format!(
                    "unsupported SmartArt `{algorithm}` algorithm attributes"
                ));
            }
            let parameters = instruction
                .children
                .iter()
                .filter(|child| child.kind == DiagramRenderInstructionKind::Parameter)
                .map(|parameter| {
                    Ok((
                        attribute(&parameter.attributes, "type")
                            .ok_or_else(|| "SmartArt parameter has no type".to_owned())?,
                        attribute(&parameter.attributes, "val")
                            .ok_or_else(|| "SmartArt parameter has no value".to_owned())?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            if !allowed_algorithm_parameters(family, algorithm, &parameters) {
                return Err(format!(
                    "unsupported SmartArt `{}` algorithm parameter semantics for `{algorithm}`: {parameters:?}",
                    family_name(family)
                ));
            }
            let owner = instruction_owner.unwrap_or("#none");
            if !allowed_algorithm_owner(family, owner, path, algorithm) {
                return Err(format!(
                    "unsupported SmartArt `{}` algorithm `{algorithm}` for owner `{owner}` at `{path}`",
                    family_name(family)
                ));
            }
        }
        DiagramRenderInstructionKind::Shape => {
            if !allowed_shape_semantics(family, instruction) {
                let shape_type = attribute(&instruction.attributes, "type").unwrap_or("none");
                return Err(format!(
                    "unsupported SmartArt `{}` shape semantics for type `{shape_type}`",
                    family_name(family)
                ));
            }
            let owner = instruction_owner.unwrap_or("#none");
            if !allowed_shape_owner(family, owner, instruction) {
                return Err(format!(
                    "unsupported SmartArt `{}` shape semantics for owner `{owner}`: {:?}",
                    family_name(family),
                    instruction.attributes
                ));
            }
        }
        DiagramRenderInstructionKind::ForEach => {
            foreach_count(instruction, node_count)?;
            validate_foreach_semantics(family, instruction)?;
        }
        DiagramRenderInstructionKind::Condition => {
            validate_condition_semantics(family, instruction_owner, path, instruction)?;
        }
        DiagramRenderInstructionKind::PresentationOf => {
            validate_presentation_of_semantics(family, instruction_owner, instruction)?;
        }
        DiagramRenderInstructionKind::Constraint => {
            validate_constraint_semantics(family, instruction_owner, instruction)?;
        }
        DiagramRenderInstructionKind::Rule => {
            validate_rule_semantics(family, instruction_owner, instruction)?;
        }
        DiagramRenderInstructionKind::Variable(name) => {
            validate_variable_semantics(family, instruction_owner, name, instruction)?;
        }
        DiagramRenderInstructionKind::Adjustment => {
            validate_adjustment_semantics(family, instruction_owner, instruction)?;
        }
        _ => {}
    }
    let mut occurrences = HashMap::new();
    for child in &instruction.children {
        let kind = instruction_kind_name(&child.kind);
        let occurrence = occurrences.entry(kind).or_insert(0usize);
        let child_path = format!("{path}/{kind}[{occurrence}]");
        *occurrence += 1;
        validate_identity_instruction_semantics_with_owner(
            family,
            child,
            node_count,
            instruction_owner,
            &child_path,
        )?;
    }
    Ok(())
}

fn attribute_value_signature(instruction: &DiagramRenderInstruction) -> String {
    let mut values = instruction
        .attributes
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>();
    values.sort_unstable();
    values.join(",")
}

fn allowed_shape_owner(
    family: &DiagramLayoutFamily,
    owner: &str,
    instruction: &DiagramRenderInstruction,
) -> bool {
    let signature = attribute_value_signature(instruction);
    match family {
        DiagramLayoutFamily::List => matches!(
            (owner, signature.as_str()),
            (
                "linear" | "parentLin" | "negativeSpace" | "spaceBetweenRectangles",
                ""
            ) | ("parentLeftMargin", "hideGeom=true,type=rect")
                | ("parentText", "type=roundRect")
                | ("childText", "type=rect,zOrderOff=-2")
        ),
        DiagramLayoutFamily::Hierarchy => matches!(
            (owner, signature.as_str()),
            (
                "hierChild1" | "hierChild2" | "hierChild3" | "hierChild4" | "hierChild5",
                ""
            ) | ("hierRoot1" | "hierRoot2" | "hierRoot3" | "hierRoot4", "")
                | ("composite" | "composite2" | "composite3" | "composite4", "")
                | (
                    "background" | "background2" | "background3" | "background4",
                    "type=roundRect"
                )
                | ("text" | "text2" | "text3" | "text4", "type=roundRect")
                | ("#anonymous", "type=conn,zOrderOff=-999")
        ),
        DiagramLayoutFamily::Relationship => {
            (matches!(owner, "#root" | "#anonymous") && signature.is_empty())
                || (owner.starts_with("AccentHold") && signature == "type=ellipse")
                || (matches!(
                    owner,
                    "Parent" | "Child1" | "Child2" | "Child3" | "Child4" | "Child5"
                ) && signature == "type=ellipse")
                || (matches!(
                    owner,
                    "Accent1" | "Accent2" | "Accent3" | "Accent4" | "Accent5" | "Accent6"
                ) && signature == "type=ellipse")
                || (owner.starts_with("Accent") && signature.is_empty())
        }
        DiagramLayoutFamily::Matrix => match owner {
            "diagram" | "matrix" => signature.is_empty(),
            "tile1" => signature == "rot=270,type=round1Rect",
            "tile2" => signature == "type=round1Rect",
            "tile3" => signature == "rot=180,type=round1Rect",
            "tile4" => signature == "rot=90,type=round1Rect",
            "tile1text" => signature == "hideGeom=true,rot=270,type=rect",
            "tile2text" => signature == "hideGeom=true,type=rect",
            "tile3text" => signature == "hideGeom=true,rot=180,type=rect",
            "tile4text" => signature == "hideGeom=true,rot=90,type=rect",
            "centerTile" => signature == "type=roundRect",
            _ => false,
        },
        DiagramLayoutFamily::Pyramid => match owner {
            "#root" | "#anonymous" => signature.is_empty(),
            "acctBkgd" => signature == "type=nonIsoscelesTrapezoid",
            "acctTx" => signature == "hideGeom=true,type=nonIsoscelesTrapezoid",
            "level" => signature == "type=trapezoid",
            "levelTx" => signature == "hideGeom=true,type=rect",
            _ => false,
        },
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    }
}

fn allowed_algorithm_owner(
    family: &DiagramLayoutFamily,
    owner: &str,
    path: &str,
    algorithm: &str,
) -> bool {
    match family {
        DiagramLayoutFamily::List => matches!(
            (owner, algorithm),
            ("linear" | "parentLin", "lin")
                | (
                    "parentLeftMargin" | "negativeSpace" | "spaceBetweenRectangles",
                    "sp"
                )
                | ("parentText" | "childText", "tx")
        ),
        DiagramLayoutFamily::Hierarchy => matches!(
            (owner, algorithm),
            (
                "hierChild1" | "hierChild2" | "hierChild3" | "hierChild4" | "hierChild5",
                "hierChild"
            ) | (
                "hierRoot1" | "hierRoot2" | "hierRoot3" | "hierRoot4",
                "hierRoot"
            ) | (
                "composite" | "composite2" | "composite3" | "composite4",
                "composite"
            ) | (
                "background" | "background2" | "background3" | "background4",
                "sp"
            ) | ("text" | "text2" | "text3" | "text4", "tx")
                | ("#anonymous", "conn")
        ),
        DiagramLayoutFamily::Relationship => {
            matches!(algorithm,
                "composite" if owner == "#root" || owner == "#anonymous" || owner.starts_with("AccentHold")
            ) || matches!(algorithm, "tx" if matches!(owner, "Parent" | "Child1" | "Child2" | "Child3" | "Child4" | "Child5"))
                || matches!(algorithm, "sp" if owner.starts_with("Accent"))
        }
        DiagramLayoutFamily::Matrix => matches!(
            (owner, algorithm),
            ("diagram" | "matrix", "composite")
                | ("tile1" | "tile2" | "tile3" | "tile4", "sp")
                | (
                    "tile1text" | "tile2text" | "tile3text" | "tile4text" | "centerTile",
                    "tx"
                )
        ),
        DiagramLayoutFamily::Pyramid => match owner {
            "#root" | "#anonymous" if path.starts_with("layoutNode[0]/choose[0]/") => {
                algorithm == "pyra"
            }
            "#anonymous" => algorithm == "composite",
            "acctBkgd" | "level" => algorithm == "sp",
            "acctTx" | "levelTx" => algorithm == "tx",
            _ => false,
        },
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    }
}

fn validate_condition_semantics(
    family: &DiagramLayoutFamily,
    owner: Option<&str>,
    path: &str,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    let values = (
        attribute(&instruction.attributes, "axis"),
        attribute(&instruction.attributes, "ptType"),
        attribute(&instruction.attributes, "func"),
        attribute(&instruction.attributes, "arg"),
        attribute(&instruction.attributes, "op"),
        attribute(&instruction.attributes, "val"),
        attribute(&instruction.attributes, "name"),
    );
    let owner = owner.unwrap_or("#none");
    let valid = match family {
        DiagramLayoutFamily::List => {
            values
                == (
                    None,
                    None,
                    Some("var"),
                    Some("dir"),
                    Some("equ"),
                    Some("norm"),
                    None,
                )
                && matches!(owner, "linear" | "parentLin" | "parentText")
        }
        DiagramLayoutFamily::Hierarchy => {
            (values
                == (
                    None,
                    None,
                    Some("var"),
                    Some("dir"),
                    Some("equ"),
                    Some("norm"),
                    None,
                )
                && matches!(
                    owner,
                    "hierChild1" | "hierChild2" | "hierChild3" | "hierChild4" | "hierChild5"
                ))
                || (values
                    == (
                        Some("self"),
                        None,
                        Some("depth"),
                        None,
                        Some("lte"),
                        Some("4"),
                        None,
                    )
                    && owner == "#anonymous")
        }
        DiagramLayoutFamily::Relationship => {
            (values.0 == Some("ch ch")
                && values.1 == Some("node node")
                && values.2 == Some("cnt")
                && values.3.is_none()
                && values.4 == Some("equ")
                && matches!(values.5, Some("0" | "1" | "2" | "3" | "4"))
                && values.6.is_none()
                && path.starts_with("layoutNode[0]/choose[0]/"))
                || values
                    == (
                        Some("ch"),
                        Some("node"),
                        Some("cnt"),
                        None,
                        Some("lte"),
                        Some("4"),
                        None,
                    )
        }
        DiagramLayoutFamily::Matrix => {
            values
                == (
                    Some("ch"),
                    Some("node"),
                    Some("cnt"),
                    None,
                    Some("gte"),
                    Some("1"),
                    None,
                )
                || values
                    == (
                        Some("root des"),
                        None,
                        Some("maxDepth"),
                        None,
                        Some("gte"),
                        Some("3"),
                        None,
                    )
                || values
                    == (
                        None,
                        None,
                        Some("var"),
                        Some("dir"),
                        Some("equ"),
                        Some("norm"),
                        None,
                    )
        }
        DiagramLayoutFamily::Pyramid => {
            values
                == (
                    None,
                    None,
                    Some("var"),
                    Some("dir"),
                    Some("equ"),
                    Some("norm"),
                    None,
                )
                || values
                    == (
                        Some("root des"),
                        Some("all node"),
                        Some("maxDepth"),
                        None,
                        Some("gte"),
                        Some("2"),
                        None,
                    )
                || values
                    == (
                        Some("self"),
                        Some("node"),
                        Some("pos"),
                        None,
                        Some("equ"),
                        Some("1"),
                        None,
                    )
                || values
                    == (
                        Some("ch"),
                        Some("node"),
                        Some("cnt"),
                        None,
                        Some("gte"),
                        Some("1"),
                        None,
                    )
        }
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    if !valid {
        return Err(format!(
            "unsupported SmartArt `{}` condition semantics {values:?} for owner `{owner}` at `{path}`",
            family_name(family)
        ));
    }
    Ok(())
}

fn family_name(family: &DiagramLayoutFamily) -> &str {
    match family {
        DiagramLayoutFamily::List => "list1",
        DiagramLayoutFamily::Hierarchy => "hierarchy1",
        DiagramLayoutFamily::Cycle => "cycle1",
        DiagramLayoutFamily::Relationship => "CircleRelationship",
        DiagramLayoutFamily::Matrix => "matrix1",
        DiagramLayoutFamily::Pyramid => "pyramid1",
        DiagramLayoutFamily::Unsupported(name) => name,
    }
}

fn validate_identity_instruction_topology(
    family: &DiagramLayoutFamily,
    root: &DiagramRenderInstruction,
) -> Result<(), String> {
    let expected_count = match family {
        DiagramLayoutFamily::List => 7,
        DiagramLayoutFamily::Hierarchy => 24,
        DiagramLayoutFamily::Relationship => 25,
        DiagramLayoutFamily::Matrix => 11,
        DiagramLayoutFamily::Pyramid => 6,
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => {
            return Err("unsupported readable SmartArt topology family".to_owned());
        }
    };
    let mut consumed = HashSet::new();
    validate_instruction_owners(family, root, None, "ROOT", "layoutNode[0]", &mut consumed)?;
    if consumed.len() != expected_count {
        return Err(format!(
            "SmartArt `{}` topology requires {expected_count} exact layout-node profiles, found {}",
            family_name(family),
            consumed.len()
        ));
    }
    Ok(())
}

fn validate_instruction_owners(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
    parent_kind: Option<&DiagramRenderInstructionKind>,
    layout_owner: &str,
    path: &str,
    consumed: &mut HashSet<String>,
) -> Result<(), String> {
    let parent = parent_kind.map(instruction_kind_name).unwrap_or("ROOT");
    let valid_parent = match instruction.kind {
        DiagramRenderInstructionKind::LayoutNode => {
            matches!(parent, "ROOT" | "layoutNode" | "forEach" | "if" | "else")
        }
        DiagramRenderInstructionKind::Algorithm => matches!(parent, "layoutNode" | "if" | "else"),
        DiagramRenderInstructionKind::Parameter => parent == "alg",
        DiagramRenderInstructionKind::ConstraintList => {
            matches!(parent, "layoutNode" | "if" | "else")
        }
        DiagramRenderInstructionKind::Constraint => parent == "constrLst",
        DiagramRenderInstructionKind::RuleList => matches!(parent, "layoutNode" | "if" | "else"),
        DiagramRenderInstructionKind::Rule => parent == "ruleLst",
        DiagramRenderInstructionKind::ForEach => matches!(parent, "layoutNode" | "forEach"),
        DiagramRenderInstructionKind::Choose => matches!(parent, "layoutNode" | "forEach"),
        DiagramRenderInstructionKind::Condition | DiagramRenderInstructionKind::Else => {
            parent == "choose"
        }
        DiagramRenderInstructionKind::Shape | DiagramRenderInstructionKind::PresentationOf => {
            matches!(parent, "layoutNode" | "if" | "else")
        }
        DiagramRenderInstructionKind::VariableList => parent == "layoutNode",
        DiagramRenderInstructionKind::Variable(_) => parent == "varLst",
        DiagramRenderInstructionKind::AdjustmentList => parent == "shape",
        DiagramRenderInstructionKind::Adjustment => parent == "adjLst",
        DiagramRenderInstructionKind::Unsupported(_) => false,
    };
    if !valid_parent {
        return Err(format!(
            "SmartArt `{}` topology rejects `{}` under `{parent}` for layout owner `{layout_owner}`",
            family_name(family),
            instruction_kind_name(&instruction.kind)
        ));
    }

    let next_owner = if instruction.kind == DiagramRenderInstructionKind::LayoutNode {
        let name = attribute(&instruction.attributes, "name").unwrap_or("#anonymous");
        let children = instruction_child_signature(instruction);
        let semantics = owned_semantic_counts(instruction)?;
        let Some((expected_children, expected_semantics)) =
            expected_layout_node_profile(family, name, layout_owner)
        else {
            return Err(format!(
                "SmartArt `{}` topology has unsupported layout node `{name}` under `{layout_owner}` with child cardinalities {children:?} and owned semantic cardinalities {semantics:?}",
                family_name(family)
            ));
        };
        if children != expected_children || semantics != expected_semantics {
            return Err(format!(
                "SmartArt `{}` topology has unsupported cardinalities or ordered children for layout node `{name}` under `{layout_owner}`: expected {expected_children:?}/{expected_semantics:?}, found {children:?}/{semantics:?}",
                family_name(family)
            ));
        }
        if !consumed.insert(format!("{layout_owner}\0{name}")) {
            return Err(format!(
                "SmartArt `{}` topology repeats layout node `{name}` under `{layout_owner}` at `{path}`",
                family_name(family)
            ));
        }
        name
    } else {
        layout_owner
    };

    validate_container_children(family, instruction, next_owner, path)?;
    let mut occurrences = HashMap::new();
    for child in &instruction.children {
        let kind = instruction_kind_name(&child.kind);
        let occurrence = occurrences.entry(kind).or_insert(0usize);
        let child_path = format!("{path}/{kind}[{occurrence}]");
        *occurrence += 1;
        validate_instruction_owners(
            family,
            child,
            Some(&instruction.kind),
            next_owner,
            &child_path,
            consumed,
        )?;
    }
    Ok(())
}

fn instruction_child_signature(instruction: &DiagramRenderInstruction) -> String {
    instruction
        .children
        .iter()
        .map(|child| instruction_kind_name(&child.kind))
        .collect::<Vec<_>>()
        .join(",")
}

fn owned_semantic_counts(instruction: &DiagramRenderInstruction) -> Result<[u16; 4], String> {
    fn collect(
        instruction: &DiagramRenderInstruction,
        counts: &mut [u16; 4],
    ) -> Result<(), String> {
        for child in &instruction.children {
            if child.kind == DiagramRenderInstructionKind::LayoutNode {
                continue;
            }
            let index = match child.kind {
                DiagramRenderInstructionKind::Constraint => Some(0),
                DiagramRenderInstructionKind::Rule => Some(1),
                DiagramRenderInstructionKind::Variable(_) => Some(2),
                DiagramRenderInstructionKind::Adjustment => Some(3),
                _ => None,
            };
            if let Some(index) = index {
                counts[index] = counts[index]
                    .checked_add(1)
                    .ok_or_else(|| "SmartArt owned semantic cardinality overflow".to_owned())?;
            }
            collect(child, counts)?;
        }
        Ok(())
    }
    let mut counts = [0u16; 4];
    collect(instruction, &mut counts)?;
    Ok(counts)
}

fn validate_container_children(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
    owner: &str,
    path: &str,
) -> Result<(), String> {
    let child_signature = instruction_child_signature(instruction);
    let valid = match instruction.kind {
        DiagramRenderInstructionKind::LayoutNode => true,
        DiagramRenderInstructionKind::Algorithm => instruction
            .children
            .iter()
            .all(|child| child.kind == DiagramRenderInstructionKind::Parameter),
        DiagramRenderInstructionKind::ConstraintList => {
            instruction
                .children
                .iter()
                .all(|child| child.kind == DiagramRenderInstructionKind::Constraint)
                && expected_constraint_count(family, owner, path)
                    .is_some_and(|expected| instruction.children.len() == expected)
        }
        DiagramRenderInstructionKind::RuleList => instruction
            .children
            .iter()
            .all(|child| child.kind == DiagramRenderInstructionKind::Rule),
        DiagramRenderInstructionKind::VariableList => expected_variable_sequence(family, owner)
            .is_some_and(|expected| {
                instruction
                    .children
                    .iter()
                    .map(|child| match &child.kind {
                        DiagramRenderInstructionKind::Variable(name) => name.as_str(),
                        _ => "#invalid",
                    })
                    .eq(expected.iter().copied())
            }),
        DiagramRenderInstructionKind::AdjustmentList => child_signature == "adj",
        DiagramRenderInstructionKind::Shape => matches!(child_signature.as_str(), "" | "adjLst"),
        DiagramRenderInstructionKind::ForEach => {
            expected_foreach_child_signature(family, owner, instruction)
                .is_some_and(|expected| child_signature == expected)
        }
        DiagramRenderInstructionKind::Choose => {
            expected_choose_child_signature(family, owner, path)
                .is_some_and(|expected| child_signature == expected)
        }
        DiagramRenderInstructionKind::Condition | DiagramRenderInstructionKind::Else => {
            expected_branch_child_signature(family, owner, path, instruction)
                .is_some_and(|expected| child_signature == expected)
        }
        DiagramRenderInstructionKind::Parameter
        | DiagramRenderInstructionKind::Constraint
        | DiagramRenderInstructionKind::Rule
        | DiagramRenderInstructionKind::PresentationOf
        | DiagramRenderInstructionKind::Variable(_)
        | DiagramRenderInstructionKind::Adjustment => child_signature.is_empty(),
        _ => false,
    };
    if !valid {
        return Err(format!(
            "SmartArt topology rejects `{}` ordered children `{child_signature}` under layout owner `{owner}` at `{path}`",
            instruction_kind_name(&instruction.kind)
        ));
    }
    Ok(())
}

fn expected_constraint_count(
    family: &DiagramLayoutFamily,
    owner: &str,
    path: &str,
) -> Option<usize> {
    match family {
        DiagramLayoutFamily::List => match owner {
            "linear" => Some(20),
            "parentLeftMargin" | "childText" => Some(1),
            "parentText" => Some(2),
            "parentLin" | "negativeSpace" | "spaceBetweenRectangles" => Some(0),
            _ => None,
        },
        DiagramLayoutFamily::Hierarchy => match owner {
            "hierChild1" => Some(22),
            "hierRoot1" | "hierRoot2" | "hierRoot3" | "hierRoot4" => Some(1),
            "composite" | "composite2" | "composite3" | "composite4" => Some(8),
            "text" | "text2" | "text3" | "text4" => Some(4),
            "#anonymous" => Some(2),
            "background" | "background2" | "background3" | "background4" | "hierChild2"
            | "hierChild3" | "hierChild4" | "hierChild5" => Some(0),
            _ => None,
        },
        DiagramLayoutFamily::Relationship => match owner {
            "#anonymous" => {
                if path.contains("/if[0]/") {
                    Some(29)
                } else if path.contains("/if[1]/") {
                    Some(41)
                } else if path.contains("/if[2]/") {
                    Some(57)
                } else if path.contains("/if[3]/") {
                    Some(65)
                } else if path.contains("/if[4]/") {
                    Some(73)
                } else if path.contains("/else[0]/") {
                    Some(85)
                } else {
                    None
                }
            }
            "Parent" | "Child1" | "Child2" | "Child3" | "Child4" | "Child5" => Some(4),
            owner if owner.starts_with("Accent") => Some(0),
            _ => None,
        },
        DiagramLayoutFamily::Matrix => match owner {
            "diagram" => Some(9),
            "matrix" => Some(32),
            "centerTile" => Some(4),
            owner if owner.starts_with("tile") => Some(0),
            _ => None,
        },
        DiagramLayoutFamily::Pyramid => match owner {
            "acctBkgd" => Some(0),
            "acctTx" => Some(6),
            "level" => Some(2),
            "levelTx" => Some(5),
            "#anonymous" if path.starts_with("layoutNode[0]/choose[1]/") => Some(3),
            "#anonymous" => Some(16),
            _ => None,
        },
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => None,
    }
}

fn expected_variable_sequence(
    family: &DiagramLayoutFamily,
    owner: &str,
) -> Option<&'static [&'static str]> {
    match family {
        DiagramLayoutFamily::List => match owner {
            "linear" => Some(&["dir", "animLvl", "resizeHandles"]),
            "parentText" => Some(&["chMax", "bulletEnabled"]),
            "childText" => Some(&["bulletEnabled"]),
            _ => None,
        },
        DiagramLayoutFamily::Hierarchy => match owner {
            "hierChild1" => Some(&["chPref", "dir", "animOne", "animLvl", "resizeHandles"]),
            "text" | "text2" | "text3" | "text4" => Some(&["chPref"]),
            _ => None,
        },
        DiagramLayoutFamily::Relationship => match owner {
            "#root" | "#anonymous" | "Parent" | "Child1" | "Child2" | "Child3" | "Child4"
            | "Child5" => Some(&["chMax", "chPref"]),
            _ => None,
        },
        DiagramLayoutFamily::Matrix => match owner {
            "diagram" => Some(&["chMax", "dir", "animLvl", "resizeHandles"]),
            "tile1text" | "tile2text" | "tile3text" | "tile4text" => {
                Some(&["chMax", "chPref", "bulletEnabled"])
            }
            "centerTile" => Some(&["chMax", "chPref"]),
            _ => None,
        },
        DiagramLayoutFamily::Pyramid => match owner {
            "#root" | "#anonymous" => Some(&["dir", "animLvl", "resizeHandles"]),
            "acctTx" => Some(&["bulletEnabled"]),
            "level" | "levelTx" => Some(&["chMax", "bulletEnabled"]),
            _ => None,
        },
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => None,
    }
}

fn expected_foreach_child_signature(
    family: &DiagramLayoutFamily,
    owner: &str,
    instruction: &DiagramRenderInstruction,
) -> Option<&'static str> {
    let values = (
        attribute(&instruction.attributes, "name"),
        attribute(&instruction.attributes, "ref"),
        attribute(&instruction.attributes, "axis"),
        attribute(&instruction.attributes, "ptType"),
        attribute(&instruction.attributes, "st"),
        attribute(&instruction.attributes, "cnt"),
    );
    match family {
        DiagramLayoutFamily::List => match values {
            (None, None, Some("ch"), Some("node"), None, None) => {
                Some("layoutNode,layoutNode,layoutNode,forEach")
            }
            (None, None, Some("followSib"), Some("sibTrans"), None, Some("1")) => {
                Some("layoutNode")
            }
            _ => None,
        },
        DiagramLayoutFamily::Hierarchy => match values {
            (None, Some("repeat"), None, None, None, None) => Some(""),
            (None, None, Some("self"), Some("node" | "parTrans"), None, None | Some("1")) => {
                Some("layoutNode")
            }
            (None | Some("repeat"), None, Some("ch"), Some("all"), None, None)
                if owner == "hierChild1" =>
            {
                Some("forEach")
            }
            (None | Some("repeat"), None, Some("ch"), Some("all"), None, None) => {
                Some("forEach,forEach")
            }
            _ => None,
        },
        DiagramLayoutFamily::Relationship => match values {
            (Some("wrapper"), None, Some("self"), Some("parTrans"), Some("1"), Some("0")) => {
                Some("forEach,forEach,forEach")
            }
            (
                Some("accentRepeat1" | "accentRepeat2" | "accentRepeat3"),
                None,
                Some("self"),
                None,
                None,
                None,
            ) => Some("layoutNode"),
            (
                None,
                Some("accentRepeat1" | "accentRepeat2" | "accentRepeat3"),
                None,
                None,
                None,
                None,
            ) => Some(""),
            (None, None, Some("ch"), Some("node"), Some("1"), Some("1")) => {
                Some("layoutNode,choose,layoutNode,layoutNode,layoutNode,layoutNode,layoutNode")
            }
            (None, None, Some("ch ch"), Some("node node"), Some("1 1"), Some("1 1")) => {
                Some("layoutNode,layoutNode,layoutNode")
            }
            (None, None, Some("ch ch"), Some("node node"), Some("1 2"), Some("1 1")) => {
                Some("layoutNode,layoutNode,layoutNode,layoutNode")
            }
            (None, None, Some("ch ch"), Some("node node"), Some("1 3" | "1 4"), Some("1 1")) => {
                Some("layoutNode,layoutNode")
            }
            (None, None, Some("ch ch"), Some("node node"), Some("1 5"), Some("1 1")) => {
                Some("layoutNode,layoutNode,layoutNode")
            }
            _ => None,
        },
        DiagramLayoutFamily::Pyramid => match values {
            (None, None, Some("ch"), Some("node"), None, None) => Some("layoutNode"),
            _ => None,
        },
        DiagramLayoutFamily::Matrix
        | DiagramLayoutFamily::Cycle
        | DiagramLayoutFamily::Unsupported(_) => None,
    }
}

fn expected_choose_child_signature(
    family: &DiagramLayoutFamily,
    _owner: &str,
    path: &str,
) -> Option<&'static str> {
    match family {
        DiagramLayoutFamily::Relationship if path.contains("/forEach[1]/choose[0]") => Some("if"),
        DiagramLayoutFamily::Relationship => Some("if,if,if,if,if,else"),
        DiagramLayoutFamily::List
        | DiagramLayoutFamily::Hierarchy
        | DiagramLayoutFamily::Matrix
        | DiagramLayoutFamily::Pyramid => Some("if,else"),
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => None,
    }
}

fn expected_branch_child_signature(
    family: &DiagramLayoutFamily,
    owner: &str,
    path: &str,
    instruction: &DiagramRenderInstruction,
) -> Option<&'static str> {
    match family {
        DiagramLayoutFamily::List | DiagramLayoutFamily::Hierarchy => Some("alg"),
        DiagramLayoutFamily::Relationship => {
            if path.contains("/forEach[1]/choose[0]/") {
                Some("layoutNode")
            } else if instruction.kind == DiagramRenderInstructionKind::Else
                || attribute(&instruction.attributes, "func") == Some("cnt")
            {
                Some("alg,constrLst")
            } else {
                None
            }
        }
        DiagramLayoutFamily::Matrix => {
            if owner == "diagram" {
                if instruction.kind == DiagramRenderInstructionKind::Condition {
                    Some("layoutNode,layoutNode")
                } else {
                    Some("")
                }
            } else if (owner.starts_with("tile") && !owner.ends_with("text"))
                || (owner.ends_with("text") && path.contains("/choose[1]/"))
            {
                Some("presOf")
            } else {
                Some("alg")
            }
        }
        DiagramLayoutFamily::Pyramid => {
            if path.starts_with("layoutNode[0]/choose[0]/") {
                Some("alg")
            } else if path.starts_with("layoutNode[0]/choose[1]/") {
                Some("constrLst")
            } else if instruction.kind == DiagramRenderInstructionKind::Else
                && path.contains("/choose[1]/")
            {
                Some("")
            } else if path.contains("/choose[1]/") {
                Some("layoutNode,layoutNode")
            } else {
                Some("constrLst")
            }
        }
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => None,
    }
}

fn expected_layout_node_profile(
    family: &DiagramLayoutFamily,
    name: &str,
    parent: &str,
) -> Option<(&'static str, [u16; 4])> {
    const ASPKR: &str = "alg,shape,presOf,constrLst,ruleLst";
    const VASPKR: &str = "varLst,alg,shape,presOf,constrLst,ruleLst";
    const AS_PKR_LL: &str = "alg,shape,presOf,constrLst,ruleLst,layoutNode,layoutNode";
    const CS_PKRF: &str = "choose,shape,presOf,constrLst,ruleLst,forEach";
    let profile = match family {
        DiagramLayoutFamily::List => match (name, parent) {
            ("linear", "ROOT") => (
                "varLst,choose,shape,presOf,constrLst,ruleLst,forEach",
                [20, 1, 3, 0],
            ),
            ("parentLin", "linear") => (
                "choose,shape,presOf,constrLst,ruleLst,layoutNode,layoutNode",
                [0; 4],
            ),
            ("parentLeftMargin", "parentLin") => (ASPKR, [1, 0, 0, 0]),
            ("parentText", "parentLin") => {
                ("varLst,choose,shape,presOf,constrLst,ruleLst", [2, 0, 2, 0])
            }
            ("negativeSpace" | "spaceBetweenRectangles", "linear") => (ASPKR, [0; 4]),
            ("childText", "linear") => (VASPKR, [1, 1, 1, 0]),
            _ => return None,
        },
        DiagramLayoutFamily::Hierarchy => match (name, parent) {
            ("hierChild1", "ROOT") => (
                "varLst,choose,shape,presOf,constrLst,ruleLst,forEach",
                [22, 0, 5, 0],
            ),
            ("hierChild2", "hierRoot1")
            | ("hierChild3", "hierRoot2")
            | ("hierChild4", "hierRoot3")
            | ("hierChild5", "hierRoot4") => (CS_PKRF, [0; 4]),
            ("hierRoot1", "hierChild1")
            | ("hierRoot2", "hierChild2")
            | ("hierRoot3", "hierChild3")
            | ("hierRoot4", "hierChild4") => (AS_PKR_LL, [1, 0, 0, 0]),
            ("composite", "hierRoot1")
            | ("composite2", "hierRoot2")
            | ("composite3", "hierRoot3")
            | ("composite4", "hierRoot4") => (AS_PKR_LL, [8, 0, 0, 0]),
            ("background", "composite")
            | ("background2", "composite2")
            | ("background3", "composite3")
            | ("background4", "composite4") => (ASPKR, [0, 0, 0, 1]),
            ("text", "composite")
            | ("text2", "composite2")
            | ("text3", "composite3")
            | ("text4", "composite4") => (VASPKR, [4, 1, 1, 1]),
            ("#anonymous", "hierChild2" | "hierChild3") => (ASPKR, [2, 0, 0, 0]),
            ("#anonymous", "hierChild4") => ("choose,shape,presOf,constrLst,ruleLst", [2, 0, 0, 0]),
            _ => return None,
        },
        DiagramLayoutFamily::Relationship if parent == "#anonymous" => match name {
            "AccentHold1" | "AccentHold2" | "AccentHold3" => ("alg,shape,presOf", [0; 4]),
            "Parent" | "Child1" | "Child2" | "Child3" | "Child4" | "Child5" => {
                (VASPKR, [4, 1, 2, 0])
            }
            "Accent1" | "Accent2" | "Accent3" | "Accent4" | "Accent5" | "Accent6" => {
                ("alg,shape,presOf,constrLst", [0; 4])
            }
            "Accent7" | "Accent8" | "Accent9" | "Accent10" | "Accent11" | "Accent12"
            | "Accent13" | "Accent15" | "Accent16" => {
                ("alg,shape,presOf,constrLst,forEach", [0; 4])
            }
            _ => return None,
        },
        DiagramLayoutFamily::Relationship if (name, parent) == ("#anonymous", "ROOT") => (
            "varLst,shape,choose,forEach,forEach,forEach,forEach,forEach,forEach,forEach",
            [350, 0, 2, 0],
        ),
        DiagramLayoutFamily::Matrix => match (name, parent) {
            ("diagram", "ROOT") => (
                "varLst,alg,shape,presOf,constrLst,ruleLst,choose",
                [9, 0, 4, 0],
            ),
            ("matrix", "diagram") => (
                "alg,shape,presOf,constrLst,ruleLst,layoutNode,layoutNode,layoutNode,layoutNode,layoutNode,layoutNode,layoutNode,layoutNode",
                [32, 0, 0, 0],
            ),
            ("tile1" | "tile2" | "tile3" | "tile4", "matrix") => {
                ("alg,shape,choose,constrLst,ruleLst", [0; 4])
            }
            ("tile1text", "matrix") => {
                ("varLst,choose,shape,choose,constrLst,ruleLst", [0, 1, 3, 1])
            }
            ("tile2text" | "tile3text" | "tile4text", "matrix") => {
                ("varLst,choose,shape,choose,constrLst,ruleLst", [0, 1, 3, 0])
            }
            ("centerTile", "diagram") => (VASPKR, [4, 1, 2, 0]),
            _ => return None,
        },
        DiagramLayoutFamily::Pyramid => match (name, parent) {
            ("#anonymous", "ROOT") => (
                "varLst,choose,shape,presOf,choose,ruleLst,forEach",
                [6, 0, 3, 0],
            ),
            ("#anonymous", "#anonymous") => (
                "alg,shape,presOf,choose,ruleLst,choose,layoutNode,layoutNode",
                [32, 0, 0, 0],
            ),
            ("acctBkgd", "#anonymous") => (ASPKR, [0; 4]),
            ("acctTx", "#anonymous") => (VASPKR, [6, 1, 1, 0]),
            ("level", "#anonymous") => (VASPKR, [2, 0, 2, 0]),
            ("levelTx", "#anonymous") => (VASPKR, [5, 1, 2, 0]),
            _ => return None,
        },
        DiagramLayoutFamily::Cycle
        | DiagramLayoutFamily::Unsupported(_)
        | DiagramLayoutFamily::Relationship => return None,
    };
    Some(profile)
}

fn validate_layout_node_semantics(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    let name = attribute(&instruction.attributes, "name");
    let style = attribute(&instruction.attributes, "styleLbl");
    let move_with = attribute(&instruction.attributes, "moveWith");
    let valid_name = name.is_none()
        || match family {
            DiagramLayoutFamily::List => matches!(
                name,
                Some(
                    "childText"
                        | "linear"
                        | "negativeSpace"
                        | "parentLeftMargin"
                        | "parentLin"
                        | "parentText"
                        | "spaceBetweenRectangles"
                )
            ),
            DiagramLayoutFamily::Hierarchy => matches!(
                name,
                Some(
                    "background"
                        | "background2"
                        | "background3"
                        | "background4"
                        | "composite"
                        | "composite2"
                        | "composite3"
                        | "composite4"
                        | "hierChild1"
                        | "hierChild2"
                        | "hierChild3"
                        | "hierChild4"
                        | "hierChild5"
                        | "hierRoot1"
                        | "hierRoot2"
                        | "hierRoot3"
                        | "hierRoot4"
                        | "text"
                        | "text2"
                        | "text3"
                        | "text4"
                )
            ),
            DiagramLayoutFamily::Relationship => matches!(
                name,
                Some(
                    "Accent1"
                        | "Accent2"
                        | "Accent3"
                        | "Accent4"
                        | "Accent5"
                        | "Accent6"
                        | "Accent7"
                        | "Accent8"
                        | "Accent9"
                        | "Accent10"
                        | "Accent11"
                        | "Accent12"
                        | "Accent13"
                        | "Accent15"
                        | "Accent16"
                        | "AccentHold1"
                        | "AccentHold2"
                        | "AccentHold3"
                        | "Child1"
                        | "Child2"
                        | "Child3"
                        | "Child4"
                        | "Child5"
                        | "Parent"
                )
            ),
            DiagramLayoutFamily::Matrix => matches!(
                name,
                Some(
                    "centerTile"
                        | "diagram"
                        | "matrix"
                        | "tile1"
                        | "tile1text"
                        | "tile2"
                        | "tile2text"
                        | "tile3"
                        | "tile3text"
                        | "tile4"
                        | "tile4text"
                )
            ),
            DiagramLayoutFamily::Pyramid => {
                matches!(name, Some("acctBkgd" | "acctTx" | "level" | "levelTx"))
            }
            DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
        };
    let valid_style = style.is_none()
        || match family {
            DiagramLayoutFamily::List => matches!(style, Some("conFgAcc1" | "node1")),
            DiagramLayoutFamily::Hierarchy => {
                matches!(
                    style,
                    Some("fgAcc0" | "fgAcc2" | "fgAcc3" | "fgAcc4" | "node0")
                )
            }
            DiagramLayoutFamily::Relationship => matches!(style, Some("node0" | "node1")),
            DiagramLayoutFamily::Matrix => matches!(style, Some("fgShp" | "node1")),
            DiagramLayoutFamily::Pyramid => matches!(style, Some("alignAcc1" | "revTx")),
            DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
        };
    let valid_move = move_with.is_none()
        || matches!(
            (family, move_with),
            (
                DiagramLayoutFamily::Hierarchy,
                Some("text" | "text2" | "text3" | "text4")
            )
        );
    if !valid_name || !valid_style || !valid_move {
        return Err(format!(
            "unsupported SmartArt `{}` layout-node ownership name={name:?} style={style:?} moveWith={move_with:?}",
            family_name(family)
        ));
    }
    Ok(())
}

fn validate_foreach_semantics(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    let values = (
        attribute(&instruction.attributes, "name"),
        attribute(&instruction.attributes, "ref"),
        attribute(&instruction.attributes, "axis"),
        attribute(&instruction.attributes, "ptType"),
        attribute(&instruction.attributes, "st"),
        attribute(&instruction.attributes, "cnt"),
        attribute(&instruction.attributes, "step"),
        attribute(&instruction.attributes, "hideLastTrans"),
    );
    let valid = match family {
        DiagramLayoutFamily::List => matches!(
            values,
            (None, None, Some("ch"), Some("node"), None, None, None, None)
                | (
                    None,
                    None,
                    Some("followSib"),
                    Some("sibTrans"),
                    None,
                    Some("1"),
                    None,
                    None
                )
        ),
        DiagramLayoutFamily::Hierarchy => matches!(
            values,
            (None, None, Some("ch"), Some("all"), None, None, None, None)
                | (
                    None,
                    None,
                    Some("self"),
                    Some("node"),
                    None,
                    None,
                    None,
                    None
                )
                | (
                    None,
                    None,
                    Some("self"),
                    Some("parTrans"),
                    None,
                    Some("1"),
                    None,
                    None
                )
                | (
                    Some("repeat"),
                    None,
                    Some("ch"),
                    Some("all"),
                    None,
                    None,
                    None,
                    None
                )
                | (None, Some("repeat"), None, None, None, None, None, None)
        ),
        DiagramLayoutFamily::Relationship => {
            let numbered = matches!(
                values,
                (
                    None,
                    None,
                    Some("ch ch"),
                    Some("node node"),
                    Some("1 1" | "1 2" | "1 3" | "1 4" | "1 5"),
                    Some("1 1"),
                    None,
                    None
                )
            );
            numbered
                || matches!(
                    values,
                    (
                        None,
                        None,
                        Some("ch"),
                        Some("node"),
                        Some("1"),
                        Some("1"),
                        None,
                        None
                    ) | (
                        Some("wrapper"),
                        None,
                        Some("self"),
                        Some("parTrans"),
                        Some("1"),
                        Some("0"),
                        None,
                        None
                    ) | (
                        Some("accentRepeat1" | "accentRepeat2" | "accentRepeat3"),
                        None,
                        Some("self"),
                        None,
                        None,
                        None,
                        None,
                        None
                    ) | (
                        None,
                        Some("accentRepeat1" | "accentRepeat2" | "accentRepeat3"),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None
                    )
                )
        }
        DiagramLayoutFamily::Matrix => false,
        DiagramLayoutFamily::Pyramid => matches!(
            values,
            (None, None, Some("ch"), Some("node"), None, None, None, None)
        ),
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    if !valid {
        return Err(format!(
            "unsupported SmartArt `{}` forEach selector {values:?}",
            family_name(family)
        ));
    }
    Ok(())
}

fn validate_presentation_of_semantics(
    family: &DiagramLayoutFamily,
    owner: Option<&str>,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    let values = (
        attribute(&instruction.attributes, "axis"),
        attribute(&instruction.attributes, "ptType"),
        attribute(&instruction.attributes, "st"),
        attribute(&instruction.attributes, "cnt"),
        attribute(&instruction.attributes, "step"),
    );
    let valid = match family {
        DiagramLayoutFamily::List => matches!(
            values,
            (None, None, None, None, None)
                | (Some("des"), Some("node"), None, None, None)
                | (Some("self"), None, None, None, None)
                | (Some("self"), Some("node"), None, None, None)
        ),
        DiagramLayoutFamily::Hierarchy => matches!(
            values,
            (None, None, None, None, None) | (Some("self"), None, None, None, None)
        ),
        DiagramLayoutFamily::Relationship => matches!(
            values,
            (None, None, None, None, None)
                | (Some("self"), Some("node"), None, None, None)
                | (Some("self"), Some("node"), Some("1"), Some("0"), None)
        ),
        DiagramLayoutFamily::Matrix => matches!(
            values,
            (None, None, None, None, None)
                | (
                    Some("ch ch desOrSelf"),
                    Some("node node node"),
                    Some("1 1 1" | "1 2 1" | "1 3 1" | "1 4 1"),
                    Some("1 1 0"),
                    None
                )
                | (Some("ch"), Some("node"), Some("1"), Some("1"), None)
        ),
        DiagramLayoutFamily::Pyramid => matches!(
            values,
            (None, None, None, None, None)
                | (Some("des"), Some("node"), None, None, None)
                | (Some("self"), None, None, None, None)
        ),
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    let owner = owner.unwrap_or("#none");
    let valid_owner = match family {
        DiagramLayoutFamily::List => match owner {
            "linear" | "parentLin" | "negativeSpace" | "spaceBetweenRectangles" => {
                values == (None, None, None, None, None)
            }
            "parentLeftMargin" => values == (Some("self"), None, None, None, None),
            "parentText" => values == (Some("self"), Some("node"), None, None, None),
            "childText" => values == (Some("des"), Some("node"), None, None, None),
            _ => false,
        },
        DiagramLayoutFamily::Hierarchy => match owner {
            "text" | "text2" | "text3" | "text4" | "#anonymous" => {
                values == (Some("self"), None, None, None, None)
            }
            _ => values == (None, None, None, None, None),
        },
        DiagramLayoutFamily::Relationship => match owner {
            "Parent" => values == (Some("self"), Some("node"), Some("1"), Some("0"), None),
            "Child1" | "Child2" | "Child3" | "Child4" | "Child5" => {
                values == (Some("self"), Some("node"), None, None, None)
            }
            _ => values == (None, None, None, None, None),
        },
        DiagramLayoutFamily::Matrix => match owner {
            "diagram" | "matrix" => values == (None, None, None, None, None),
            "centerTile" => values == (Some("ch"), Some("node"), Some("1"), Some("1"), None),
            "tile1" | "tile1text" | "tile2" | "tile2text" | "tile3" | "tile3text" | "tile4"
            | "tile4text" => matches!(
                values,
                (
                    Some("ch ch desOrSelf"),
                    Some("node node node"),
                    Some("1 1 1" | "1 2 1" | "1 3 1" | "1 4 1"),
                    Some("1 1 0"),
                    None
                )
            ),
            _ => false,
        },
        DiagramLayoutFamily::Pyramid => match owner {
            "#root" | "#anonymous" => values == (None, None, None, None, None),
            "acctBkgd" | "acctTx" => values == (Some("des"), Some("node"), None, None, None),
            "level" | "levelTx" => values == (Some("self"), None, None, None, None),
            _ => false,
        },
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    if !valid || !valid_owner {
        return Err(format!(
            "unsupported SmartArt `{}` presOf selector {values:?} for owner `{owner}`",
            family_name(family),
        ));
    }
    Ok(())
}

fn optional_attribute_is(
    instruction: &DiagramRenderInstruction,
    name: &str,
    allowed: &[&str],
) -> bool {
    attribute(&instruction.attributes, name).is_none_or(|value| allowed.contains(&value))
}

fn attribute_field_signature(instruction: &DiagramRenderInstruction) -> String {
    let mut names = instruction
        .attributes
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.join(",")
}

fn allowed_constraint_field_signature(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
) -> bool {
    let signature = attribute_field_signature(instruction);
    let allowed: &[&str] = match family {
        DiagramLayoutFamily::List => &[
            "fact,for,forName,op,refFor,refForName,refType,type",
            "fact,for,forName,refFor,refForName,refType,type",
            "fact,for,forName,refType,type",
            "fact,for,forName,refType,type",
            "for,forName,refFor,refForName,refType,type",
            "for,forName,refType,type",
            "for,forName,type,val",
            "refType,type",
            "type,val",
        ],
        DiagramLayoutFamily::Hierarchy => &[
            "fact,for,forName,refFor,refForName,refType,type",
            "fact,for,forName,refType,type",
            "fact,for,ptType,refType,type",
            "fact,refFor,refForName,refType,type",
            "fact,refType,type",
            "for,forName,refFor,refForName,refType,type",
            "for,forName,refType,type",
            "for,forName,type,val",
            "for,op,ptType,type,val",
            "type,val",
        ],
        DiagramLayoutFamily::Relationship => &[
            "fact,for,forName,refType,type",
            "fact,refType,type",
            "for,op,ptType,type,val",
        ],
        DiagramLayoutFamily::Matrix => &[
            "fact,for,forName,refFor,refForName,refType,type",
            "fact,for,forName,refType,type",
            "fact,refType,type",
            "for,forName,refFor,refForName,refType,type",
            "for,forName,type,val",
            "for,op,ptType,type,val",
        ],
        DiagramLayoutFamily::Pyramid => &[
            "fact,for,forName,refFor,refForName,refType,type",
            "fact,refType,type",
            "for,forName,op,type",
            "for,forName,type,val",
            "refType,type",
            "type,val",
        ],
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => return false,
    };
    allowed.contains(&signature.as_str())
}

fn validate_constraint_semantics(
    family: &DiagramLayoutFamily,
    owner: Option<&str>,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    if !allowed_constraint_field_signature(family, instruction) {
        return Err(format!(
            "unsupported SmartArt `{}` constraint field set `{}`",
            family_name(family),
            attribute_field_signature(instruction)
        ));
    }
    validate_numeric_attribute(instruction, "fact")?;
    validate_numeric_or_infinite_attribute(instruction, "val")?;
    let constraint_type = attribute(&instruction.attributes, "type")
        .ok_or_else(|| "SmartArt constraint has no type".to_owned())?;
    let valid = match family {
        DiagramLayoutFamily::List => {
            matches!(
                constraint_type,
                "bMarg" | "h" | "lMarg" | "primFontSz" | "rMarg" | "secFontSz" | "tMarg" | "w"
            ) && optional_attribute_is(instruction, "for", &["ch", "des"])
                && optional_attribute_is(
                    instruction,
                    "forName",
                    &[
                        "childText",
                        "negativeSpace",
                        "parentLeftMargin",
                        "parentLin",
                        "parentText",
                        "spaceBetweenRectangles",
                    ],
                )
                && optional_attribute_is(instruction, "op", &["gte", "lte"])
                && attribute(&instruction.attributes, "ptType").is_none()
                && optional_attribute_is(instruction, "refFor", &["ch", "des"])
                && optional_attribute_is(instruction, "refForName", &["childText", "parentText"])
                && optional_attribute_is(instruction, "refType", &["h", "lMarg", "primFontSz", "w"])
        }
        DiagramLayoutFamily::Hierarchy => {
            matches!(
                constraint_type,
                "bMarg"
                    | "begPad"
                    | "bendDist"
                    | "endPad"
                    | "h"
                    | "l"
                    | "lMarg"
                    | "primFontSz"
                    | "rMarg"
                    | "sibSp"
                    | "sp"
                    | "t"
                    | "tMarg"
                    | "w"
            ) && optional_attribute_is(instruction, "for", &["ch", "des"])
                && optional_attribute_is(
                    instruction,
                    "forName",
                    &[
                        "background",
                        "background2",
                        "background3",
                        "background4",
                        "composite",
                        "composite2",
                        "composite3",
                        "composite4",
                        "composite5",
                        "hierChild2",
                        "hierChild3",
                        "hierChild4",
                        "hierChild5",
                        "hierChild6",
                        "hierRoot1",
                        "hierRoot2",
                        "hierRoot3",
                        "hierRoot4",
                        "hierRoot5",
                        "text",
                        "text2",
                        "text3",
                        "text4",
                    ],
                )
                && optional_attribute_is(instruction, "op", &["equ"])
                && optional_attribute_is(instruction, "ptType", &["node", "parTrans"])
                && optional_attribute_is(instruction, "refFor", &["ch", "des"])
                && optional_attribute_is(
                    instruction,
                    "refForName",
                    &[
                        "background",
                        "background2",
                        "background3",
                        "background4",
                        "composite",
                        "hierRoot1",
                        "text",
                        "text2",
                        "text3",
                        "text4",
                    ],
                )
                && optional_attribute_is(
                    instruction,
                    "refType",
                    &["h", "primFontSz", "sibSp", "sp", "w"],
                )
        }
        DiagramLayoutFamily::Relationship => {
            matches!(
                constraint_type,
                "bMarg" | "h" | "l" | "lMarg" | "primFontSz" | "rMarg" | "t" | "tMarg" | "w"
            ) && optional_attribute_is(instruction, "for", &["ch", "des"])
                && optional_attribute_is(
                    instruction,
                    "forName",
                    &[
                        "Accent1", "Accent2", "Accent3", "Accent4", "Accent5", "Accent6",
                        "Accent7", "Accent8", "Accent9", "Accent10", "Accent11", "Accent12",
                        "Accent13", "Accent15", "Accent16", "Child1", "Child2", "Child3", "Child4",
                        "Child5", "Parent",
                    ],
                )
                && optional_attribute_is(instruction, "op", &["equ"])
                && optional_attribute_is(instruction, "ptType", &["node"])
                && attribute(&instruction.attributes, "refFor").is_none()
                && attribute(&instruction.attributes, "refForName").is_none()
                && optional_attribute_is(instruction, "refType", &["h", "primFontSz", "w"])
        }
        DiagramLayoutFamily::Matrix => {
            matches!(
                constraint_type,
                "b" | "bMarg"
                    | "ctrX"
                    | "ctrY"
                    | "h"
                    | "l"
                    | "lMarg"
                    | "primFontSz"
                    | "r"
                    | "rMarg"
                    | "t"
                    | "tMarg"
                    | "w"
            ) && optional_attribute_is(instruction, "for", &["ch", "des"])
                && optional_attribute_is(
                    instruction,
                    "forName",
                    &[
                        "centerTile",
                        "matrix",
                        "tile1",
                        "tile1text",
                        "tile2",
                        "tile2text",
                        "tile3",
                        "tile3text",
                        "tile4",
                        "tile4text",
                    ],
                )
                && optional_attribute_is(instruction, "op", &["equ"])
                && optional_attribute_is(instruction, "ptType", &["node"])
                && optional_attribute_is(instruction, "refFor", &["ch"])
                && optional_attribute_is(
                    instruction,
                    "refForName",
                    &["tile1", "tile2", "tile3", "tile4"],
                )
                && optional_attribute_is(
                    instruction,
                    "refType",
                    &["b", "h", "l", "primFontSz", "r", "t", "w"],
                )
        }
        DiagramLayoutFamily::Pyramid => {
            matches!(
                constraint_type,
                "bMarg"
                    | "ctrX"
                    | "ctrY"
                    | "h"
                    | "lMarg"
                    | "primFontSz"
                    | "pyraAcctRatio"
                    | "rMarg"
                    | "secFontSz"
                    | "tMarg"
                    | "w"
            ) && optional_attribute_is(instruction, "for", &["ch", "des"])
                && optional_attribute_is(
                    instruction,
                    "forName",
                    &["acctBkgd", "acctTx", "level", "levelTx"],
                )
                && optional_attribute_is(instruction, "op", &["equ"])
                && attribute(&instruction.attributes, "ptType").is_none()
                && optional_attribute_is(instruction, "refFor", &["ch"])
                && optional_attribute_is(instruction, "refForName", &["level"])
                && optional_attribute_is(
                    instruction,
                    "refType",
                    &["ctrX", "ctrY", "h", "primFontSz", "secFontSz", "w"],
                )
        }
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    let owner = owner.unwrap_or("#none");
    let valid_owner = match family {
        DiagramLayoutFamily::List => match owner {
            "linear" => true,
            "parentLeftMargin" => constraint_type == "h",
            "parentText" => matches!(constraint_type, "tMarg" | "bMarg"),
            "childText" => constraint_type == "secFontSz",
            _ => false,
        },
        DiagramLayoutFamily::Hierarchy => match owner {
            "hierChild1" => true,
            "hierRoot1" | "hierRoot2" | "hierRoot3" | "hierRoot4" => constraint_type == "bendDist",
            "composite" | "composite2" | "composite3" | "composite4" => {
                matches!(constraint_type, "w" | "h" | "t" | "l")
            }
            "text" | "text2" | "text3" | "text4" => {
                matches!(constraint_type, "tMarg" | "bMarg" | "lMarg" | "rMarg")
            }
            "#anonymous" => matches!(constraint_type, "begPad" | "endPad"),
            _ => false,
        },
        DiagramLayoutFamily::Relationship => match owner {
            "#root" => true,
            "Parent" | "Child1" | "Child2" | "Child3" | "Child4" | "Child5" => {
                matches!(constraint_type, "lMarg" | "rMarg" | "tMarg" | "bMarg")
            }
            _ => false,
        },
        DiagramLayoutFamily::Matrix => match owner {
            "diagram" => matches!(constraint_type, "ctrX" | "ctrY" | "w" | "h" | "primFontSz"),
            "matrix" => matches!(constraint_type, "l" | "t" | "r" | "b" | "w" | "h"),
            "centerTile" => matches!(constraint_type, "tMarg" | "bMarg" | "lMarg" | "rMarg"),
            _ => false,
        },
        DiagramLayoutFamily::Pyramid => match owner {
            "#root" | "#anonymous" => true,
            "acctTx" => matches!(
                constraint_type,
                "secFontSz" | "primFontSz" | "tMarg" | "bMarg" | "lMarg" | "rMarg"
            ),
            "level" => matches!(constraint_type, "h" | "w"),
            "levelTx" => matches!(
                constraint_type,
                "tMarg" | "bMarg" | "lMarg" | "rMarg" | "primFontSz"
            ),
            _ => false,
        },
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    if !valid || !valid_owner {
        return Err(format!(
            "unsupported SmartArt `{}` constraint semantics for owner `{owner}`: {instruction:?}",
            family_name(family),
        ));
    }
    Ok(())
}

fn validate_rule_semantics(
    family: &DiagramLayoutFamily,
    owner: Option<&str>,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    validate_numeric_attribute(instruction, "fact")?;
    validate_numeric_or_infinite_attribute(instruction, "val")?;
    validate_numeric_or_infinite_attribute(instruction, "max")?;
    let rule_type = attribute(&instruction.attributes, "type")
        .ok_or_else(|| "SmartArt rule has no type".to_owned())?;
    let owner = owner.unwrap_or("#none");
    let values = (
        rule_type,
        attribute(&instruction.attributes, "for"),
        attribute(&instruction.attributes, "forName"),
        attribute(&instruction.attributes, "val"),
        attribute(&instruction.attributes, "fact"),
        attribute(&instruction.attributes, "max"),
        attribute(&instruction.attributes, "op"),
        attribute_field_signature(instruction),
    );
    let valid = match family {
        DiagramLayoutFamily::List => {
            (owner == "linear"
                && values.0 == "primFontSz"
                && values.1 == Some("des")
                && values.2 == Some("parentText")
                && values.3 == Some("5")
                && values.4.is_none()
                && values.5.is_none()
                && values.6.is_none()
                && values.7 == "for,forName,type,val")
                || (owner == "childText"
                    && values.0 == "h"
                    && values.1.is_none()
                    && values.2.is_none()
                    && values.3 == Some("INF")
                    && values.4.is_none()
                    && values.5.is_none()
                    && values.6.is_none()
                    && values.7 == "type,val")
        }
        DiagramLayoutFamily::Hierarchy
        | DiagramLayoutFamily::Relationship
        | DiagramLayoutFamily::Matrix => {
            matches!(
                owner,
                "text"
                    | "text2"
                    | "text3"
                    | "text4"
                    | "Parent"
                    | "Child1"
                    | "Child2"
                    | "Child3"
                    | "Child4"
                    | "Child5"
                    | "tile1text"
                    | "tile2text"
                    | "tile3text"
                    | "tile4text"
                    | "centerTile"
            ) && values
                == (
                    "primFontSz",
                    None,
                    None,
                    Some("5"),
                    None,
                    None,
                    None,
                    "type,val".to_owned(),
                )
        }
        DiagramLayoutFamily::Pyramid => {
            matches!(
                (owner, rule_type),
                ("acctTx", "secFontSz") | ("levelTx", "primFontSz")
            ) && values.1.is_none()
                && values.2.is_none()
                && values.3 == Some("5")
                && values.4.is_none()
                && values.5.is_none()
                && values.6.is_none()
                && values.7 == "type,val"
        }
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    if !valid {
        return Err(format!(
            "unsupported SmartArt `{}` rule semantics for owner `{owner}`: {instruction:?}",
            family_name(family),
        ));
    }
    Ok(())
}

fn validate_variable_semantics(
    family: &DiagramLayoutFamily,
    owner: Option<&str>,
    name: &str,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    let value = attribute(&instruction.attributes, "val")
        .ok_or_else(|| format!("SmartArt variable `{name}` has no value"))?;
    let valid = match family {
        DiagramLayoutFamily::List => matches!(
            (name, value),
            ("animLvl", "lvl")
                | ("bulletEnabled", "true")
                | ("chMax", "0")
                | ("dir", "norm")
                | ("resizeHandles", "exact")
        ),
        DiagramLayoutFamily::Hierarchy => matches!(
            (name, value),
            ("animLvl", "lvl")
                | ("animOne", "branch")
                | ("chPref", "1" | "3")
                | ("dir", "norm")
                | ("resizeHandles", "rel")
        ),
        DiagramLayoutFamily::Relationship => matches!(
            (name, value),
            ("chMax", "0" | "1" | "5") | ("chPref", "0" | "1" | "5")
        ),
        DiagramLayoutFamily::Matrix => matches!(
            (name, value),
            ("animLvl", "ctr")
                | ("bulletEnabled", "true")
                | ("chMax", "0" | "1")
                | ("chPref", "0")
                | ("dir", "norm")
                | ("resizeHandles", "exact")
        ),
        DiagramLayoutFamily::Pyramid => matches!(
            (name, value),
            ("animLvl", "lvl")
                | ("bulletEnabled", "true")
                | ("chMax", "1")
                | ("dir", "norm")
                | ("resizeHandles", "exact")
        ),
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    let owner = owner.unwrap_or("#none");
    let valid_owner = match family {
        DiagramLayoutFamily::List => match owner {
            "linear" => matches!(name, "dir" | "animLvl" | "resizeHandles"),
            "parentText" => matches!(name, "chMax" | "bulletEnabled"),
            "childText" => name == "bulletEnabled",
            _ => false,
        },
        DiagramLayoutFamily::Hierarchy => match owner {
            "hierChild1" => matches!(
                name,
                "dir" | "animLvl" | "animOne" | "chPref" | "resizeHandles"
            ),
            "text" | "text2" | "text3" | "text4" => name == "chPref",
            _ => false,
        },
        DiagramLayoutFamily::Relationship => match owner {
            "#root" => matches!(name, "chMax" | "chPref"),
            "Parent" | "Child1" | "Child2" | "Child3" | "Child4" | "Child5" => {
                matches!(name, "chMax" | "chPref")
            }
            _ => false,
        },
        DiagramLayoutFamily::Matrix => match owner {
            "diagram" => matches!(name, "dir" | "animLvl" | "resizeHandles" | "chMax"),
            "tile1text" | "tile2text" | "tile3text" | "tile4text" => {
                matches!(name, "bulletEnabled" | "chMax" | "chPref")
            }
            "centerTile" => matches!(name, "chMax" | "chPref"),
            _ => false,
        },
        DiagramLayoutFamily::Pyramid => match owner {
            "#root" => matches!(name, "dir" | "animLvl" | "resizeHandles"),
            "acctTx" => name == "bulletEnabled",
            "level" | "levelTx" => matches!(name, "bulletEnabled" | "chMax"),
            _ => false,
        },
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    };
    if !valid || !valid_owner {
        return Err(format!(
            "unsupported SmartArt `{}` variable `{name}` value `{value}` for owner `{owner}`",
            family_name(family),
        ));
    }
    Ok(())
}

fn validate_adjustment_semantics(
    family: &DiagramLayoutFamily,
    owner: Option<&str>,
    instruction: &DiagramRenderInstruction,
) -> Result<(), String> {
    let index = attribute(&instruction.attributes, "idx")
        .ok_or_else(|| "SmartArt adjustment has no index".to_owned())?;
    let value = attribute(&instruction.attributes, "val")
        .ok_or_else(|| "SmartArt adjustment has no value".to_owned())?;
    let owner = owner.unwrap_or("#none");
    let valid = matches!(
        (family, index, value),
        (DiagramLayoutFamily::Hierarchy, "1", "0.1") | (DiagramLayoutFamily::Matrix, "1", "0.2")
    ) && match family {
        DiagramLayoutFamily::Hierarchy => matches!(
            owner,
            "background"
                | "background2"
                | "background3"
                | "background4"
                | "text"
                | "text2"
                | "text3"
                | "text4"
        ),
        DiagramLayoutFamily::Matrix => owner == "tile1text",
        _ => false,
    };
    if !valid {
        return Err(format!(
            "unsupported SmartArt `{}` adjustment index `{index}` value `{value}` for owner `{owner}`",
            family_name(family),
        ));
    }
    Ok(())
}

fn allowed_shape_semantics(
    family: &DiagramLayoutFamily,
    instruction: &DiagramRenderInstruction,
) -> bool {
    let shape_type = attribute(&instruction.attributes, "type");
    let hide_geometry = attribute(&instruction.attributes, "hideGeom");
    let rotation = attribute(&instruction.attributes, "rot");
    let z_order = attribute(&instruction.attributes, "zOrderOff");
    if attribute(&instruction.attributes, "blip").is_some()
        || attribute(&instruction.attributes, "lkTxEntry").is_some()
    {
        return false;
    }
    match family {
        DiagramLayoutFamily::List => matches!(
            (shape_type, hide_geometry, rotation, z_order),
            (None, None, None, None)
                | (Some("rect"), Some("true"), None, None)
                | (Some("rect"), None, None, Some("-2"))
                | (Some("roundRect"), None, None, None)
        ),
        DiagramLayoutFamily::Hierarchy => matches!(
            (shape_type, hide_geometry, rotation, z_order),
            (None, None, None, None)
                | (Some("conn"), None, None, Some("-999"))
                | (Some("roundRect"), None, None, None)
        ),
        DiagramLayoutFamily::Relationship => matches!(
            (shape_type, hide_geometry, rotation, z_order),
            (None, None, None, None) | (Some("ellipse"), None, None, None)
        ),
        DiagramLayoutFamily::Matrix => matches!(
            (shape_type, hide_geometry, rotation, z_order),
            (None, None, None, None)
                | (
                    Some("rect"),
                    Some("true"),
                    None | Some("90" | "180" | "270"),
                    None
                )
                | (
                    Some("round1Rect"),
                    None,
                    None | Some("90" | "180" | "270"),
                    None
                )
                | (Some("roundRect"), None, None, None)
        ),
        DiagramLayoutFamily::Pyramid => matches!(
            (shape_type, hide_geometry, rotation, z_order),
            (None, None, None, None)
                | (
                    Some("nonIsoscelesTrapezoid"),
                    None | Some("true"),
                    None,
                    None
                )
                | (Some("rect"), Some("true"), None, None)
                | (Some("trapezoid"), None, None, None)
        ),
        DiagramLayoutFamily::Cycle | DiagramLayoutFamily::Unsupported(_) => false,
    }
}

fn allowed_algorithm_parameters(
    family: &DiagramLayoutFamily,
    algorithm: &str,
    parameters: &[(&str, &str)],
) -> bool {
    let exact = |expected: &[(&str, &str)]| parameters == expected;
    match (family, algorithm) {
        (DiagramLayoutFamily::List, "lin") => {
            exact(&[
                ("linDir", "fromT"),
                ("vertAlign", "mid"),
                ("horzAlign", "l"),
                ("nodeHorzAlign", "l"),
            ]) || exact(&[
                ("linDir", "fromT"),
                ("vertAlign", "mid"),
                ("horzAlign", "r"),
                ("nodeHorzAlign", "r"),
            ]) || exact(&[
                ("linDir", "fromL"),
                ("horzAlign", "l"),
                ("nodeHorzAlign", "l"),
            ]) || exact(&[
                ("linDir", "fromR"),
                ("horzAlign", "r"),
                ("nodeHorzAlign", "r"),
            ])
        }
        (DiagramLayoutFamily::List, "sp") => parameters.is_empty(),
        (DiagramLayoutFamily::List, "tx") => {
            exact(&[("parTxLTRAlign", "l"), ("parTxRTLAlign", "l")])
                || exact(&[("parTxLTRAlign", "r"), ("parTxRTLAlign", "r")])
                || exact(&[("stBulletLvl", "1")])
        }
        (DiagramLayoutFamily::Hierarchy, "composite" | "hierRoot" | "sp" | "tx") => {
            parameters.is_empty()
        }
        (DiagramLayoutFamily::Hierarchy, "hierChild") => {
            exact(&[("linDir", "fromL")]) || exact(&[("linDir", "fromR")])
        }
        (DiagramLayoutFamily::Hierarchy, "conn") => {
            exact(&[
                ("dim", "1D"),
                ("endSty", "noArr"),
                ("connRout", "bend"),
                ("bendPt", "end"),
                ("begPts", "bCtr"),
                ("endPts", "tCtr"),
                ("srcNode", "background"),
                ("dstNode", "background2"),
            ]) || exact(&[
                ("dim", "1D"),
                ("endSty", "noArr"),
                ("connRout", "bend"),
                ("bendPt", "end"),
                ("begPts", "bCtr"),
                ("endPts", "tCtr"),
                ("srcNode", "background2"),
                ("dstNode", "background3"),
            ]) || exact(&[
                ("dim", "1D"),
                ("endSty", "noArr"),
                ("connRout", "bend"),
                ("bendPt", "end"),
                ("begPts", "bCtr"),
                ("endPts", "tCtr"),
                ("srcNode", "background3"),
                ("dstNode", "background4"),
            ]) || exact(&[
                ("dim", "1D"),
                ("endSty", "noArr"),
                ("connRout", "bend"),
                ("bendPt", "end"),
                ("begPts", "bCtr"),
                ("endPts", "tCtr"),
                ("srcNode", "background4"),
                ("dstNode", "background4"),
            ])
        }
        (DiagramLayoutFamily::Relationship, "composite") => {
            parameters.len() == 1
                && parameters[0].0 == "ar"
                && matches!(
                    parameters[0].1,
                    "0.98" | "1.1477" | "1.2476" | "1.3749" | "1.592" | "1.7557"
                )
        }
        (DiagramLayoutFamily::Relationship, "sp") => parameters.is_empty(),
        (DiagramLayoutFamily::Relationship, "tx") => {
            parameters.is_empty() || exact(&[("shpTxLTRAlignCh", "ctr"), ("txAnchorVertCh", "mid")])
        }
        (DiagramLayoutFamily::Matrix, "composite" | "sp") => parameters.is_empty(),
        (DiagramLayoutFamily::Matrix, "tx") => {
            parameters.is_empty()
                || exact(&[
                    ("txAnchorVert", "t"),
                    ("parTxLTRAlign", "l"),
                    ("parTxRTLAlign", "r"),
                ])
        }
        (DiagramLayoutFamily::Pyramid, "pyra") => {
            exact(&[
                ("linDir", "fromB"),
                ("txDir", "fromT"),
                ("pyraAcctPos", "aft"),
                ("pyraAcctTxMar", "step"),
                ("pyraAcctBkgdNode", "acctBkgd"),
                ("pyraAcctTxNode", "acctTx"),
                ("pyraLvlNode", "level"),
            ]) || exact(&[
                ("linDir", "fromB"),
                ("txDir", "fromT"),
                ("pyraAcctPos", "bef"),
                ("pyraAcctTxMar", "step"),
                ("pyraAcctBkgdNode", "acctBkgd"),
                ("pyraAcctTxNode", "acctTx"),
                ("pyraLvlNode", "level"),
            ])
        }
        (DiagramLayoutFamily::Pyramid, "composite") => exact(&[("horzAlign", "none")]),
        (DiagramLayoutFamily::Pyramid, "sp") => parameters.is_empty(),
        (DiagramLayoutFamily::Pyramid, "tx") => {
            parameters.is_empty() || exact(&[("stBulletLvl", "1"), ("txAnchorVertCh", "mid")])
        }
        _ => false,
    }
}

fn evaluate_program(
    root: &DiagramRenderInstruction,
    node_count: usize,
) -> Result<ProgramEvaluation, String> {
    let mut variables = HashMap::new();
    let mut budget = ProgramBudget::default();
    collect_variables(root, &mut variables, 0, &mut budget)?;
    let mut evaluation = ProgramEvaluation::default();
    evaluate_instruction(
        root,
        node_count,
        &variables,
        0,
        &mut evaluation,
        &mut budget,
    )?;
    Ok(evaluation)
}

fn collect_variables<'a>(
    instruction: &'a DiagramRenderInstruction,
    variables: &mut HashMap<&'a str, &'a str>,
    depth: usize,
    budget: &mut ProgramBudget,
) -> Result<(), String> {
    if depth > MAX_GRAPH_DEPTH {
        return Err(format!(
            "SmartArt variable collection exceeds depth bound {MAX_GRAPH_DEPTH}"
        ));
    }
    budget.charge("variable collection")?;
    if let DiagramRenderInstructionKind::Variable(name) = &instruction.kind
        && let Some(value) = attribute(&instruction.attributes, "val")
    {
        variables.insert(name, value);
    }
    for child in &instruction.children {
        collect_variables(child, variables, depth + 1, budget)?;
    }
    Ok(())
}

fn evaluate_instruction(
    instruction: &DiagramRenderInstruction,
    node_count: usize,
    variables: &HashMap<&str, &str>,
    depth: usize,
    evaluation: &mut ProgramEvaluation,
    budget: &mut ProgramBudget,
) -> Result<(), String> {
    if depth > MAX_GRAPH_DEPTH {
        return Err(format!(
            "SmartArt evaluation exceeds depth bound {MAX_GRAPH_DEPTH}"
        ));
    }
    budget.charge("instruction evaluation")?;
    match &instruction.kind {
        DiagramRenderInstructionKind::LayoutNode => {
            checked_increment(&mut evaluation.layout_nodes, "layout-node")?
        }
        DiagramRenderInstructionKind::Shape => {
            checked_increment(&mut evaluation.shapes, "generated-shape")?
        }
        DiagramRenderInstructionKind::PresentationOf => checked_increment(
            &mut evaluation.presentation_mappings,
            "presentation-mapping",
        )?,
        DiagramRenderInstructionKind::Algorithm => {
            let kind = attribute(&instruction.attributes, "type")
                .ok_or_else(|| "SmartArt algorithm has no type".to_owned())?;
            if !matches!(
                kind,
                "composite"
                    | "conn"
                    | "cycle"
                    | "hierChild"
                    | "hierRoot"
                    | "lin"
                    | "pyra"
                    | "sp"
                    | "tx"
            ) {
                return Err(format!("unsupported SmartArt algorithm `{kind}`"));
            }
            checked_increment(&mut evaluation.algorithms, "algorithm")?;
        }
        DiagramRenderInstructionKind::Constraint => {
            validate_numeric_attribute(instruction, "fact")?;
            validate_numeric_or_infinite_attribute(instruction, "val")?;
            checked_increment(&mut evaluation.constraints, "constraint")?;
        }
        DiagramRenderInstructionKind::Rule => {
            validate_numeric_attribute(instruction, "fact")?;
            validate_numeric_or_infinite_attribute(instruction, "val")?;
            checked_increment(&mut evaluation.rules, "rule")?;
        }
        DiagramRenderInstructionKind::ForEach => {
            let repeats = foreach_count(instruction, node_count)?;
            if repeats > MAX_NODES {
                return Err(format!("SmartArt iteration exceeds bound {MAX_NODES}"));
            }
            for _ in 0..repeats {
                for child in &instruction.children {
                    evaluate_instruction(
                        child,
                        node_count,
                        variables,
                        depth + 1,
                        evaluation,
                        budget,
                    )?;
                }
            }
            return Ok(());
        }
        DiagramRenderInstructionKind::Choose => {
            for child in &instruction.children {
                match child.kind {
                    DiagramRenderInstructionKind::Condition
                        if {
                            budget.charge("condition evaluation")?;
                            checked_increment(&mut evaluation.conditions, "condition")?;
                            condition_matches(child, node_count, variables)?
                        } =>
                    {
                        for nested in &child.children {
                            evaluate_instruction(
                                nested,
                                node_count,
                                variables,
                                depth + 1,
                                evaluation,
                                budget,
                            )?;
                        }
                        return Ok(());
                    }
                    DiagramRenderInstructionKind::Else => {
                        for nested in &child.children {
                            evaluate_instruction(
                                nested,
                                node_count,
                                variables,
                                depth + 1,
                                evaluation,
                                budget,
                            )?;
                        }
                        return Ok(());
                    }
                    DiagramRenderInstructionKind::Condition => {}
                    _ => return Err("SmartArt choose contains a non-branch child".to_owned()),
                }
            }
            return Err("SmartArt choose has no matching branch".to_owned());
        }
        DiagramRenderInstructionKind::Condition | DiagramRenderInstructionKind::Else => {
            return Err("SmartArt branch instruction occurs outside choose".to_owned());
        }
        DiagramRenderInstructionKind::Unsupported(local) => {
            return Err(format!("unsupported SmartArt instruction `{local}`"));
        }
        _ => {}
    }
    for child in &instruction.children {
        evaluate_instruction(child, node_count, variables, depth + 1, evaluation, budget)?;
    }
    Ok(())
}

fn checked_increment(counter: &mut usize, kind: &str) -> Result<(), String> {
    *counter = counter
        .checked_add(1)
        .ok_or_else(|| format!("SmartArt {kind} counter overflow"))?;
    Ok(())
}

fn validate_numeric_attribute(
    instruction: &DiagramRenderInstruction,
    name: &str,
) -> Result<(), String> {
    if let Some(value) = attribute(&instruction.attributes, name) {
        let parsed = value
            .parse::<f64>()
            .map_err(|_| format!("SmartArt `{name}` is not numeric"))?;
        if !parsed.is_finite() {
            return Err(format!("SmartArt `{name}` is not finite"));
        }
    }
    Ok(())
}

fn validate_numeric_or_infinite_attribute(
    instruction: &DiagramRenderInstruction,
    name: &str,
) -> Result<(), String> {
    if attribute(&instruction.attributes, name) != Some("INF") {
        validate_numeric_attribute(instruction, name)?;
    }
    Ok(())
}

fn foreach_count(
    instruction: &DiagramRenderInstruction,
    node_count: usize,
) -> Result<usize, String> {
    if attribute(&instruction.attributes, "ref").is_some() {
        return Ok(1);
    }
    let axis = attribute(&instruction.attributes, "axis").unwrap_or("self");
    let available = match axis {
        "ch" => node_count,
        "followSib" => node_count.saturating_sub(1),
        "self" | "ch ch" | "root des" | "par ch" => 1,
        _ => return Err(format!("unsupported SmartArt iteration axis `{axis}`")),
    };
    let count = attribute(&instruction.attributes, "cnt")
        .and_then(|value| value.split_whitespace().next())
        .map(str::parse::<usize>)
        .transpose()
        .map_err(|_| "SmartArt iteration count is invalid".to_owned())?
        .unwrap_or(available);
    Ok(count.min(available))
}

fn condition_matches(
    instruction: &DiagramRenderInstruction,
    node_count: usize,
    variables: &HashMap<&str, &str>,
) -> Result<bool, String> {
    let function = attribute(&instruction.attributes, "func")
        .ok_or_else(|| "SmartArt condition has no function".to_owned())?;
    let expected = attribute(&instruction.attributes, "val").unwrap_or_default();
    let ordering = match function {
        "var" => {
            let name = attribute(&instruction.attributes, "arg")
                .ok_or_else(|| "SmartArt variable condition has no argument".to_owned())?;
            return compare_condition(
                variables.get(name).copied().unwrap_or_default(),
                expected,
                instruction,
            );
        }
        "cnt" => match attribute(&instruction.attributes, "axis").unwrap_or("ch") {
            "ch ch" => node_count.saturating_sub(1) as i64,
            "ch" | "par ch" => node_count as i64,
            "self" => 1,
            axis => return Err(format!("unsupported SmartArt count axis `{axis}`")),
        },
        "depth" | "pos" => 1,
        "maxDepth" => 1,
        _ => {
            return Err(format!(
                "unsupported SmartArt condition function `{function}`"
            ));
        }
    };
    let expected = expected
        .parse::<i64>()
        .map_err(|_| "SmartArt condition value is invalid".to_owned())?;
    compare_ordering(ordering, expected, instruction)
}

fn compare_condition(
    actual: &str,
    expected: &str,
    instruction: &DiagramRenderInstruction,
) -> Result<bool, String> {
    match attribute(&instruction.attributes, "op").unwrap_or("equ") {
        "equ" => Ok(actual == expected),
        op => Err(format!(
            "unsupported SmartArt string condition operator `{op}`"
        )),
    }
}

fn compare_ordering(
    actual: i64,
    expected: i64,
    instruction: &DiagramRenderInstruction,
) -> Result<bool, String> {
    match attribute(&instruction.attributes, "op").unwrap_or("equ") {
        "equ" => Ok(actual == expected),
        "gt" => Ok(actual > expected),
        "gte" => Ok(actual >= expected),
        "lt" => Ok(actual < expected),
        "lte" => Ok(actual <= expected),
        op => Err(format!(
            "unsupported SmartArt numeric condition operator `{op}`"
        )),
    }
}

fn require_layout_node<'a>(
    root: &'a DiagramRenderInstruction,
    name: &str,
    style_label: Option<&str>,
) -> Result<&'a DiagramRenderInstruction, String> {
    let mut stack = vec![root];
    while let Some(instruction) = stack.pop() {
        if instruction.kind == DiagramRenderInstructionKind::LayoutNode
            && attribute(&instruction.attributes, "name") == Some(name)
        {
            if let Some(expected) = style_label
                && attribute(&instruction.attributes, "styleLbl") != Some(expected)
            {
                return Err(format!(
                    "SmartArt layout node `{name}` has unexpected style ownership"
                ));
            }
            if !instruction
                .children
                .iter()
                .any(|child| matches!(child.kind, DiagramRenderInstructionKind::Shape))
            {
                return Err(format!("SmartArt layout node `{name}` has no direct shape"));
            }
            return Ok(instruction);
        }
        stack.extend(instruction.children.iter().rev());
    }
    Err(format!("SmartArt layout is missing owned node `{name}`"))
}

fn presentation_graph<'a>(
    data: &'a CT_DiagramData,
    _layout: &'a CT_DiagramLayoutDefinition,
) -> Result<(Vec<RenderNode<'a>>, Vec<GraphEdge<'a>>), String> {
    let mut all_ids = HashMap::with_capacity(data.points().len());
    for point in data.points() {
        if point.model_id.is_empty() || all_ids.insert(point.model_id.as_str(), point).is_some() {
            return Err(format!(
                "duplicate or empty SmartArt model id `{}`",
                point.model_id
            ));
        }
    }
    let mut connection_ids = HashSet::new();
    for connection in data.connections() {
        if connection.model_id.is_empty() || !connection_ids.insert(connection.model_id.as_str()) {
            return Err(format!(
                "duplicate or empty SmartArt connection id `{}`",
                connection.model_id
            ));
        }
        if matches!(
            connection.kind,
            DiagramConnectionKind::ParentOf
                | DiagramConnectionKind::PresentationOf
                | DiagramConnectionKind::PresentationParentOf
        ) && (!all_ids.contains_key(connection.source_id.as_str())
            || !all_ids.contains_key(connection.destination_id.as_str()))
        {
            return Err(format!(
                "SmartArt connection `{}` references missing endpoint `{}` -> `{}`",
                connection.model_id, connection.source_id, connection.destination_id
            ));
        }
    }
    let presentation = data
        .points()
        .iter()
        .filter(|point| point.kind == DiagramPointKind::Presentation)
        .collect::<Vec<_>>();
    if presentation.is_empty() {
        let mut nodes = data
            .points()
            .iter()
            .filter(|point| {
                matches!(
                    point.kind,
                    DiagramPointKind::Node | DiagramPointKind::Assistant
                )
            })
            .map(|point| RenderNode {
                id: point.model_id.as_str(),
                text: point.text.as_ref(),
            })
            .collect::<Vec<_>>();
        let node_ids = nodes.iter().map(|node| node.id).collect::<HashSet<_>>();
        let mut edges = data
            .connections()
            .iter()
            .filter(|connection| connection.kind == DiagramConnectionKind::ParentOf)
            .filter_map(|connection| {
                (node_ids.contains(connection.source_id.as_str())
                    && node_ids.contains(connection.destination_id.as_str()))
                .then_some(GraphEdge {
                    source: connection.source_id.as_str(),
                    destination: connection.destination_id.as_str(),
                    source_order: connection.source_order,
                    destination_order: connection.destination_order,
                })
            })
            .collect::<Vec<_>>();
        let mut semantic_edges = HashSet::new();
        for edge in &edges {
            if !semantic_edges.insert((edge.source, edge.destination)) {
                return Err(format!(
                    "duplicate SmartArt data topology `{}` -> `{}`",
                    edge.source, edge.destination
                ));
            }
        }
        normalize_presentation_graph(&mut nodes, &mut edges, &HashMap::new())?;
        return Ok((nodes, edges));
    }

    let presentation_ids = presentation
        .iter()
        .map(|point| point.model_id.as_str())
        .collect::<HashSet<_>>();
    let mut owner_by_presentation = HashMap::new();
    let mut presentation_by_data = HashMap::new();
    for connection in data
        .connections()
        .iter()
        .filter(|connection| connection.kind == DiagramConnectionKind::PresentationOf)
    {
        let (data_id, presentation_id, owner_order, presentation_order) = match (
            presentation_ids.contains(connection.source_id.as_str()),
            presentation_ids.contains(connection.destination_id.as_str()),
        ) {
            (false, true) => (
                connection.source_id.as_str(),
                connection.destination_id.as_str(),
                connection.source_order,
                connection.destination_order,
            ),
            (true, false) => (
                connection.destination_id.as_str(),
                connection.source_id.as_str(),
                connection.destination_order,
                connection.source_order,
            ),
            _ => {
                return Err(format!(
                    "SmartArt presentation ownership `{}` has invalid endpoint roles",
                    connection.model_id
                ));
            }
        };
        let owner = all_ids[data_id];
        if !matches!(
            owner.kind,
            DiagramPointKind::Node | DiagramPointKind::Assistant
        ) {
            return Err(format!(
                "SmartArt presentation node `{presentation_id}` has non-data owner `{data_id}`"
            ));
        }
        if owner_by_presentation
            .insert(presentation_id, (owner, presentation_order, owner_order))
            .is_some()
            || presentation_by_data
                .insert(data_id, presentation_id)
                .is_some()
        {
            return Err(format!(
                "ambiguous SmartArt presentation ownership for `{presentation_id}`"
            ));
        }
    }
    for point in data.points().iter().filter(|point| {
        matches!(
            point.kind,
            DiagramPointKind::Node | DiagramPointKind::Assistant
        )
    }) {
        if !presentation_by_data.contains_key(point.model_id.as_str()) {
            return Err(format!(
                "SmartArt data node `{}` has no presentation mapping",
                point.model_id
            ));
        }
    }
    let mut nodes = presentation
        .iter()
        .map(|point| {
            let (owner, _, _) = owner_by_presentation
                .get(point.model_id.as_str())
                .ok_or_else(|| {
                    format!(
                        "SmartArt presentation node `{}` has no data owner",
                        point.model_id
                    )
                })?;
            Ok(RenderNode {
                id: point.model_id.as_str(),
                text: owner.text.as_ref(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut edges = Vec::new();
    for connection in data.connections() {
        match connection.kind {
            DiagramConnectionKind::PresentationParentOf => {
                if !presentation_ids.contains(connection.source_id.as_str())
                    || !presentation_ids.contains(connection.destination_id.as_str())
                {
                    return Err(format!(
                        "SmartArt presentation parent `{}` does not join presentation nodes",
                        connection.model_id
                    ));
                }
                edges.push(GraphEdge {
                    source: connection.source_id.as_str(),
                    destination: connection.destination_id.as_str(),
                    source_order: connection.source_order,
                    destination_order: connection.destination_order,
                });
            }
            DiagramConnectionKind::ParentOf => {
                if all_ids.get(connection.source_id.as_str()).is_some_and(
                    |point| matches!(&point.kind, DiagramPointKind::Other(kind) if kind == "doc"),
                ) {
                    continue;
                }
                let source = presentation_by_data
                    .get(connection.source_id.as_str())
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "SmartArt data parent `{}` has no presentation source",
                            connection.model_id
                        )
                    })?;
                let destination = presentation_by_data
                    .get(connection.destination_id.as_str())
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "SmartArt data parent `{}` has no presentation destination",
                            connection.model_id
                        )
                    })?;
                edges.push(GraphEdge {
                    source,
                    destination,
                    source_order: connection.source_order,
                    destination_order: connection.destination_order,
                });
            }
            DiagramConnectionKind::PresentationOf | DiagramConnectionKind::Other(_) => {}
        }
    }
    let mut semantic_edges = HashSet::new();
    for edge in &edges {
        if !semantic_edges.insert((edge.source, edge.destination)) {
            return Err(format!(
                "duplicate SmartArt presentation topology `{}` -> `{}`",
                edge.source, edge.destination
            ));
        }
    }
    normalize_presentation_graph(&mut nodes, &mut edges, &owner_by_presentation)?;
    Ok((nodes, edges))
}

fn normalize_presentation_graph<'a>(
    nodes: &mut Vec<RenderNode<'a>>,
    edges: &mut Vec<GraphEdge<'a>>,
    ownership: &HashMap<&'a str, (&'a rpptx_oxml::diagram::DiagramPoint, u32, u32)>,
) -> Result<(), String> {
    let mut indegree = nodes
        .iter()
        .map(|node| (node.id, 0usize))
        .collect::<HashMap<_, _>>();
    let mut outgoing: HashMap<&str, Vec<GraphEdge<'a>>> = HashMap::new();
    let mut incoming_order = HashMap::new();
    for edge in edges.iter().copied() {
        *indegree
            .get_mut(edge.destination)
            .ok_or_else(|| format!("missing topology destination `{}`", edge.destination))? += 1;
        outgoing.entry(edge.source).or_default().push(edge);
        incoming_order
            .entry(edge.destination)
            .and_modify(|current: &mut (u32, u32)| {
                *current = (*current).min((edge.source_order, edge.destination_order));
            })
            .or_insert((edge.source_order, edge.destination_order));
    }
    for children in outgoing.values_mut() {
        children.sort_by_key(|edge| (edge.source_order, edge.destination_order, edge.destination));
    }
    let order_key = |id: &'a str| {
        let (presentation_order, owner_order) = ownership
            .get(id)
            .map(|(_, presentation_order, owner_order)| (*presentation_order, *owner_order))
            .unwrap_or((u32::MAX, u32::MAX));
        let (source_order, destination_order) = incoming_order
            .get(id)
            .copied()
            .unwrap_or((u32::MAX, u32::MAX));
        (
            source_order,
            destination_order,
            presentation_order,
            owner_order,
            id,
        )
    };
    let mut ready = indegree
        .iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(*id))
        .collect::<Vec<_>>();
    let mut normalized = Vec::with_capacity(nodes.len());
    while !ready.is_empty() {
        ready.sort_by_key(|id| order_key(id));
        let id = ready.remove(0);
        normalized.push(id);
        for edge in outgoing.get(id).into_iter().flatten() {
            let degree = indegree
                .get_mut(edge.destination)
                .expect("validated presentation endpoint");
            *degree -= 1;
            if *degree == 0 {
                ready.push(edge.destination);
            }
        }
    }
    if normalized.len() != nodes.len() {
        return Err("SmartArt presentation topology contains a cycle".to_owned());
    }
    let by_id = nodes
        .drain(..)
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    for id in normalized {
        nodes.push(by_id.get(id).expect("normalized presentation id").clone());
    }
    let node_index = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id, index))
        .collect::<HashMap<_, _>>();
    edges.sort_by_key(|edge| {
        (
            node_index[edge.source],
            edge.source_order,
            edge.destination_order,
            node_index[edge.destination],
        )
    });
    Ok(())
}

fn authentic_list(
    nodes: &[RenderNode<'_>],
    bounds: Rect,
    program: &DiagramRenderInstruction,
) -> Result<PreparedDiagram, String> {
    if nodes.len() != 3 {
        return Err("authentic SmartArt list requires exactly three data nodes".to_owned());
    }
    let left_factor = constraint_factor(program, "w", "parentLeftMargin")?;
    let width_factor = constraint_factor(program, "w", "parentText")?;
    if !(0.0..=1.0).contains(&left_factor)
        || !(0.0..=1.0).contains(&width_factor)
        || left_factor + width_factor > 1.0
    {
        return Err("SmartArt list width constraints exceed the parent bounds".to_owned());
    }
    let text_x = 576.0 * left_factor;
    let text_width = 576.0 * width_factor;
    let mut shapes = Vec::with_capacity(6);
    for (index, node) in nodes.iter().enumerate() {
        let y = 44.590 + index as f64 * 121.433;
        shapes.push(prepared_shape(
            relative_rect(bounds, 0.0, y, 576.0, 67.465),
            "rect",
            None,
            "conFgAcc1",
            index,
        ));
        shapes.push(prepared_shape(
            relative_rect(bounds, text_x, y - 39.516, text_width, 79.032),
            "roundRect",
            diagram_text(node.text, 3_400, false, [15.3, 3.0, 15.3, 1.0], None)?,
            "node1",
            index,
        ));
    }
    Ok(PreparedDiagram {
        shapes,
        connectors: Vec::new(),
    })
}

fn constraint_factor(
    root: &DiagramRenderInstruction,
    constraint_type: &str,
    for_name: &str,
) -> Result<f64, String> {
    let mut stack = vec![root];
    while let Some(instruction) = stack.pop() {
        if instruction.kind == DiagramRenderInstructionKind::Constraint
            && attribute(&instruction.attributes, "type") == Some(constraint_type)
            && (if for_name.is_empty() {
                attribute(&instruction.attributes, "forName").is_none()
            } else {
                attribute(&instruction.attributes, "forName") == Some(for_name)
            })
            && let Some(value) = attribute(&instruction.attributes, "fact")
        {
            let value = value.parse::<f64>().map_err(|_| {
                format!("SmartArt constraint `{constraint_type}` for `{for_name}` is invalid")
            })?;
            if value.is_finite() {
                return Ok(value);
            }
        }
        stack.extend(instruction.children.iter().rev());
    }
    Err(format!(
        "SmartArt program is missing `{constraint_type}` factor for `{for_name}`"
    ))
}

fn global_constraint_factor(
    root: &DiagramRenderInstruction,
    constraint_type: &str,
) -> Result<f64, String> {
    let mut stack = vec![root];
    while let Some(instruction) = stack.pop() {
        if instruction.kind == DiagramRenderInstructionKind::Constraint
            && attribute(&instruction.attributes, "type") == Some(constraint_type)
            && attribute(&instruction.attributes, "forName").is_none()
            && let Some(value) = attribute(&instruction.attributes, "fact")
        {
            return value
                .parse::<f64>()
                .map_err(|_| format!("SmartArt global constraint `{constraint_type}` is invalid"));
        }
        stack.extend(instruction.children.iter().rev());
    }
    Err(format!(
        "SmartArt program is missing global `{constraint_type}` factor"
    ))
}

fn selected_constraint_number(
    root: &DiagramRenderInstruction,
    node_count: usize,
    constraint_type: &str,
    for_name: &str,
    value_attribute: &str,
) -> Result<f64, String> {
    fn visit(
        instruction: &DiagramRenderInstruction,
        node_count: usize,
        constraint_type: &str,
        for_name: &str,
        value_attribute: &str,
        variables: &HashMap<&str, &str>,
    ) -> Result<Option<f64>, String> {
        if instruction.kind == DiagramRenderInstructionKind::Constraint
            && attribute(&instruction.attributes, "type") == Some(constraint_type)
            && (if for_name.is_empty() {
                attribute(&instruction.attributes, "forName").is_none()
            } else {
                attribute(&instruction.attributes, "forName") == Some(for_name)
            })
            && let Some(value) = attribute(&instruction.attributes, value_attribute)
        {
            let value = value.parse::<f64>().map_err(|_| {
                format!("SmartArt constraint `{constraint_type}` for `{for_name}` is invalid")
            })?;
            if !value.is_finite() {
                return Err(format!(
                    "SmartArt constraint `{constraint_type}` for `{for_name}` is not finite"
                ));
            }
            return Ok(Some(value));
        }
        if instruction.kind == DiagramRenderInstructionKind::Choose {
            for branch in &instruction.children {
                let selected = match branch.kind {
                    DiagramRenderInstructionKind::Condition => {
                        condition_matches(branch, node_count, variables)?
                    }
                    DiagramRenderInstructionKind::Else => true,
                    _ => return Err("SmartArt choose contains a non-branch child".to_owned()),
                };
                if selected {
                    for child in &branch.children {
                        if let Some(value) = visit(
                            child,
                            node_count,
                            constraint_type,
                            for_name,
                            value_attribute,
                            variables,
                        )? {
                            return Ok(Some(value));
                        }
                    }
                    return Ok(None);
                }
            }
            return Ok(None);
        }
        for child in &instruction.children {
            if let Some(value) = visit(
                child,
                node_count,
                constraint_type,
                for_name,
                value_attribute,
                variables,
            )? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    let mut variables = HashMap::new();
    let mut budget = ProgramBudget::default();
    collect_variables(root, &mut variables, 0, &mut budget)?;
    visit(
        root,
        node_count,
        constraint_type,
        for_name,
        value_attribute,
        &variables,
    )?
    .ok_or_else(|| {
        format!(
            "SmartArt program is missing selected `{constraint_type}` {value_attribute} for `{for_name}`"
        )
    })
}

fn selected_parameter_number(
    root: &DiagramRenderInstruction,
    node_count: usize,
    parameter_type: &str,
) -> Result<f64, String> {
    fn visit(
        instruction: &DiagramRenderInstruction,
        node_count: usize,
        parameter_type: &str,
        variables: &HashMap<&str, &str>,
        matches: &mut Vec<f64>,
    ) -> Result<(), String> {
        if instruction.kind == DiagramRenderInstructionKind::Parameter
            && attribute(&instruction.attributes, "type") == Some(parameter_type)
            && let Some(value) = attribute(&instruction.attributes, "val")
        {
            let value = value
                .parse::<f64>()
                .map_err(|_| format!("SmartArt parameter `{parameter_type}` is invalid"));
            matches.push(value?);
            return Ok(());
        }
        if instruction.kind == DiagramRenderInstructionKind::Choose {
            for branch in &instruction.children {
                let selected = match branch.kind {
                    DiagramRenderInstructionKind::Condition => {
                        condition_matches(branch, node_count, variables)?
                    }
                    DiagramRenderInstructionKind::Else => true,
                    _ => return Err("SmartArt choose contains a non-branch child".to_owned()),
                };
                if selected {
                    for child in &branch.children {
                        visit(child, node_count, parameter_type, variables, matches)?;
                    }
                    return Ok(());
                }
            }
            return Ok(());
        }
        for child in &instruction.children {
            visit(child, node_count, parameter_type, variables, matches)?;
        }
        Ok(())
    }

    let mut variables = HashMap::new();
    let mut budget = ProgramBudget::default();
    collect_variables(root, &mut variables, 0, &mut budget)?;
    let mut matches = Vec::with_capacity(1);
    visit(root, node_count, parameter_type, &variables, &mut matches)?;
    match matches.as_slice() {
        [value] => Ok(*value),
        [] => Err(format!(
            "SmartArt program is missing parameter `{parameter_type}`"
        )),
        _ => Err(format!(
            "SmartArt program has duplicate selected parameter `{parameter_type}`"
        )),
    }
}

fn selected_constraint_fraction(
    root: &DiagramRenderInstruction,
    node_count: usize,
    constraint_type: &str,
    for_name: &str,
) -> Result<f64, String> {
    selected_constraint_number(root, node_count, constraint_type, for_name, "fact")
        .or_else(|_| selected_constraint_number(root, node_count, constraint_type, for_name, "val"))
}

fn layout_node_constraint_number(
    root: &DiagramRenderInstruction,
    node_name: &str,
    constraint_type: &str,
    value_attribute: &str,
) -> Result<f64, String> {
    let node = require_layout_node(root, node_name, None)?;
    for child in &node.children {
        if child.kind != DiagramRenderInstructionKind::ConstraintList {
            continue;
        }
        for constraint in &child.children {
            if constraint.kind == DiagramRenderInstructionKind::Constraint
                && attribute(&constraint.attributes, "type") == Some(constraint_type)
                && let Some(value) = attribute(&constraint.attributes, value_attribute)
            {
                return value.parse::<f64>().map_err(|_| {
                    format!(
                        "SmartArt layout node `{node_name}` constraint `{constraint_type}` is invalid"
                    )
                });
            }
        }
    }
    Err(format!(
        "SmartArt layout node `{node_name}` is missing `{constraint_type}` {value_attribute}"
    ))
}

fn authentic_hierarchy(
    nodes: &[RenderNode<'_>],
    edges: &[GraphEdge<'_>],
    bounds: Rect,
    program: &DiagramRenderInstruction,
) -> Result<PreparedDiagram, String> {
    const BRANCH_GEOMETRY: &str = r#"<a:custGeom xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:avLst/><a:gdLst/><a:ahLst/><a:cxnLst/><a:rect l="l" t="t" r="r" b="b"/><a:pathLst><a:path w="100000" h="100000"><a:moveTo><a:pt x="0" y="0"/></a:moveTo><a:lnTo><a:pt x="0" y="68148"/></a:lnTo><a:lnTo><a:pt x="100000" y="68148"/></a:lnTo><a:lnTo><a:pt x="100000" y="100000"/></a:lnTo></a:path></a:pathLst></a:custGeom>"#;
    if nodes.len() != 3 || edges.len() != 2 {
        return Err("authentic SmartArt hierarchy requires one root and two children".to_owned());
    }
    let background_width = constraint_factor(program, "w", "background")?;
    let background_height = constraint_factor(program, "h", "background")?;
    let text_left = constraint_factor(program, "l", "text")?;
    let text_top = constraint_factor(program, "t", "text")?;
    let composite_height = constraint_factor(program, "h", "composite")?;
    let sibling_spacing = global_constraint_factor(program, "sibSp")?;
    let level_spacing = constraint_factor(program, "sp", "hierRoot1")?;
    let outer_margin_factor = 0.1504;
    let lane_width = 576.0 / (2.0 + sibling_spacing + 2.0 * outer_margin_factor);
    let lane_height = lane_width * composite_height;
    let child_y = lane_height * (1.0 + level_spacing);
    let outer_margin = lane_width * outer_margin_factor;
    let lanes = [
        ((576.0 - lane_width) / 2.0, 0.0),
        (outer_margin, child_y),
        (outer_margin + lane_width * (1.0 + sibling_spacing), child_y),
    ];
    let mut shapes = Vec::with_capacity(6);
    for (index, node) in nodes.iter().enumerate() {
        let (x, y) = lanes[index];
        let width = lane_width * background_width;
        let height = width * background_height;
        let mut back_shape = prepared_shape(
            relative_rect(bounds, x, y, width, height),
            "roundRect",
            None,
            "node0",
            index,
        );
        back_shape.adjustments = vec![("adj", 10_000.0)];
        shapes.push(back_shape);
        let x = x + lane_width * text_left;
        let y = y + lane_width * text_top;
        let mut front_shape = prepared_shape(
            relative_rect(bounds, x, y, width, height),
            "roundRect",
            diagram_text(
                node.text,
                4_000,
                true,
                [7.2, 8.3, 7.2, 6.1],
                Some(DiagramLineSpacing::Points(4_300)),
            )?,
            if index == 0 { "fgAcc0" } else { "fgAcc2" },
            index,
        );
        front_shape.adjustments = vec![("adj", 10_000.0)];
        shapes.push(front_shape);
    }
    let mut connectors = Vec::with_capacity(edges.len());
    for edge in edges {
        let source_index = nodes
            .iter()
            .position(|node| node.id == edge.source)
            .ok_or_else(|| format!("missing hierarchy source `{}`", edge.source))?;
        let destination_index = nodes
            .iter()
            .position(|node| node.id == edge.destination)
            .ok_or_else(|| format!("missing hierarchy destination `{}`", edge.destination))?;
        let start = (
            shapes[source_index * 2].rect.x + shapes[source_index * 2].rect.width / 2.0,
            shapes[source_index * 2].rect.y + shapes[source_index * 2].rect.height,
        );
        let end = (
            shapes[destination_index * 2].rect.x + shapes[destination_index * 2].rect.width / 2.0,
            shapes[destination_index * 2].rect.y,
        );
        connectors.push(PreparedConnector {
            start,
            end,
            preset: "bentConnector3",
            style_label: "parChTrans1D1",
            custom_geometry: Some(BRANCH_GEOMETRY.to_owned()),
        });
    }
    Ok(PreparedDiagram { shapes, connectors })
}

const CYCLE1_NODE_RECTS: [(f64, f64, f64, f64); 3] = [
    (326.0, 18.0, 168.0, 144.0),
    (212.0, 216.0, 168.0, 144.0),
    (98.0, 18.0, 168.0, 144.0),
];
const CYCLE1_ARROW_RECTS: [(f64, f64, f64, f64); 3] = [
    (354.714_844, 162.070_312, 76.035_156, 123.066_407),
    (129.750_001, 162.269_614, 98.347_706, 120.429_749),
    (237.890_681, 4.234_377, 95.574_267, 57.816_436),
];
const CYCLE1_ARROW_PATHS: [&str; 3] = [
    r#"<a:moveTo><a:pt x="100000" y="324"/></a:moveTo><a:cubicBezTo><a:pt x="99178" y="34210"/><a:pt x="74410" y="66180"/><a:pt x="32207" y="87831"/></a:cubicBezTo><a:lnTo><a:pt x="44043" y="100000"/></a:lnTo><a:lnTo><a:pt x="0" y="88805"/></a:lnTo><a:lnTo><a:pt x="2219" y="56994"/></a:lnTo><a:lnTo><a:pt x="14025" y="69135"/></a:lnTo><a:cubicBezTo><a:pt x="45995" y="51532"/><a:pt x="64562" y="26456"/><a:pt x="65204" y="0"/></a:cubicBezTo><a:close/>"#,
    r#"<a:moveTo><a:pt x="86142" y="100000"/></a:moveTo><a:cubicBezTo><a:pt x="49791" y="82163"/><a:pt x="24808" y="52235"/><a:pt x="17762" y="18086"/></a:cubicBezTo><a:lnTo><a:pt x="0" y="18304"/></a:lnTo><a:lnTo><a:pt x="29209" y="0"/></a:lnTo><a:lnTo><a:pt x="62772" y="17532"/></a:lnTo><a:lnTo><a:pt x="45049" y="17749"/></a:lnTo><a:cubicBezTo><a:pt x="51690" y="44239"/><a:pt x="71621" y="67240"/><a:pt x="100000" y="81164"/></a:cubicBezTo><a:close/>"#,
    r#"<a:moveTo><a:pt x="0" y="38754"/></a:moveTo><a:cubicBezTo><a:pt x="26534" y="22309"/><a:pt x="55352" y="18681"/><a:pt x="83087" y="28295"/></a:cubicBezTo><a:lnTo><a:pt x="89504" y="0"/></a:lnTo><a:lnTo><a:pt x="100000" y="60178"/></a:lnTo><a:lnTo><a:pt x="66829" y="100000"/></a:lnTo><a:lnTo><a:pt x="73229" y="71772"/></a:lnTo><a:cubicBezTo><a:pt x="51907" y="65651"/><a:pt x="29983" y="69049"/><a:pt x="9719" y="81609"/></a:cubicBezTo><a:close/>"#,
];

fn authentic_cycle(nodes: &[RenderNode<'_>], bounds: Rect) -> Result<PreparedDiagram, String> {
    if nodes.len() != 3 {
        return Err(
            "PowerPoint 16.104 cycle1 compatibility profile requires exactly three data nodes"
                .to_owned(),
        );
    }
    let mut shapes = Vec::with_capacity(6);
    for (index, node) in nodes.iter().enumerate() {
        let (x, y, width, height) = CYCLE1_NODE_RECTS[index];
        shapes.push(prepared_shape(
            relative_rect(bounds, x, y, width, height),
            "rect",
            diagram_text(
                node.text,
                4_600,
                true,
                [7.2, 13.2, 7.2, 1.2],
                Some(DiagramLineSpacing::Points(4_900)),
            )?,
            "revTx",
            index,
        ));
        let (x, y, width, height) = CYCLE1_ARROW_RECTS[index];
        let mut arrow = prepared_shape(
            relative_rect(bounds, x, y, width, height),
            "circularArrow",
            None,
            "node1",
            index,
        );
        arrow.custom_geometry = Some(cycle1_custom_geometry(CYCLE1_ARROW_PATHS[index]));
        shapes.push(arrow);
    }
    Ok(PreparedDiagram {
        shapes,
        connectors: Vec::new(),
    })
}

fn cycle1_custom_geometry(path: &str) -> String {
    format!(
        r#"<a:custGeom xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:avLst/><a:gdLst/><a:ahLst/><a:cxnLst/><a:rect l="l" t="t" r="r" b="b"/><a:pathLst><a:path w="100000" h="100000">{path}</a:path></a:pathLst></a:custGeom>"#
    )
}
fn authentic_relationship(
    nodes: &[RenderNode<'_>],
    bounds: Rect,
    program: &DiagramRenderInstruction,
) -> Result<PreparedDiagram, String> {
    if nodes.len() != 3 {
        return Err("authentic SmartArt relationship requires exactly three data nodes".to_owned());
    }
    let aspect_ratio = selected_parameter_number(program, nodes.len(), "ar")?;
    let content_width = 360.0 * aspect_ratio;
    let content_x = (576.0 - content_width) * 1.222_222_222_222_222_3;
    let parent_left = selected_constraint_number(program, nodes.len(), "l", "Parent", "fact")?;
    let parent_top = selected_constraint_number(program, nodes.len(), "t", "Parent", "fact")?;
    let parent_width = selected_constraint_number(program, nodes.len(), "w", "Parent", "fact")?;
    let parent_height = selected_constraint_number(program, nodes.len(), "h", "Parent", "fact")?;
    let mut shapes = vec![prepared_shape(
        relative_rect(
            bounds,
            content_x + parent_left * content_width,
            parent_top * 360.0,
            parent_width * content_width,
            parent_height * 360.0,
        ),
        "ellipse",
        diagram_text(
            nodes[0].text,
            4_000,
            true,
            [12.0; 4],
            Some(DiagramLineSpacing::Points(4_500)),
        )?,
        "node0",
        0,
    )];
    for (index, name) in [
        "Accent1", "Accent2", "Accent3", "Accent4", "Accent5", "Accent6",
    ]
    .into_iter()
    .enumerate()
    {
        let left = selected_constraint_number(program, nodes.len(), "l", name, "fact")?;
        let top = selected_constraint_number(program, nodes.len(), "t", name, "fact")?;
        let width = selected_constraint_number(program, nodes.len(), "w", name, "fact")?;
        let height = selected_constraint_number(program, nodes.len(), "h", name, "fact")?;
        let size = (width * content_width).min(height * 360.0);
        shapes.push(prepared_shape(
            relative_rect(
                bounds,
                content_x + left * content_width,
                top * 360.0,
                size,
                size,
            ),
            "ellipse",
            None,
            if index % 2 == 0 { "node1" } else { "node2" },
            index,
        ));
    }
    Ok(PreparedDiagram {
        shapes,
        connectors: Vec::new(),
    })
}

fn authentic_matrix(
    nodes: &[RenderNode<'_>],
    bounds: Rect,
    program: &DiagramRenderInstruction,
) -> Result<PreparedDiagram, String> {
    if nodes.len() != 3 {
        return Err("authentic SmartArt matrix requires exactly three data nodes".to_owned());
    }
    let center_width =
        selected_constraint_number(program, nodes.len(), "w", "centerTile", "fact")? * 576.0;
    let center_height =
        selected_constraint_number(program, nodes.len(), "h", "centerTile", "fact")? * 360.0;
    let mut shapes = Vec::with_capacity(5);
    for (index, name) in ["tile1", "tile2", "tile3", "tile4"].into_iter().enumerate() {
        let left = selected_constraint_fraction(program, nodes.len(), "l", name)? * 576.0;
        let top = selected_constraint_fraction(program, nodes.len(), "t", name)? * 360.0;
        let right = selected_constraint_fraction(program, nodes.len(), "r", name)? * 576.0;
        let bottom = selected_constraint_fraction(program, nodes.len(), "b", name)? * 360.0;
        let mut tile = prepared_shape(
            relative_rect(bounds, left, top, right - left, bottom - top),
            "round1Rect",
            None,
            "node1",
            index,
        );
        tile.flip_horizontal = index % 2 == 0;
        tile.flip_vertical = index >= 2;
        shapes.push(tile);
    }
    shapes.push(prepared_shape(
        relative_rect(
            bounds,
            (576.0 - center_width) / 2.0,
            (360.0 - center_height) / 2.0,
            center_width,
            center_height,
        ),
        "roundRect",
        diagram_text(
            nodes[0].text,
            2_900,
            true,
            [8.7, 10.7, 8.7, 10.7],
            Some(DiagramLineSpacing::Points(3_200)),
        )?,
        "fgShp",
        0,
    ));
    Ok(PreparedDiagram {
        shapes,
        connectors: Vec::new(),
    })
}

fn authentic_pyramid(
    nodes: &[RenderNode<'_>],
    bounds: Rect,
    program: &DiagramRenderInstruction,
) -> Result<PreparedDiagram, String> {
    if nodes.len() != 3 {
        return Err("authentic SmartArt pyramid requires exactly three data nodes".to_owned());
    }
    let level_height = layout_node_constraint_number(program, "level", "h", "val")? * 120.0 / 500.0;
    let mut shapes = Vec::with_capacity(6);
    for (index, node) in nodes.iter().enumerate() {
        let width = 576.0 * (index + 1) as f64 / nodes.len() as f64;
        let x = (576.0 - width) / 2.0;
        let y = level_height * index as f64;
        let rect = relative_rect(bounds, x, y, width, level_height);
        let mut shape = prepared_shape(rect, "trapezoid", None, "node1", index);
        shape.custom_geometry = Some(pyramid_level_geometry(index));
        shapes.push(shape);
        let insets = match index {
            0 => [4.4, 7.0, 4.4, 1.0],
            1 => [96.0, 6.4, 96.0, 4.4],
            2 => [4.4, 6.5, 4.4, 2.3],
            _ => unreachable!("three pyramid levels"),
        };
        shapes.push(prepared_shape(
            rect,
            "rect",
            diagram_text(
                node.text,
                4_400,
                true,
                insets,
                Some(DiagramLineSpacing::Points(4_600)),
            )?,
            "revTx",
            index,
        ));
    }
    Ok(PreparedDiagram {
        shapes,
        connectors: Vec::new(),
    })
}

fn pyramid_level_geometry(index: usize) -> String {
    let inset = (50_000.0 / (index + 1) as f64).round() as i64;
    let right = 100_000 - inset;
    format!(
        r#"<a:custGeom xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:avLst/><a:gdLst/><a:ahLst/><a:cxnLst/><a:rect l="l" t="t" r="r" b="b"/><a:pathLst><a:path w="100000" h="100000"><a:moveTo><a:pt x="0" y="100000"/></a:moveTo><a:lnTo><a:pt x="{inset}" y="0"/></a:lnTo><a:lnTo><a:pt x="{right}" y="0"/></a:lnTo><a:lnTo><a:pt x="100000" y="100000"/></a:lnTo><a:close/></a:path></a:pathLst></a:custGeom>"#
    )
}

fn prepared_shape(
    rect: Rect,
    preset: &'static str,
    text: Option<CT_TextBody>,
    style_label: &'static str,
    color_index: usize,
) -> PreparedShape {
    PreparedShape {
        rect,
        preset,
        text,
        style_label,
        color_index,
        rotation_degrees: 0.0,
        flip_horizontal: false,
        flip_vertical: false,
        adjustments: Vec::new(),
        custom_geometry: None,
    }
}

fn diagram_text(
    text: Option<&CT_TextBody>,
    font_size: i32,
    centered: bool,
    insets: [f64; 4],
    line_spacing: Option<DiagramLineSpacing>,
) -> Result<Option<CT_TextBody>, String> {
    let Some(text) = text else {
        return Ok(None);
    };
    let mut text = text.clone();
    let [left, top, right, bottom] =
        insets.map(|points| Coordinate32Value::Emu((points * EMU_PER_POINT).round() as i32));
    text.body_properties.left_inset = Some(left);
    text.body_properties.top_inset = Some(top);
    text.body_properties.right_inset = Some(right);
    text.body_properties.bottom_inset = Some(bottom);
    let mut xml = String::from_utf8(text.to_xml().map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let mut cursor = 0usize;
    let mut replaced = false;
    while let Some(relative) = xml[cursor..].find(" sz=\"") {
        let value_start = cursor + relative + 5;
        let value_end = xml[value_start..]
            .find('"')
            .map(|relative| value_start + relative)
            .ok_or_else(|| "SmartArt text size attribute is unterminated".to_owned())?;
        xml.replace_range(value_start..value_end, &font_size.to_string());
        cursor = value_start + font_size.to_string().len() + 1;
        replaced = true;
    }
    if !replaced {
        if let Some(run) = xml.find("<a:r>") {
            xml.insert_str(run + 5, &format!("<a:rPr sz=\"{font_size}\"/>"));
        } else {
            return Err("SmartArt owned text has no regular DrawingML run".to_owned());
        }
    }
    if centered {
        let paragraph_properties = line_spacing.map_or_else(
            || "<a:pPr algn=\"ctr\"/>".to_owned(),
            |spacing| {
                let spacing = match spacing {
                    DiagramLineSpacing::Points(value) => {
                        format!("<a:spcPts val=\"{value}\"/>")
                    }
                };
                format!("<a:pPr algn=\"ctr\"><a:lnSpc>{spacing}</a:lnSpc></a:pPr>")
            },
        );
        xml = xml.replace("<a:p>", &format!("<a:p>{paragraph_properties}"));
    }
    CT_TextBody::from_xml(xml.as_bytes())
        .map(Some)
        .map_err(|error| error.to_string())
}

fn relative_rect(bounds: Rect, x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        x: bounds.x + bounds.width * x / 576.0,
        y: bounds.y + bounds.height * y / 360.0,
        width: bounds.width * width / 576.0,
        height: bounds.height * height / 360.0,
    }
}

fn styled_shape(
    id: u32,
    prepared: &PreparedShape,
    styles: &CT_DiagramStyleDefinition,
    colors: &CT_DiagramColorsDefinition,
) -> Result<CT_Shape, String> {
    let style = style_for(prepared.style_label, styles)?;
    let colors = color_for(prepared.style_label, colors)?;
    let mut shape_transform = transform(prepared.rect);
    shape_transform.rotation = Angle::from_degrees(prepared.rotation_degrees);
    shape_transform.flip_horizontal = prepared.flip_horizontal;
    shape_transform.flip_vertical = prepared.flip_vertical;
    let mut shape = CT_Shape::new_preset(
        id,
        &format!("SmartArt node {id}"),
        prepared.preset,
        shape_transform,
    )
    .map_err(|error| error.to_string())?;
    for (name, value) in &prepared.adjustments {
        shape
            .shape_properties
            .preset_geometry
            .as_mut()
            .ok_or_else(|| "SmartArt shape has no preset geometry".to_owned())?
            .set_adjust_value(name, *value)
            .map_err(|error| error.to_string())?;
    }
    if let Some(xml) = prepared.custom_geometry.as_deref() {
        shape.shape_properties.preset_geometry = None;
        shape.shape_properties.custom_geometry =
            Some(CT_CustomGeometry2D::from_xml(xml.as_bytes()).map_err(|error| error.to_string())?);
    }
    shape.text_body = prepared.text.clone();
    apply_style_to_shape(shape, style, colors, prepared.color_index)
}

fn styled_connector(
    id: u32,
    prepared: &PreparedConnector,
    styles: &CT_DiagramStyleDefinition,
    colors: &CT_DiagramColorsDefinition,
) -> Result<CT_ConnectionShape, String> {
    let (start_x, start_y) = prepared.start;
    let (end_x, end_y) = prepared.end;
    let rect = Rect {
        x: start_x.min(end_x),
        y: start_y.min(end_y),
        width: (end_x - start_x).abs().max(1.0 / EMU_PER_POINT),
        height: (end_y - start_y).abs().max(1.0 / EMU_PER_POINT),
    };
    let mut connector_transform = transform(rect);
    connector_transform.flip_horizontal = end_x < start_x;
    connector_transform.flip_vertical = end_y < start_y;
    let mut connector = CT_ConnectionShape::new_free_standing(
        id,
        &format!("SmartArt connector {id}"),
        prepared.preset,
        connector_transform,
    )
    .map_err(|error| error.to_string())?;
    if let Some(xml) = prepared.custom_geometry.as_deref() {
        connector.shape_properties.preset_geometry = None;
        connector.shape_properties.custom_geometry =
            Some(CT_CustomGeometry2D::from_xml(xml.as_bytes()).map_err(|error| error.to_string())?);
    }
    let style = style_for(prepared.style_label, styles)?;
    let colors = color_for(prepared.style_label, colors)?;
    let mut line = CT_LineProperties::default();
    line.width = Some(25_400);
    line.join = Some(LineJoin::round());
    let mut solid = SolidFill::default();
    solid.color = Some(cyclic_color(&colors.line, 0, "line")?.clone());
    line.fill = Some(Fill::Solid(solid));
    connector.shape_properties.line = Some(line);
    apply_style_to_connector(connector, style, colors)
}

fn apply_style_to_shape(
    shape: CT_Shape,
    style: &DiagramShapeStyle,
    colors: &DiagramColorRenderLabel,
    index: usize,
) -> Result<CT_Shape, String> {
    let style_xml = style_xml(style, colors, index)?;
    let xml = String::from_utf8(shape.to_xml().map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let marker = xml
        .find("<p:txBody")
        .unwrap_or_else(|| xml.rfind("</p:sp>").expect("shape end"));
    let expanded = format!("{}{}{}", &xml[..marker], style_xml, &xml[marker..]);
    CT_Shape::from_xml(expanded.as_bytes()).map_err(|error| error.to_string())
}

fn apply_style_to_connector(
    connector: CT_ConnectionShape,
    style: &DiagramShapeStyle,
    colors: &DiagramColorRenderLabel,
) -> Result<CT_ConnectionShape, String> {
    let style_xml = style_xml(style, colors, 0)?;
    let xml = String::from_utf8(connector.to_xml().map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let marker = xml
        .rfind("</p:cxnSp>")
        .ok_or_else(|| "connector XML has no end tag".to_owned())?;
    let expanded = format!("{}{}{}", &xml[..marker], style_xml, &xml[marker..]);
    CT_ConnectionShape::from_xml(expanded.as_bytes()).map_err(|error| error.to_string())
}

fn style_xml(
    style: &DiagramShapeStyle,
    colors: &DiagramColorRenderLabel,
    index: usize,
) -> Result<String, String> {
    let line = style
        .line_reference
        .ok_or_else(|| "SmartArt style label has no line reference".to_owned())?;
    let fill = style
        .fill_reference
        .ok_or_else(|| "SmartArt style label has no fill reference".to_owned())?;
    let effect = style
        .effect_reference
        .ok_or_else(|| "SmartArt style label has no effect reference".to_owned())?;
    let font = style
        .font_reference
        .as_deref()
        .ok_or_else(|| "SmartArt style label has no font reference".to_owned())?;
    if !matches!(font, "major" | "minor" | "none") {
        return Err(format!("unsupported SmartArt font reference `{font}`"));
    }
    let fill_color = cyclic_color(&colors.fill, index, "fill")?;
    let line_color = cyclic_color(&colors.line, index, "line")?;
    let effect_color = colors
        .effect
        .get(index % colors.effect.len().max(1))
        .unwrap_or(fill_color);
    let text_color = colors
        .text_fill
        .get(index % colors.text_fill.len().max(1))
        .unwrap_or(fill_color);
    Ok(format!(
        "<p:style><a:lnRef idx=\"{line}\">{}</a:lnRef><a:fillRef idx=\"{fill}\">{}</a:fillRef><a:effectRef idx=\"{effect}\">{}</a:effectRef><a:fontRef idx=\"{font}\">{}</a:fontRef></p:style>",
        color_xml(line_color)?,
        color_xml(fill_color)?,
        color_xml(effect_color)?,
        color_xml(text_color)?,
    ))
}

fn color_xml(color: &ColorChoice) -> Result<String, String> {
    let mut solid = SolidFill::default();
    solid.color = Some(color.clone());
    let xml = String::from_utf8(
        Fill::Solid(solid)
            .to_xml()
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    xml.strip_prefix("<a:solidFill>")
        .and_then(|value| value.strip_suffix("</a:solidFill>"))
        .map(str::to_owned)
        .ok_or_else(|| "cannot encode SmartArt colour choice".to_owned())
}

fn cyclic_color<'a>(
    colors: &'a [ColorChoice],
    index: usize,
    kind: &str,
) -> Result<&'a ColorChoice, String> {
    colors
        .get(index % colors.len().max(1))
        .ok_or_else(|| format!("SmartArt colour label has no {kind} colour"))
}

fn style_for<'a>(
    label: &str,
    styles: &'a CT_DiagramStyleDefinition,
) -> Result<&'a DiagramShapeStyle, String> {
    styles
        .labels
        .iter()
        .find(|candidate| candidate.name == label)
        .and_then(|candidate| candidate.shape_style.as_ref())
        .ok_or_else(|| format!("SmartArt style label `{label}` has no shape style"))
}

fn color_for<'a>(
    label: &str,
    colors: &'a CT_DiagramColorsDefinition,
) -> Result<&'a DiagramColorRenderLabel, String> {
    colors
        .render_projection()
        .labels
        .iter()
        .find(|candidate| candidate.name == label)
        .ok_or_else(|| format!("SmartArt colour label `{label}` is missing"))
}

fn require_owned_label(
    label: &str,
    styles: &CT_DiagramStyleDefinition,
    colors: &CT_DiagramColorsDefinition,
) -> Result<(), String> {
    style_for(label, styles)?;
    color_for(label, colors)?;
    Ok(())
}

fn placeholder_group(
    frame: &CT_GraphicFrame,
    frame_id: u32,
    frame_name: &str,
    allocator: &mut ShapeIdAllocator,
) -> Result<CT_GroupShape, String> {
    let bounds = frame_bounds(frame)?;
    let mut group = CT_GroupShape::new_empty(frame_id, frame_name);
    let mut shape = CT_Shape::new_preset(
        allocator.allocate(),
        "Unsupported SmartArt",
        "rect",
        transform(bounds),
    )
    .map_err(|error| error.to_string())?;
    shape.text_body = Some(
        CT_TextBody::from_xml(
            br#"<a:txBody xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:bodyPr anchor="ctr"/><a:lstStyle/><a:p><a:pPr algn="ctr"/><a:r><a:t>Unsupported SmartArt</a:t></a:r></a:p></a:txBody>"#,
        )
        .map_err(|error| error.to_string())?,
    );
    group.children.push(ShapeTreeChild::Shape(shape));
    apply_frame_transform(&mut group, frame)?;
    Ok(group)
}

fn apply_frame_transform(group: &mut CT_GroupShape, frame: &CT_GraphicFrame) -> Result<(), String> {
    let offset = frame
        .transform
        .offset
        .ok_or_else(|| "SmartArt frame has no offset".to_owned())?;
    let extent = frame
        .transform
        .extent
        .ok_or_else(|| "SmartArt frame has no extent".to_owned())?;
    let mut transform = CT_Transform2D::default();
    transform.offset = Some(offset);
    transform.extent = Some(extent);
    transform.child_offset = Some(offset);
    transform.child_extent = Some(extent);
    transform.rotation = frame.transform.rotation;
    transform.flip_horizontal = frame.transform.flip_horizontal;
    transform.flip_vertical = frame.transform.flip_vertical;
    *group.group_transform_mut() = transform;
    Ok(())
}

fn frame_bounds(frame: &CT_GraphicFrame) -> Result<Rect, String> {
    let offset = frame
        .transform
        .offset
        .as_ref()
        .ok_or_else(|| "SmartArt frame has no offset".to_owned())?;
    let extent = frame
        .transform
        .extent
        .as_ref()
        .ok_or_else(|| "SmartArt frame has no extent".to_owned())?;
    let bounds = Rect {
        x: offset.x.0 as f64 / EMU_PER_POINT,
        y: offset.y.0 as f64 / EMU_PER_POINT,
        width: extent.cx.0 as f64 / EMU_PER_POINT,
        height: extent.cy.0 as f64 / EMU_PER_POINT,
    };
    validate_bounds(bounds)?;
    Ok(bounds)
}

fn transform(rect: Rect) -> CT_Transform2D {
    let mut transform = CT_Transform2D::default();
    transform.offset = Some(CT_Point2D {
        x: Emu((rect.x * EMU_PER_POINT).round() as i64),
        y: Emu((rect.y * EMU_PER_POINT).round() as i64),
    });
    transform.extent = Some(CT_PositiveSize2D {
        cx: Emu((rect.width * EMU_PER_POINT).round() as i64),
        cy: Emu((rect.height * EMU_PER_POINT).round() as i64),
    });
    transform
}

fn validate_bounds(bounds: Rect) -> Result<(), String> {
    if !finite_rect(bounds) {
        return Err("SmartArt frame has non-finite or empty bounds".to_owned());
    }
    Ok(())
}

fn validate_rects(rects: &[Rect], bounds: Rect) -> Result<(), String> {
    if rects.is_empty()
        || rects
            .iter()
            .any(|rect| !finite_rect(*rect) || !contains(bounds, *rect))
    {
        return Err("SmartArt layout produced geometry outside authoritative bounds".to_owned());
    }
    Ok(())
}

fn finite_rect(rect: Rect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn contains(bounds: Rect, rect: Rect) -> bool {
    const EPSILON: f64 = 1.0e-7;
    rect.x + EPSILON >= bounds.x
        && rect.y + EPSILON >= bounds.y
        && rect.x + rect.width <= bounds.x + bounds.width + EPSILON
        && rect.y + rect.height <= bounds.y + bounds.height + EPSILON
}

fn attribute<'a>(record: &'a [(String, String)], name: &str) -> Option<&'a str> {
    record
        .iter()
        .find_map(|(candidate, value)| (candidate == name).then_some(value.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    const RESOURCE_ROOT: &str = "/Applications/Microsoft PowerPoint.app/Contents/Frameworks/SmartArt.framework/Versions/A/Resources/lo";
    const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    const DGM_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/diagram";

    fn text(value: &str) -> CT_TextBody {
        CT_TextBody::from_xml(
            format!(
                r#"<a:txBody xmlns:a="{A_NS}"><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr sz="1600"/><a:t>{value}</a:t></a:r></a:p></a:txBody>"#
            )
            .as_bytes(),
        )
        .unwrap()
    }

    fn test_program() -> DiagramRenderInstruction {
        let constraints = [
            ("w", "parentLeftMargin", "fact", "0.05"),
            ("w", "parentText", "fact", "0.7"),
            ("w", "background", "fact", "0.9"),
            ("h", "background", "fact", "0.635"),
            ("l", "text", "fact", "0.1"),
            ("t", "text", "fact", "0.095"),
            ("h", "composite", "fact", "0.667"),
            ("sp", "hierRoot1", "fact", "0.25"),
            ("w", "dummy", "fact", "2.8"),
            ("w", "Parent", "fact", "0.5377"),
            ("l", "Parent", "fact", "0.1886"),
            ("t", "Parent", "fact", "0.039"),
            ("h", "Parent", "fact", "0.856"),
            ("l", "Accent1", "fact", "0.4954"),
            ("t", "Accent1", "fact", "0"),
            ("w", "Accent1", "fact", "0.0598"),
            ("h", "Accent1", "fact", "0.0952"),
            ("l", "Accent2", "fact", "0.3538"),
            ("t", "Accent2", "fact", "0.8314"),
            ("w", "Accent2", "fact", "0.0433"),
            ("h", "Accent2", "fact", "0.069"),
            ("l", "Accent3", "fact", "0.7609"),
            ("t", "Accent3", "fact", "0.3864"),
            ("w", "Accent3", "fact", "0.0433"),
            ("h", "Accent3", "fact", "0.069"),
            ("l", "Accent4", "fact", "0.5537"),
            ("t", "Accent4", "fact", "0.9048"),
            ("w", "Accent4", "fact", "0.0598"),
            ("h", "Accent4", "fact", "0.0952"),
            ("l", "Accent5", "fact", "0.3661"),
            ("t", "Accent5", "fact", "0.1353"),
            ("w", "Accent5", "fact", "0.0433"),
            ("h", "Accent5", "fact", "0.069"),
            ("l", "Accent6", "fact", "0.2296"),
            ("t", "Accent6", "fact", "0.53"),
            ("w", "Accent6", "fact", "0.0433"),
            ("h", "Accent6", "fact", "0.069"),
            ("w", "centerTile", "fact", "0.3"),
            ("h", "centerTile", "fact", "0.25"),
            ("l", "tile1", "val", "0"),
            ("t", "tile1", "val", "0"),
            ("r", "tile1", "fact", "0.5"),
            ("b", "tile1", "fact", "0.5"),
            ("l", "tile2", "fact", "0.5"),
            ("t", "tile2", "val", "0"),
            ("r", "tile2", "fact", "1"),
            ("b", "tile2", "fact", "0.5"),
            ("l", "tile3", "val", "0"),
            ("t", "tile3", "fact", "0.5"),
            ("r", "tile3", "fact", "0.5"),
            ("b", "tile3", "fact", "1"),
            ("l", "tile4", "fact", "0.5"),
            ("t", "tile4", "fact", "0.5"),
            ("r", "tile4", "fact", "1"),
            ("b", "tile4", "fact", "1"),
        ];
        DiagramRenderInstruction {
            kind: DiagramRenderInstructionKind::LayoutNode,
            attributes: vec![("name".to_owned(), "linear".to_owned())],
            children: constraints
                .into_iter()
                .map(
                    |(kind, owner, value_name, value)| DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::Constraint,
                        attributes: vec![
                            ("type".to_owned(), kind.to_owned()),
                            ("forName".to_owned(), owner.to_owned()),
                            (value_name.to_owned(), value.to_owned()),
                        ],
                        children: vec![],
                    },
                )
                .chain([
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::Constraint,
                        attributes: vec![
                            ("type".to_owned(), "sibSp".to_owned()),
                            ("refForName".to_owned(), "node".to_owned()),
                            ("fact".to_owned(), "0.15".to_owned()),
                        ],
                        children: vec![],
                    },
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::Constraint,
                        attributes: vec![
                            ("type".to_owned(), "w".to_owned()),
                            ("ptType".to_owned(), "sibTrans".to_owned()),
                            ("fact".to_owned(), "0.5".to_owned()),
                        ],
                        children: vec![],
                    },
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::Parameter,
                        attributes: vec![
                            ("type".to_owned(), "stAng".to_owned()),
                            ("val".to_owned(), "0".to_owned()),
                        ],
                        children: vec![],
                    },
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::Parameter,
                        attributes: vec![
                            ("type".to_owned(), "spanAng".to_owned()),
                            ("val".to_owned(), "360".to_owned()),
                        ],
                        children: vec![],
                    },
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::Parameter,
                        attributes: vec![
                            ("type".to_owned(), "ar".to_owned()),
                            ("val".to_owned(), "1.592".to_owned()),
                        ],
                        children: vec![],
                    },
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::Constraint,
                        attributes: vec![
                            ("type".to_owned(), "sibSp".to_owned()),
                            ("fact".to_owned(), "0.1".to_owned()),
                        ],
                        children: vec![],
                    },
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::LayoutNode,
                        attributes: vec![("name".to_owned(), "sibTrans".to_owned())],
                        children: vec![
                            DiagramRenderInstruction {
                                kind: DiagramRenderInstructionKind::Shape,
                                attributes: vec![("type".to_owned(), "conn".to_owned())],
                                children: vec![],
                            },
                            DiagramRenderInstruction {
                                kind: DiagramRenderInstructionKind::ConstraintList,
                                attributes: vec![],
                                children: vec![DiagramRenderInstruction {
                                    kind: DiagramRenderInstructionKind::Constraint,
                                    attributes: vec![
                                        ("type".to_owned(), "h".to_owned()),
                                        ("fact".to_owned(), "0.65".to_owned()),
                                    ],
                                    children: vec![],
                                }],
                            },
                        ],
                    },
                    DiagramRenderInstruction {
                        kind: DiagramRenderInstructionKind::LayoutNode,
                        attributes: vec![("name".to_owned(), "level".to_owned())],
                        children: vec![
                            DiagramRenderInstruction {
                                kind: DiagramRenderInstructionKind::Shape,
                                attributes: vec![("type".to_owned(), "trapezoid".to_owned())],
                                children: vec![],
                            },
                            DiagramRenderInstruction {
                                kind: DiagramRenderInstructionKind::ConstraintList,
                                attributes: vec![],
                                children: vec![DiagramRenderInstruction {
                                    kind: DiagramRenderInstructionKind::Constraint,
                                    attributes: vec![
                                        ("type".to_owned(), "h".to_owned()),
                                        ("forName".to_owned(), "level".to_owned()),
                                        ("val".to_owned(), "500".to_owned()),
                                    ],
                                    children: vec![],
                                }],
                            },
                        ],
                    },
                ])
                .collect(),
        }
    }

    fn mutate_constraint(
        program: &mut DiagramRenderInstruction,
        owner: &str,
        value_name: &str,
        value: &str,
    ) {
        fn find<'a>(
            instruction: &'a mut DiagramRenderInstruction,
            owner: &str,
        ) -> Option<&'a mut DiagramRenderInstruction> {
            if attribute(&instruction.attributes, "forName") == Some(owner) {
                return Some(instruction);
            }
            instruction
                .children
                .iter_mut()
                .find_map(|child| find(child, owner))
        }
        let constraint = find(program, owner).unwrap();
        constraint
            .attributes
            .iter_mut()
            .find(|(name, _)| name == value_name)
            .unwrap()
            .1 = value.to_owned();
    }

    #[test]
    fn authentic_programs_keep_text_and_decorative_ownership_distinct() {
        let texts = [text("one"), text("two"), text("three")];
        let nodes = [
            RenderNode {
                id: "1",
                text: Some(&texts[0]),
            },
            RenderNode {
                id: "2",
                text: Some(&texts[1]),
            },
            RenderNode {
                id: "3",
                text: Some(&texts[2]),
            },
        ];
        let edges = [
            GraphEdge {
                source: "1",
                destination: "2",
                source_order: 0,
                destination_order: 0,
            },
            GraphEdge {
                source: "1",
                destination: "3",
                source_order: 1,
                destination_order: 0,
            },
        ];
        let bounds = Rect {
            x: 72.0,
            y: 72.0,
            width: 576.0,
            height: 360.0,
        };
        let list_program = test_program();
        let prepared = [
            authentic_list(&nodes, bounds, &list_program).unwrap(),
            authentic_hierarchy(&nodes, &edges, bounds, &list_program).unwrap(),
            authentic_cycle(&nodes, bounds).unwrap(),
            authentic_relationship(&nodes, bounds, &list_program).unwrap(),
            authentic_matrix(&nodes, bounds, &list_program).unwrap(),
            authentic_pyramid(&nodes, bounds, &list_program).unwrap(),
        ];
        assert_eq!(
            prepared
                .iter()
                .map(|item| item.shapes.len())
                .collect::<Vec<_>>(),
            [6, 6, 6, 7, 5, 6]
        );
        assert_eq!(prepared[1].connectors.len(), 2);
        assert!(
            prepared[1]
                .shapes
                .iter()
                .all(|shape| shape.adjustments == [("adj", 10_000.0)])
        );
        assert!(prepared[1].connectors.iter().all(|connector| {
            connector.preset == "bentConnector3"
                && connector
                    .custom_geometry
                    .as_deref()
                    .is_some_and(|geometry| geometry.contains("68148"))
        }));
        assert_eq!(
            prepared
                .iter()
                .map(|item| item
                    .shapes
                    .iter()
                    .filter(|shape| shape.text.is_some())
                    .count())
                .collect::<Vec<_>>(),
            [3, 3, 3, 1, 1, 3]
        );
        for (index, item) in prepared.iter().enumerate() {
            validate_rects(
                &item
                    .shapes
                    .iter()
                    .map(|shape| shape.rect)
                    .collect::<Vec<_>>(),
                bounds,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "family {index}: {error}: {:?}",
                    item.shapes
                        .iter()
                        .map(|shape| shape.rect)
                        .collect::<Vec<_>>()
                )
            });
        }
    }

    #[test]
    fn data_only_graph_is_ordered_and_cycles_or_duplicate_edges_fail_closed() {
        let layout = CT_DiagramLayoutDefinition::from_xml(
            format!(r#"<dgm:layoutDef xmlns:dgm="{DGM_NS}"><dgm:layoutNode/></dgm:layoutDef>"#)
                .as_bytes(),
        )
        .unwrap();
        let parse = |connections: &str| {
            CT_DiagramData::from_xml(
                format!(r#"<dgm:dataModel xmlns:dgm="{DGM_NS}"><dgm:ptLst><dgm:pt modelId="0" type="doc"><dgm:prSet/></dgm:pt><dgm:pt modelId="3"><dgm:prSet/></dgm:pt><dgm:pt modelId="1"><dgm:prSet/></dgm:pt><dgm:pt modelId="2"><dgm:prSet/></dgm:pt></dgm:ptLst><dgm:cxnLst>{connections}</dgm:cxnLst></dgm:dataModel>"#).as_bytes(),
            )
            .unwrap()
        };
        let data = parse(
            r#"<dgm:cxn modelId="4" srcId="0" destId="1" srcOrd="0" destOrd="0" type="parOf"/><dgm:cxn modelId="5" srcId="1" destId="3" srcOrd="1" destOrd="0" type="parOf"/><dgm:cxn modelId="6" srcId="1" destId="2" srcOrd="0" destOrd="0" type="parOf"/>"#,
        );
        let (nodes, edges) = presentation_graph(&data, &layout).unwrap();
        assert_eq!(
            nodes.iter().map(|node| node.id).collect::<Vec<_>>(),
            ["1", "2", "3"]
        );
        assert_eq!(
            edges
                .iter()
                .map(|edge| edge.destination)
                .collect::<Vec<_>>(),
            ["2", "3"]
        );

        let duplicate = parse(
            r#"<dgm:cxn modelId="4" srcId="1" destId="2" type="parOf"/><dgm:cxn modelId="5" srcId="1" destId="2" type="parOf"/>"#,
        );
        assert!(
            presentation_graph(&duplicate, &layout)
                .unwrap_err()
                .contains("duplicate SmartArt data topology")
        );
        let cyclic = parse(
            r#"<dgm:cxn modelId="4" srcId="1" destId="2" type="parOf"/><dgm:cxn modelId="5" srcId="2" destId="1" type="parOf"/>"#,
        );
        assert!(
            presentation_graph(&cyclic, &layout)
                .unwrap_err()
                .contains("contains a cycle")
        );
    }

    #[test]
    fn cycle1_compatibility_geometry_is_exact_and_three_node_only() {
        let texts = [text("one"), text("two"), text("three")];
        let nodes = [
            RenderNode {
                id: "1",
                text: Some(&texts[0]),
            },
            RenderNode {
                id: "2",
                text: Some(&texts[1]),
            },
            RenderNode {
                id: "3",
                text: Some(&texts[2]),
            },
        ];
        let bounds = Rect {
            x: 72.0,
            y: 72.0,
            width: 576.0,
            height: 360.0,
        };
        let prepared = authentic_cycle(&nodes, bounds).unwrap();
        assert_eq!(
            prepared
                .shapes
                .iter()
                .map(|shape| shape.rect)
                .collect::<Vec<_>>(),
            vec![
                relative_rect(bounds, 326.0, 18.0, 168.0, 144.0),
                relative_rect(bounds, 354.714_844, 162.070_312, 76.035_156, 123.066_407,),
                relative_rect(bounds, 212.0, 216.0, 168.0, 144.0),
                relative_rect(bounds, 129.750_001, 162.269_614, 98.347_706, 120.429_749,),
                relative_rect(bounds, 98.0, 18.0, 168.0, 144.0),
                relative_rect(bounds, 237.890_681, 4.234_377, 95.574_267, 57.816_436,),
            ]
        );
        assert_eq!(
            prepared
                .shapes
                .iter()
                .filter(|shape| shape.text.is_some())
                .count(),
            3
        );
        assert!(prepared.shapes.iter().skip(1).step_by(2).all(|shape| {
            shape
                .custom_geometry
                .as_deref()
                .is_some_and(|geometry| geometry.contains("<a:cubicBezTo>"))
        }));
        assert!(
            authentic_cycle(&nodes[..2], bounds)
                .unwrap_err()
                .contains("exactly three")
        );
    }

    #[test]
    fn nested_iteration_exhausts_one_checked_total_work_budget() {
        let mut nested = DiagramRenderInstruction {
            kind: DiagramRenderInstructionKind::Shape,
            attributes: vec![],
            children: vec![],
        };
        for _ in 0..12 {
            nested = DiagramRenderInstruction {
                kind: DiagramRenderInstructionKind::ForEach,
                attributes: vec![("axis".to_owned(), "ch".to_owned())],
                children: vec![nested],
            };
        }
        let error = evaluate_program(&nested, 3).unwrap_err();
        assert!(error.contains("total work bound 65536"), "{error}");
    }

    #[test]
    fn duplicate_parameter_ownership_and_selected_matches_fail_closed() {
        let parameter = |value: &str| DiagramRenderInstruction {
            kind: DiagramRenderInstructionKind::Parameter,
            attributes: vec![
                ("type".to_owned(), "ar".to_owned()),
                ("val".to_owned(), value.to_owned()),
            ],
            children: vec![],
        };
        let algorithm = |values: &[&str]| DiagramRenderInstruction {
            kind: DiagramRenderInstructionKind::Algorithm,
            attributes: vec![("type".to_owned(), "composite".to_owned())],
            children: values.iter().map(|value| parameter(value)).collect(),
        };
        let duplicate_owner = algorithm(&["1", "2"]);
        assert!(
            validate_parameter_cardinality(&duplicate_owner, false)
                .unwrap_err()
                .contains("duplicate parameter `ar`")
        );

        let selected_duplicates = DiagramRenderInstruction {
            kind: DiagramRenderInstructionKind::LayoutNode,
            attributes: vec![],
            children: vec![algorithm(&["1"]), algorithm(&["2"])],
        };
        validate_parameter_cardinality(&selected_duplicates, false).unwrap();
        assert!(
            selected_parameter_number(&selected_duplicates, 3, "ar")
                .unwrap_err()
                .contains("duplicate selected parameter `ar`")
        );
    }

    #[test]
    fn authoritative_empty_data_text_never_uses_stale_presentation_text() {
        let layout = CT_DiagramLayoutDefinition::from_xml(
            format!(r#"<dgm:layoutDef xmlns:dgm="{DGM_NS}"><dgm:layoutNode/></dgm:layoutDef>"#)
                .as_bytes(),
        )
        .unwrap();
        let data = CT_DiagramData::from_xml(
            format!(r#"<dgm:dataModel xmlns:dgm="{DGM_NS}" xmlns:a="{A_NS}"><dgm:ptLst><dgm:pt modelId="1"><dgm:t><a:bodyPr/><a:lstStyle/><a:p/></dgm:t></dgm:pt><dgm:pt modelId="101" type="pres"><dgm:t><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>stale</a:t></a:r></a:p></dgm:t></dgm:pt></dgm:ptLst><dgm:cxnLst><dgm:cxn modelId="201" srcId="1" destId="101" type="presOf"/></dgm:cxnLst></dgm:dataModel>"#).as_bytes(),
        )
        .unwrap();
        let (nodes, _) = presentation_graph(&data, &layout).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].text.unwrap().plain_text(), "");
    }

    #[test]
    #[ignore = "requires the pinned Microsoft PowerPoint SmartArt resource corpus"]
    fn pinned_authentic_layout_instruction_programs_are_executable() {
        for file in [
            "list1.glo",
            "hierarchy1.glo",
            "cycle1.glo",
            "circlerelationship.glo",
            "matrix1.glo",
            "pyramid1.glo",
        ] {
            let bytes = fs::read(Path::new(RESOURCE_ROOT).join(file)).unwrap();
            let layout = CT_DiagramLayoutDefinition::from_xml(&bytes).unwrap();
            validate_authentic_layout_program(&layout, 3).unwrap();
        }
    }

    #[test]
    #[ignore = "requires the pinned Microsoft PowerPoint SmartArt resource corpus"]
    fn non_cycle_profiles_reject_duplicate_parameters_and_unsupported_known_values() {
        let bytes = fs::read(Path::new(RESOURCE_ROOT).join("circlerelationship.glo")).unwrap();
        let source = String::from_utf8(bytes).unwrap();
        let layout = CT_DiagramLayoutDefinition::from_xml(source.as_bytes()).unwrap();
        assert_eq!(layout.family, DiagramLayoutFamily::Relationship);
        validate_authentic_layout_program(&layout, 3).unwrap();

        let parameter = r#"<param type="ar" val="1.592" />"#;
        assert!(source.contains(parameter));
        let duplicate = source.replacen(parameter, &format!("{parameter}{parameter}"), 1);
        let duplicate = CT_DiagramLayoutDefinition::from_xml(duplicate.as_bytes()).unwrap();
        assert_eq!(duplicate.family, DiagramLayoutFamily::Relationship);
        assert!(
            validate_authentic_layout_program(&duplicate, 3)
                .unwrap_err()
                .contains("duplicate parameter `ar`")
        );

        let unsupported_parameter =
            source.replacen(parameter, r#"<param type="ar" val="1.593" />"#, 1);
        let unsupported_parameter =
            CT_DiagramLayoutDefinition::from_xml(unsupported_parameter.as_bytes()).unwrap();
        assert_eq!(
            unsupported_parameter.family,
            DiagramLayoutFamily::Relationship
        );
        assert!(
            validate_authentic_layout_program(&unsupported_parameter, 3)
                .unwrap_err()
                .contains("algorithm parameter semantics")
        );

        let shape = r#"<shape type="ellipse" />"#;
        assert!(source.contains(shape));
        let unsupported_shape = source.replacen(shape, r#"<shape type="hexagon" />"#, 1);
        let unsupported_shape =
            CT_DiagramLayoutDefinition::from_xml(unsupported_shape.as_bytes()).unwrap();
        assert_eq!(unsupported_shape.family, DiagramLayoutFamily::Relationship);
        assert!(
            validate_authentic_layout_program(&unsupported_shape, 3)
                .unwrap_err()
                .contains("shape semantics for type `hexagon`")
        );
    }

    #[test]
    #[ignore = "requires the pinned Microsoft PowerPoint SmartArt resource corpus"]
    fn non_cycle_profiles_reject_selector_semantic_cardinality_and_owner_mutations() {
        fn assert_rejected(file: &str, from: &str, to: &str, expected: &str) {
            let source =
                String::from_utf8(fs::read(Path::new(RESOURCE_ROOT).join(file)).unwrap()).unwrap();
            assert!(source.contains(from), "{file} lacks mutation marker {from}");
            let original = CT_DiagramLayoutDefinition::from_xml(source.as_bytes()).unwrap();
            let changed = source.replacen(from, to, 1);
            let changed = CT_DiagramLayoutDefinition::from_xml(changed.as_bytes()).unwrap();
            assert_eq!(changed.family, original.family);
            let error = validate_authentic_layout_program(&changed, 3).unwrap_err();
            assert!(error.contains(expected), "{file}: {error}");
        }

        fn assert_swapped_rejected(file: &str, first: &str, second: &str) {
            let source =
                String::from_utf8(fs::read(Path::new(RESOURCE_ROOT).join(file)).unwrap()).unwrap();
            assert!(source.contains(first), "{file} lacks first swap marker");
            assert!(source.contains(second), "{file} lacks second swap marker");
            let changed = source
                .replacen(first, "F220_SWAP_MARKER", 1)
                .replacen(second, first, 1)
                .replacen("F220_SWAP_MARKER", second, 1);
            let changed = CT_DiagramLayoutDefinition::from_xml(changed.as_bytes()).unwrap();
            let error = validate_authentic_layout_program(&changed, 3).unwrap_err();
            assert!(error.contains("exact supported profile"), "{file}: {error}");
        }

        fn assert_pair_rejected(file: &str, replacements: [(&str, &str); 2]) {
            let mut source =
                String::from_utf8(fs::read(Path::new(RESOURCE_ROOT).join(file)).unwrap()).unwrap();
            for (from, to) in replacements {
                assert!(source.contains(from), "{file} lacks mutation marker {from}");
                source = source.replacen(from, to, 1);
            }
            let changed = CT_DiagramLayoutDefinition::from_xml(source.as_bytes()).unwrap();
            let error = validate_authentic_layout_program(&changed, 3).unwrap_err();
            assert!(error.contains("exact supported profile"), "{file}: {error}");
        }

        assert_rejected(
            "list1.glo",
            "<presOf />",
            r#"<presOf axis="unsupported" />"#,
            "presOf selector",
        );
        assert_rejected(
            "list1.glo",
            r#"<constr type="h" val="0" />"#,
            r#"<constr type="tMarg" val="0" />"#,
            "constraint semantics for owner `parentLeftMargin`",
        );
        assert_rejected(
            "list1.glo",
            r#"<constr type="h" val="0" />"#,
            r#"<constr type="h" val="0" /><constr type="h" val="0" />"#,
            "unsupported cardinalities",
        );
        assert_rejected(
            "list1.glo",
            r#"<rule type="primFontSz" for="des" forName="parentText" val="5" />"#,
            r#"<rule type="w" for="des" forName="parentText" val="5" />"#,
            "rule semantics",
        );
        assert_rejected(
            "list1.glo",
            r#"<dir val="norm" />"#,
            r#"<dir val="rev" />"#,
            "variable `dir` value `rev`",
        );
        assert_rejected(
            "hierarchy1.glo",
            r#"<adj idx="1" val="0.1" />"#,
            r#"<adj idx="1" val="0.2" />"#,
            "adjustment index `1` value `0.2`",
        );
        assert_rejected(
            "hierarchy1.glo",
            r#"<forEach ref="repeat" />"#,
            r#"<forEach ref="unsupported" />"#,
            "forEach selector",
        );
        assert_rejected(
            "list1.glo",
            r#"<layoutNode name="parentLeftMargin">"#,
            r#"<layoutNode name="negativeSpace">"#,
            "owner `negativeSpace`",
        );
        assert_rejected(
            "list1.glo",
            "<shape />\r\n\t\t<presOf />",
            "<presOf />\r\n\t\t<shape />",
            "ordered children",
        );
        assert_rejected(
            "list1.glo",
            "</alg>",
            "</alg><shape />",
            "ordered children `alg,shape`",
        );
        assert_rejected(
            "list1.glo",
            r#"<alg type="sp" />"#,
            r#"<alg type="tx"><param type="stBulletLvl" val="1" /></alg>"#,
            "algorithm `tx` for owner `parentLeftMargin`",
        );
        assert_rejected(
            "list1.glo",
            r#"<presOf axis="self" />"#,
            "<presOf />",
            "presOf selector",
        );
        assert_rejected(
            "list1.glo",
            r#"<constr type="w" for="ch" forName="parentLin" refType="w" />"#,
            r#"<constr type="w" for="ch" forName="parentLin" />"#,
            "constraint field set",
        );
        assert_rejected(
            "list1.glo",
            r#"<rule type="primFontSz" for="des" forName="parentText" val="5" />"#,
            r#"<rule type="primFontSz" for="des" val="5" />"#,
            "rule semantics",
        );
        assert_rejected(
            "list1.glo",
            "</alg>",
            "</alg><presOf />",
            "ordered children `alg,presOf`",
        );
        assert_swapped_rejected(
            "list1.glo",
            r#"<constr type="w" for="ch" forName="parentLin" refType="w" />"#,
            r#"<constr type="h" for="ch" forName="parentLin" val="INF" />"#,
        );
        assert_pair_rejected(
            "list1.glo",
            [
                (
                    r#"<param type="horzAlign" val="l" />"#,
                    r#"<param type="horzAlign" val="r" />"#,
                ),
                (
                    r#"<param type="nodeHorzAlign" val="l" />"#,
                    r#"<param type="nodeHorzAlign" val="r" />"#,
                ),
            ],
        );
        assert_rejected(
            "list1.glo",
            r#"<constr type="h" for="ch" forName="parentLin" val="INF" />"#,
            r#"<constr type="w" for="ch" forName="parentLin" refType="w" />"#,
            "exact supported profile",
        );
        assert_rejected(
            "list1.glo",
            r#"<layoutNode name="linear">"#,
            r#"<layoutNode name="linear" styleLbl="node1">"#,
            "exact supported profile",
        );
        assert_swapped_rejected(
            "circlerelationship.glo",
            r#"<if axis="ch ch" ptType="node node" func="cnt" op="equ" val="0">"#,
            r#"<if axis="ch ch" ptType="node node" func="cnt" op="equ" val="1">"#,
        );
    }

    #[test]
    #[ignore = "requires the pinned Microsoft PowerPoint SmartArt resource corpus"]
    fn cycle1_compatibility_gate_rejects_identity_hash_instruction_and_node_variations() {
        let bytes = fs::read(Path::new(RESOURCE_ROOT).join("cycle1.glo")).unwrap();
        let layout = CT_DiagramLayoutDefinition::from_xml(&bytes).unwrap();
        let root = validate_authentic_layout_program(&layout, 3).unwrap();
        validate_instruction_tree(root, 0).unwrap();
        validate_cycle1_compatibility_profile(&layout, 3).unwrap();

        assert!(
            validate_cycle1_compatibility_profile(&layout, 2)
                .unwrap_err()
                .contains("exactly three")
        );

        let source = String::from_utf8(bytes).unwrap();
        let changed_identity =
            source.replacen(CYCLE1_COMPATIBILITY_ID, "urn:f220:changed-cycle1", 1);
        let changed_identity =
            CT_DiagramLayoutDefinition::from_xml(changed_identity.as_bytes()).unwrap();
        assert!(
            validate_cycle1_compatibility_profile(&changed_identity, 3)
                .unwrap_err()
                .contains("pinned cycle1 identity")
        );

        let byte_safe_change = format!("{source}\n");
        let byte_safe_change =
            CT_DiagramLayoutDefinition::from_xml(byte_safe_change.as_bytes()).unwrap();
        assert_eq!(byte_safe_change.unique_id, layout.unique_id);
        assert_eq!(
            byte_safe_change.render_projection(),
            layout.render_projection()
        );
        assert!(
            validate_cycle1_compatibility_profile(&byte_safe_change, 3)
                .unwrap_err()
                .contains("layout SHA-256")
        );

        assert!(source.contains("<shape />"));
        let changed_instruction = source.replacen("<shape />", "<shape rot=\"1\" />", 1);
        let changed_instruction =
            CT_DiagramLayoutDefinition::from_xml(changed_instruction.as_bytes()).unwrap();
        assert_ne!(
            changed_instruction.render_projection(),
            layout.render_projection()
        );
        assert!(
            validate_cycle1_compatibility_profile(&changed_instruction, 3)
                .unwrap_err()
                .contains("layout SHA-256")
        );
    }

    #[test]
    fn supported_constraint_mutation_moves_output_and_unknown_instruction_fails_closed() {
        let texts = [text("one"), text("two"), text("three")];
        let nodes = [
            RenderNode {
                id: "1",
                text: Some(&texts[0]),
            },
            RenderNode {
                id: "2",
                text: Some(&texts[1]),
            },
            RenderNode {
                id: "3",
                text: Some(&texts[2]),
            },
        ];
        let edges = [
            GraphEdge {
                source: "1",
                destination: "2",
                source_order: 0,
                destination_order: 0,
            },
            GraphEdge {
                source: "1",
                destination: "3",
                source_order: 1,
                destination_order: 0,
            },
        ];
        let bounds = Rect {
            x: 72.0,
            y: 72.0,
            width: 576.0,
            height: 360.0,
        };
        let original_program = test_program();
        let original = authentic_list(&nodes, bounds, &original_program).unwrap();
        let mut mutated_program = original_program.clone();
        mutate_constraint(&mut mutated_program, "parentText", "fact", "0.6");
        let mutated = authentic_list(&nodes, bounds, &mutated_program).unwrap();
        assert!((original.shapes[1].rect.width - 403.2).abs() < 1.0e-9);
        assert!((mutated.shapes[1].rect.width - 345.6).abs() < 1.0e-9);

        let original = authentic_hierarchy(&nodes, &edges, bounds, &original_program).unwrap();
        let mut mutated_program = original_program.clone();
        mutate_constraint(&mut mutated_program, "background", "fact", "0.8");
        let mutated = authentic_hierarchy(&nodes, &edges, bounds, &mutated_program).unwrap();
        assert_ne!(original.shapes[0].rect.width, mutated.shapes[0].rect.width);

        let original = authentic_relationship(&nodes, bounds, &original_program).unwrap();
        let mut mutated_program = original_program.clone();
        mutate_constraint(&mut mutated_program, "Parent", "fact", "0.5");
        let mutated = authentic_relationship(&nodes, bounds, &mutated_program).unwrap();
        assert_ne!(original.shapes[0].rect.width, mutated.shapes[0].rect.width);

        let original = authentic_matrix(&nodes, bounds, &original_program).unwrap();
        let mut mutated_program = original_program.clone();
        mutate_constraint(&mut mutated_program, "centerTile", "fact", "0.25");
        let mutated = authentic_matrix(&nodes, bounds, &mutated_program).unwrap();
        assert_ne!(original.shapes[4].rect.width, mutated.shapes[4].rect.width);

        let original = authentic_pyramid(&nodes, bounds, &original_program).unwrap();
        let mut mutated_program = original_program.clone();
        mutate_constraint(&mut mutated_program, "level", "val", "450");
        let mutated = authentic_pyramid(&nodes, bounds, &mutated_program).unwrap();
        assert_ne!(
            original.shapes[0].rect.height,
            mutated.shapes[0].rect.height
        );

        let unknown = DiagramRenderInstruction {
            kind: DiagramRenderInstructionKind::Unsupported("futureLayout".to_owned()),
            attributes: vec![],
            children: vec![],
        };
        assert!(
            validate_instruction_tree(&unknown, 0)
                .unwrap_err()
                .contains("futureLayout")
        );
    }
}
