//! A data-first node editor widget for [`egui`].
//!
//! Graph data, interaction state, rendering, and history are separate layers.
//! [`Graph`] can be persisted without serializing any egui value, while
//! [`FlowEditor::show`] provides a complete editor surface for applications.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};

use egui::{Align2, Color32, FontId, Id, Key, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// A stable identifier for a node.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct NodeId(pub u64);

/// A stable identifier for a port.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct PortId(pub u64);

/// A stable identifier for an edge.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct EdgeId(pub u64);

/// A graph-space point independent of egui.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: f32,
    /// Vertical coordinate.
    pub y: f32,
}

impl Point {
    /// Creates a point.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A graph-space size independent of egui.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Size {
    /// Width in graph units.
    pub width: f32,
    /// Height in graph units.
    pub height: f32,
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 180.0,
            height: 100.0,
        }
    }
}

impl Size {
    /// Creates a size.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// An axis-aligned graph-space rectangle.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Bounds {
    /// Minimum corner.
    pub min: Point,
    /// Maximum corner.
    pub max: Point,
}

impl Bounds {
    /// Creates bounds from two corners.
    #[must_use]
    pub const fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }
    /// Returns the center point.
    #[must_use]
    pub fn center(self) -> Point {
        Point::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }
    /// Expands the bounds to include another rectangle.
    pub fn include(&mut self, other: Bounds) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
    }
    /// Returns whether two rectangles overlap.
    #[must_use]
    pub fn intersects(self, other: Bounds) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

/// Whether a port receives or emits a connection.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PortDirection {
    /// Receives a connection.
    Input,
    /// Emits a connection.
    Output,
}

/// A typed connection point on a node.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Port {
    /// Stable identifier.
    pub id: PortId,
    /// Display label.
    pub name: String,
    /// Input or output direction.
    pub direction: PortDirection,
    /// Application-defined type name.
    pub kind: String,
}

/// A node in the persistent graph model.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Node {
    /// Stable identifier.
    pub id: NodeId,
    /// Display title.
    pub title: String,
    /// Top-left graph-space position.
    pub position: Point,
    /// Node dimensions.
    pub size: Size,
    /// Whether the body is collapsed.
    pub collapsed: bool,
    /// Typed input and output ports.
    pub ports: Vec<Port>,
    /// Application metadata retained during persistence.
    pub metadata: BTreeMap<String, String>,
}

impl Node {
    /// Creates an empty node.
    #[must_use]
    pub fn new(title: impl Into<String>, position: Point) -> Self {
        Self {
            id: NodeId(0),
            title: title.into(),
            position,
            size: Size::default(),
            collapsed: false,
            ports: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
    /// Adds an input port using builder syntax.
    pub fn input(mut self, name: impl Into<String>, kind: impl Into<String>) -> Self {
        self.ports.push(Port {
            id: PortId(0),
            name: name.into(),
            direction: PortDirection::Input,
            kind: kind.into(),
        });
        self
    }
    /// Adds an output port using builder syntax.
    pub fn output(mut self, name: impl Into<String>, kind: impl Into<String>) -> Self {
        self.ports.push(Port {
            id: PortId(0),
            name: name.into(),
            direction: PortDirection::Output,
            kind: kind.into(),
        });
        self
    }
    /// Returns a port by identifier.
    #[must_use]
    pub fn port(&self, id: PortId) -> Option<&Port> {
        self.ports.iter().find(|port| port.id == id)
    }
    /// Returns the node bounds.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        Bounds::new(
            self.position,
            Point::new(
                self.position.x + self.size.width,
                self.position.y + self.size.height,
            ),
        )
    }
}

/// A reference to one end of an edge.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PortRef {
    /// Owning node.
    pub node: NodeId,
    /// Port on that node.
    pub port: PortId,
}

/// Visual edge rendering styles.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum EdgeStyle {
    /// A direct line.
    #[default]
    Straight,
    /// A smooth curve.
    Bezier,
    /// Right-angle segments.
    Orthogonal,
}

/// A persistent graph edge.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Edge {
    /// Stable identifier.
    pub id: EdgeId,
    /// Output endpoint.
    pub from: PortRef,
    /// Input endpoint.
    pub to: PortRef,
    /// Rendering style.
    pub style: EdgeStyle,
}

/// Errors returned when a connection cannot be created.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionError {
    /// One endpoint does not exist.
    MissingPort(PortRef),
    /// Endpoint directions are not output-to-input.
    WrongDirection,
    /// A node cannot connect to itself.
    SelfConnection,
    /// An application validator rejected the connection.
    Rejected(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPort(port) => write!(
                formatter,
                "port {} on node {} does not exist",
                port.port.0, port.node.0
            ),
            Self::WrongDirection => {
                formatter.write_str("connections must run from an output to an input")
            }
            Self::SelfConnection => formatter.write_str("a node cannot connect to itself"),
            Self::Rejected(message) => write!(formatter, "connection rejected: {message}"),
        }
    }
}

impl std::error::Error for ConnectionError {}

/// A data-only graph that can be persisted without egui.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Graph {
    nodes: BTreeMap<NodeId, Node>,
    edges: BTreeMap<EdgeId, Edge>,
    next_node: u64,
    next_port: u64,
    next_edge: u64,
}

impl Graph {
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_node: 1,
            next_port: 1,
            next_edge: 1,
            ..Self::default()
        }
    }
    /// Inserts a node and returns its stable identifier.
    pub fn add_node(&mut self, mut node: Node) -> NodeId {
        let id = self.alloc_node();
        node.id = id;
        for port in &mut node.ports {
            port.id = self.alloc_port();
        }
        self.nodes.insert(id, node);
        id
    }
    /// Removes a node and all incident edges.
    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        let removed = self.nodes.remove(&id);
        if removed.is_some() {
            self.edges
                .retain(|_, edge| edge.from.node != id && edge.to.node != id);
        }
        removed
    }
    /// Returns a node by identifier.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    /// Returns a mutable node by identifier.
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }
    /// Iterates nodes in stable order.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }
    /// Iterates edges in stable order.
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.values()
    }
    /// Finds a port by endpoint reference.
    #[must_use]
    pub fn find_port(&self, reference: PortRef) -> Option<&Port> {
        self.nodes.get(&reference.node)?.port(reference.port)
    }
    /// Connects an output to an input.
    pub fn connect(
        &mut self,
        from: PortRef,
        to: PortRef,
        style: EdgeStyle,
    ) -> Result<EdgeId, ConnectionError> {
        let from_port = self
            .find_port(from)
            .ok_or(ConnectionError::MissingPort(from))?;
        let to_port = self.find_port(to).ok_or(ConnectionError::MissingPort(to))?;
        if from.node == to.node {
            return Err(ConnectionError::SelfConnection);
        }
        if from_port.direction != PortDirection::Output || to_port.direction != PortDirection::Input
        {
            return Err(ConnectionError::WrongDirection);
        }
        let id = self.alloc_edge();
        self.edges.insert(
            id,
            Edge {
                id,
                from,
                to,
                style,
            },
        );
        Ok(id)
    }
    /// Validates and connects an edge with application-defined rules.
    pub fn connect_with<V: ConnectionValidator>(
        &mut self,
        validator: &V,
        from: PortRef,
        to: PortRef,
        style: EdgeStyle,
    ) -> Result<EdgeId, ConnectionError> {
        validator.validate(self, from, to)?;
        self.connect(from, to, style)
    }
    /// Removes an edge by identifier.
    pub fn remove_edge(&mut self, id: EdgeId) -> Option<Edge> {
        self.edges.remove(&id)
    }
    /// Returns bounds containing every node.
    #[must_use]
    pub fn bounds(&self) -> Option<Bounds> {
        let mut result = self.nodes.values().map(Node::bounds).next()?;
        for node in self.nodes.values().skip(1) {
            result.include(node.bounds());
        }
        Some(result)
    }
    /// Serializes the graph as pretty JSON.
    #[cfg(feature = "persistence")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    /// Deserializes a graph from JSON.
    #[cfg(feature = "persistence")]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    fn alloc_node(&mut self) -> NodeId {
        let id = NodeId(self.next_node);
        self.next_node += 1;
        id
    }
    fn alloc_port(&mut self) -> PortId {
        let id = PortId(self.next_port);
        self.next_port += 1;
        id
    }
    fn alloc_edge(&mut self) -> EdgeId {
        let id = EdgeId(self.next_edge);
        self.next_edge += 1;
        id
    }
}

/// Application-defined connection validation.
pub trait ConnectionValidator {
    /// Validates an output and input before insertion.
    fn validate(&self, graph: &Graph, from: PortRef, to: PortRef) -> Result<(), ConnectionError>;
}

/// A validator that accepts every connection allowed by the graph model.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllowAll;

impl ConnectionValidator for AllowAll {
    fn validate(
        &self,
        _graph: &Graph,
        _from: PortRef,
        _to: PortRef,
    ) -> Result<(), ConnectionError> {
        Ok(())
    }
}

/// A command recorded by an [`Editor`].
#[derive(Clone, Debug)]
pub enum Command {
    /// Add a prepared node.
    AddNode(Node),
    /// Remove nodes and their edges.
    RemoveNodes(Vec<NodeId>),
    /// Move nodes by a graph-space delta.
    MoveNodes {
        /// Nodes to move.
        ids: Vec<NodeId>,
        /// Graph-space delta.
        delta: Point,
    },
    /// Remove edges.
    RemoveEdges(Vec<EdgeId>),
    /// Connect two ports.
    Connect {
        /// Output endpoint.
        from: PortRef,
        /// Input endpoint.
        to: PortRef,
        /// Visual style.
        style: EdgeStyle,
    },
    /// Resize a node.
    ResizeNode {
        /// Node to resize.
        id: NodeId,
        /// New dimensions.
        size: Size,
    },
    /// Change collapsed state.
    SetCollapsed {
        /// Node to change.
        id: NodeId,
        /// New state.
        collapsed: bool,
    },
}

/// Interaction and viewport state kept separate from graph data.
#[derive(Clone, Debug)]
pub struct EditorState {
    zoom: f32,
    pan: Vec2,
    selected_nodes: BTreeSet<NodeId>,
    selected_edges: BTreeSet<EdgeId>,
    grid_enabled: bool,
    snap_enabled: bool,
    grid_size: f32,
    clipboard: Vec<Node>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            selected_nodes: BTreeSet::new(),
            selected_edges: BTreeSet::new(),
            grid_enabled: true,
            snap_enabled: true,
            grid_size: 16.0,
            clipboard: Vec::new(),
        }
    }
}

impl EditorState {
    /// Current zoom factor.
    #[must_use]
    pub fn zoom(&self) -> f32 {
        self.zoom
    }
    /// Current pan offset.
    #[must_use]
    pub fn pan(&self) -> Vec2 {
        self.pan
    }
    /// Selected nodes.
    #[must_use]
    pub fn selected_nodes(&self) -> &BTreeSet<NodeId> {
        &self.selected_nodes
    }
    /// Selected edges.
    #[must_use]
    pub fn selected_edges(&self) -> &BTreeSet<EdgeId> {
        &self.selected_edges
    }
    /// Sets zoom, clamped to a usable range.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 4.0);
    }
    /// Sets pan.
    pub fn set_pan(&mut self, pan: Vec2) {
        self.pan = pan;
    }
    /// Enables or disables grid painting.
    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid_enabled = enabled;
    }
    /// Enables or disables snapping.
    pub fn set_snap_enabled(&mut self, enabled: bool) {
        self.snap_enabled = enabled;
    }
    /// Sets grid spacing.
    pub fn set_grid_size(&mut self, size: f32) {
        self.grid_size = size.max(1.0);
    }
    /// Converts graph coordinates to viewport coordinates.
    #[must_use]
    pub fn world_to_screen(&self, point: Point, viewport: Rect) -> Pos2 {
        viewport.min + self.pan + Vec2::new(point.x * self.zoom, point.y * self.zoom)
    }
    /// Converts viewport coordinates to graph coordinates.
    #[must_use]
    pub fn screen_to_world(&self, point: Pos2, viewport: Rect) -> Point {
        let value = (point - viewport.min - self.pan) / self.zoom;
        Point::new(value.x, value.y)
    }
    /// Snaps a point when snapping is enabled.
    #[must_use]
    pub fn snap(&self, point: Point) -> Point {
        if self.snap_enabled {
            Point::new(
                (point.x / self.grid_size).round() * self.grid_size,
                (point.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            point
        }
    }
    /// Fits bounds inside a viewport with a margin.
    pub fn fit_to_view(&mut self, bounds: Bounds, viewport: Rect) {
        let width = (bounds.max.x - bounds.min.x).max(1.0);
        let height = (bounds.max.y - bounds.min.y).max(1.0);
        self.zoom = ((viewport.width() - 48.0) / width)
            .min((viewport.height() - 48.0) / height)
            .clamp(0.1, 4.0);
        let center = self.world_to_screen(bounds.center(), viewport);
        self.pan += viewport.center() - center;
    }
}

#[derive(Clone, Debug)]
struct History {
    undo: Vec<Graph>,
    redo: Vec<Graph>,
    limit: usize,
}
impl Default for History {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: 100,
        }
    }
}

/// Stateful controller combining graph data, selection, clipboard, and history.
pub struct Editor {
    graph: Graph,
    state: EditorState,
    history: History,
}
impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    /// Creates an empty editor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            state: EditorState::default(),
            history: History::default(),
        }
    }
    /// Returns the graph.
    #[must_use]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
    /// Returns mutable graph data.
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }
    /// Returns interaction state.
    #[must_use]
    pub fn state(&self) -> &EditorState {
        &self.state
    }
    /// Returns mutable interaction state.
    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }
    /// Applies a command and records an undo snapshot.
    pub fn execute(&mut self, command: Command) -> Result<(), ConnectionError> {
        let before = self.graph.clone();
        match command {
            Command::AddNode(node) => {
                self.graph.add_node(node);
            }
            Command::RemoveNodes(ids) => {
                for id in ids {
                    self.graph.remove_node(id);
                }
            }
            Command::MoveNodes { ids, delta } => {
                for id in ids {
                    if let Some(node) = self.graph.node_mut(id) {
                        node.position.x += delta.x;
                        node.position.y += delta.y;
                    }
                }
            }
            Command::RemoveEdges(ids) => {
                for id in ids {
                    self.graph.remove_edge(id);
                }
            }
            Command::Connect { from, to, style } => {
                self.graph.connect(from, to, style)?;
            }
            Command::ResizeNode { id, size } => {
                if let Some(node) = self.graph.node_mut(id) {
                    node.size = Size::new(size.width.max(80.0), size.height.max(48.0));
                }
            }
            Command::SetCollapsed { id, collapsed } => {
                if let Some(node) = self.graph.node_mut(id) {
                    node.collapsed = collapsed;
                }
            }
        }
        if self.graph != before {
            self.push_undo(before);
        }
        Ok(())
    }
    /// Undoes the most recent command.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.history.undo.pop() else {
            return false;
        };
        self.history.redo.push(self.graph.clone());
        self.graph = previous;
        true
    }
    /// Redoes the most recently undone command.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.history.redo.pop() else {
            return false;
        };
        self.history.undo.push(self.graph.clone());
        self.graph = next;
        true
    }
    /// Selects a node, optionally retaining existing selection.
    pub fn select_node(&mut self, id: NodeId, additive: bool) {
        if !additive {
            self.state.selected_nodes.clear();
            self.state.selected_edges.clear();
        }
        self.state.selected_nodes.insert(id);
    }
    /// Selects an edge, optionally retaining existing selection.
    pub fn select_edge(&mut self, id: EdgeId, additive: bool) {
        if !additive {
            self.state.selected_nodes.clear();
            self.state.selected_edges.clear();
        }
        self.state.selected_edges.insert(id);
    }
    /// Clears selection.
    pub fn clear_selection(&mut self) {
        self.state.selected_nodes.clear();
        self.state.selected_edges.clear();
    }
    /// Selects all nodes intersecting a box in graph coordinates.
    pub fn select_box(&mut self, bounds: Bounds, additive: bool) {
        if !additive {
            self.state.selected_nodes.clear();
            self.state.selected_edges.clear();
        }
        self.state.selected_nodes.extend(
            self.graph
                .nodes()
                .filter(|node| node.bounds().intersects(bounds))
                .map(|node| node.id),
        );
    }
    /// Connects two ports through the undoable command system.
    pub fn connect(
        &mut self,
        from: PortRef,
        to: PortRef,
        style: EdgeStyle,
    ) -> Result<(), ConnectionError> {
        self.execute(Command::Connect { from, to, style })
    }
    /// Resizes a node through the undoable command system.
    pub fn resize_node(&mut self, id: NodeId, size: Size) -> Result<(), ConnectionError> {
        self.execute(Command::ResizeNode { id, size })
    }
    /// Sets a node's collapsed state through the undoable command system.
    pub fn set_collapsed(&mut self, id: NodeId, collapsed: bool) -> Result<(), ConnectionError> {
        self.execute(Command::SetCollapsed { id, collapsed })
    }
    /// Deletes selected nodes and edges.
    pub fn delete_selected(&mut self) {
        let nodes = self.state.selected_nodes.iter().copied().collect();
        let edges = self.state.selected_edges.iter().copied().collect();
        let _ = self.execute(Command::RemoveNodes(nodes));
        let _ = self.execute(Command::RemoveEdges(edges));
        self.clear_selection();
    }
    /// Copies selected nodes.
    pub fn copy_selected(&mut self) -> usize {
        self.state.clipboard = self
            .state
            .selected_nodes
            .iter()
            .filter_map(|id| self.graph.node(*id).cloned())
            .collect();
        self.state.clipboard.len()
    }
    /// Pastes copied nodes with a small offset and selects them.
    pub fn paste(&mut self) -> Vec<NodeId> {
        let nodes = self.state.clipboard.clone();
        let mut ids = Vec::new();
        self.clear_selection();
        for mut node in nodes {
            node.position.x += 32.0;
            node.position.y += 32.0;
            let id = self.graph.add_node(node);
            ids.push(id);
            self.state.selected_nodes.insert(id);
        }
        ids
    }
    fn push_undo(&mut self, graph: Graph) {
        self.history.undo.push(graph);
        if self.history.undo.len() > self.history.limit {
            self.history.undo.remove(0);
        }
        self.history.redo.clear();
    }
}

/// Built-in visual configuration.
#[derive(Clone, Debug)]
pub struct FlowStyle {
    /// Canvas background.
    pub background: Color32,
    /// Grid line color.
    pub grid: Color32,
    /// Node fill.
    pub node_fill: Color32,
    /// Selected node fill.
    pub selected_node_fill: Color32,
    /// Node header color.
    pub header: Color32,
    /// Port color.
    pub port: Color32,
    /// Edge color.
    pub edge: Color32,
}
impl Default for FlowStyle {
    fn default() -> Self {
        Self {
            background: Color32::from_gray(24),
            grid: Color32::from_gray(38),
            node_fill: Color32::from_gray(48),
            selected_node_fill: Color32::from_rgb(48, 76, 112),
            header: Color32::from_rgb(36, 92, 132),
            port: Color32::from_rgb(120, 190, 220),
            edge: Color32::from_gray(150),
        }
    }
}

/// Result of one rendering pass.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlowResponse {
    /// Whether graph data changed.
    pub changed: bool,
    /// Whether selection changed.
    pub selection_changed: bool,
    /// Whether undo or redo was handled.
    pub history_changed: bool,
    /// Node under the latest secondary click, if any.
    pub context_node: Option<NodeId>,
}

/// Custom node painting hook.
pub trait NodeRenderer {
    /// Paints a node inside the supplied screen-space rectangle.
    fn draw(&self, painter: &Painter, node: &Node, rect: Rect, selected: bool);
}

/// Custom port painting hook.
pub trait PortRenderer {
    /// Paints a port at its screen-space center.
    fn draw(&self, painter: &Painter, port: &Port, center: Pos2, input: bool);
}

/// A ready-to-use egui node editor.
pub struct FlowEditor {
    /// Stateful graph and interaction controller.
    pub editor: Editor,
    /// Visual style.
    pub style: FlowStyle,
    /// Whether the minimap is drawn.
    pub show_minimap: bool,
    node_renderer: Option<Box<dyn NodeRenderer>>,
    port_renderer: Option<Box<dyn PortRenderer>>,
}
impl Default for FlowEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowEditor {
    /// Creates an empty editor widget.
    #[must_use]
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            style: FlowStyle::default(),
            show_minimap: true,
            node_renderer: None,
            port_renderer: None,
        }
    }
    /// Installs an application-defined node renderer.
    pub fn set_node_renderer(&mut self, renderer: impl NodeRenderer + 'static) {
        self.node_renderer = Some(Box::new(renderer));
    }
    /// Installs an application-defined port renderer.
    pub fn set_port_renderer(&mut self, renderer: impl PortRenderer + 'static) {
        self.port_renderer = Some(Box::new(renderer));
    }
    /// Renders the editor and processes pointer and keyboard interaction.
    pub fn show(&mut self, ui: &mut Ui) -> FlowResponse {
        let available = ui.available_size().max(Vec2::splat(64.0));
        let (canvas, canvas_response) = ui.allocate_exact_size(available, Sense::click_and_drag());
        let painter = ui.painter_at(canvas);
        painter.rect_filled(canvas, 0.0, self.style.background);
        self.draw_grid(&painter, canvas);
        let mut output = FlowResponse::default();
        let ids: Vec<NodeId> = self.editor.graph.nodes.keys().copied().collect();
        for edge in self.editor.graph.edges.values() {
            self.draw_edge(&painter, canvas, edge);
        }
        for id in ids {
            let Some(node) = self.editor.graph.node(id).cloned() else {
                continue;
            };
            let rect = self.node_rect(node.bounds(), canvas);
            let response = ui.interact(
                rect,
                Id::new(("egui-flow-node", id.0)),
                Sense::click_and_drag(),
            );
            if response.clicked() {
                self.editor
                    .select_node(id, ui.input(|input| input.modifiers.shift));
                output.selection_changed = true;
            }
            if response.secondary_clicked() {
                output.context_node = Some(id);
            }
            if response.dragged() {
                let delta = response.drag_delta() / self.editor.state.zoom();
                let _ = self.editor.execute(Command::MoveNodes {
                    ids: vec![id],
                    delta: Point::new(delta.x, delta.y),
                });
                output.changed = true;
            }
            self.draw_node(
                &painter,
                canvas,
                &node,
                self.editor.state.selected_nodes.contains(&id),
            );
        }
        if canvas_response.dragged() && ui.input(|input| input.pointer.middle_down()) {
            self.editor.state.pan += canvas_response.drag_delta();
        }
        if ui.input(|input| input.key_pressed(Key::Delete)) {
            self.editor.delete_selected();
            output.changed = true;
        }
        if ui.input(|input| input.modifiers.command && input.key_pressed(Key::Z)) {
            output.history_changed = if ui.input(|input| input.modifiers.shift) {
                self.editor.redo()
            } else {
                self.editor.undo()
            };
        }
        if ui.input(|input| input.modifiers.command && input.key_pressed(Key::C)) {
            self.editor.copy_selected();
        }
        if ui.input(|input| input.modifiers.command && input.key_pressed(Key::V)) {
            self.editor.paste();
            output.changed = true;
        }
        if self.show_minimap {
            self.draw_minimap(&painter, canvas);
        }
        output
    }
    /// Fits the graph into the supplied viewport.
    pub fn fit_to_view(&mut self, viewport: Rect) {
        if let Some(bounds) = self.editor.graph.bounds() {
            self.editor.state.fit_to_view(bounds, viewport);
        }
    }
    fn node_rect(&self, bounds: Bounds, viewport: Rect) -> Rect {
        let min = self.editor.state.world_to_screen(bounds.min, viewport);
        Rect::from_min_size(
            min,
            Vec2::new(
                (bounds.max.x - bounds.min.x) * self.editor.state.zoom(),
                (bounds.max.y - bounds.min.y) * self.editor.state.zoom(),
            ),
        )
    }
    fn draw_grid(&self, painter: &Painter, viewport: Rect) {
        if !self.editor.state.grid_enabled {
            return;
        }
        let spacing = self.editor.state.grid_size * self.editor.state.zoom();
        if spacing < 4.0 {
            return;
        }
        let mut x = viewport.left() + self.editor.state.pan.x.rem_euclid(spacing);
        while x < viewport.right() {
            painter.line_segment(
                [
                    Pos2::new(x, viewport.top()),
                    Pos2::new(x, viewport.bottom()),
                ],
                Stroke::new(1.0_f32, self.style.grid),
            );
            x += spacing;
        }
        let mut y = viewport.top() + self.editor.state.pan.y.rem_euclid(spacing);
        while y < viewport.bottom() {
            painter.line_segment(
                [
                    Pos2::new(viewport.left(), y),
                    Pos2::new(viewport.right(), y),
                ],
                Stroke::new(1.0_f32, self.style.grid),
            );
            y += spacing;
        }
    }
    fn draw_node(&self, painter: &Painter, viewport: Rect, node: &Node, selected: bool) {
        let rect = self.node_rect(node.bounds(), viewport);
        if let Some(renderer) = &self.node_renderer {
            renderer.draw(painter, node, rect, selected);
            return;
        }
        painter.rect_filled(
            rect,
            6.0,
            if selected {
                self.style.selected_node_fill
            } else {
                self.style.node_fill
            },
        );
        let header = Rect::from_min_size(
            rect.min,
            Vec2::new(
                rect.width(),
                (24.0 * self.editor.state.zoom()).min(rect.height()),
            ),
        );
        painter.rect_filled(header, 6.0, self.style.header);
        painter.text(
            header.left_top() + Vec2::new(8.0, 4.0),
            Align2::LEFT_TOP,
            &node.title,
            FontId::proportional(14.0 * self.editor.state.zoom().sqrt()),
            Color32::WHITE,
        );
        if node.collapsed {
            return;
        }
        let inputs: Vec<&Port> = node
            .ports
            .iter()
            .filter(|port| port.direction == PortDirection::Input)
            .collect();
        let outputs: Vec<&Port> = node
            .ports
            .iter()
            .filter(|port| port.direction == PortDirection::Output)
            .collect();
        for (index, port) in inputs.iter().enumerate() {
            self.draw_port(painter, rect, port, index, inputs.len(), true);
        }
        for (index, port) in outputs.iter().enumerate() {
            self.draw_port(painter, rect, port, index, outputs.len(), false);
        }
    }
    fn draw_port(
        &self,
        painter: &Painter,
        rect: Rect,
        port: &Port,
        index: usize,
        count: usize,
        input: bool,
    ) {
        let step = rect.height() / (count.max(1) as f32 + 1.0);
        let position = Pos2::new(
            if input { rect.left() } else { rect.right() },
            rect.top() + step * (index as f32 + 1.0),
        );
        if let Some(renderer) = &self.port_renderer {
            renderer.draw(painter, port, position, input);
        } else {
            painter.circle_filled(position, 5.0, self.style.port);
        }
        painter.text(
            position + Vec2::new(if input { 9.0 } else { -9.0 }, -8.0),
            if input {
                Align2::LEFT_TOP
            } else {
                Align2::RIGHT_TOP
            },
            &port.name,
            FontId::proportional(11.0),
            Color32::LIGHT_GRAY,
        );
    }
    fn draw_edge(&self, painter: &Painter, viewport: Rect, edge: &Edge) {
        let Some(from) = self.port_position(edge.from, viewport) else {
            return;
        };
        let Some(to) = self.port_position(edge.to, viewport) else {
            return;
        };
        let stroke = Stroke::new(
            2.0_f32,
            if self.editor.state.selected_edges.contains(&edge.id) {
                Color32::YELLOW
            } else {
                self.style.edge
            },
        );
        match edge.style {
            EdgeStyle::Straight => {
                painter.line_segment([from, to], stroke);
            }
            EdgeStyle::Orthogonal => {
                let middle = Pos2::new((from.x + to.x) / 2.0, from.y);
                painter.line_segment([from, middle], stroke);
                painter.line_segment([middle, Pos2::new(middle.x, to.y)], stroke);
                painter.line_segment([Pos2::new(middle.x, to.y), to], stroke);
            }
            EdgeStyle::Bezier => {
                let mut points = Vec::with_capacity(17);
                for index in 0..=16 {
                    let t = index as f32 / 16.0;
                    let u = 1.0 - t;
                    let p2 = Pos2::new(from.x + 60.0, from.y);
                    let p3 = Pos2::new(to.x - 60.0, to.y);
                    let point = from.to_vec2() * (u * u * u)
                        + p2.to_vec2() * (3.0 * u * u * t)
                        + p3.to_vec2() * (3.0 * u * t * t)
                        + to.to_vec2() * (t * t * t);
                    points.push(Pos2::new(point.x, point.y));
                }
                painter.line(points, stroke);
            }
        }
    }
    fn port_position(&self, reference: PortRef, viewport: Rect) -> Option<Pos2> {
        let node = self.editor.graph.node(reference.node)?;
        let port = node.port(reference.port)?;
        let count = node
            .ports
            .iter()
            .filter(|item| item.direction == port.direction)
            .count();
        let index = node
            .ports
            .iter()
            .filter(|item| item.direction == port.direction)
            .position(|item| item.id == port.id)?;
        let rect = self.node_rect(node.bounds(), viewport);
        let step = rect.height() / (count.max(1) as f32 + 1.0);
        Some(Pos2::new(
            if port.direction == PortDirection::Input {
                rect.left()
            } else {
                rect.right()
            },
            rect.top() + step * (index as f32 + 1.0),
        ))
    }
    fn draw_minimap(&self, painter: &Painter, viewport: Rect) {
        let Some(bounds) = self.editor.graph.bounds() else {
            return;
        };
        let size = Vec2::new(180.0, 120.0);
        let rect = Rect::from_min_size(viewport.right_bottom() - size - Vec2::splat(12.0), size);
        painter.rect_filled(rect, 4.0, Color32::from_black_alpha(180));
        let scale = ((rect.width() - 12.0) / (bounds.max.x - bounds.min.x).max(1.0))
            .min((rect.height() - 12.0) / (bounds.max.y - bounds.min.y).max(1.0));
        for node in self.editor.graph.nodes.values() {
            let min = rect.min
                + Vec2::splat(6.0)
                + Vec2::new(
                    (node.position.x - bounds.min.x) * scale,
                    (node.position.y - bounds.min.y) * scale,
                );
            painter.rect_filled(
                Rect::from_min_size(
                    min,
                    Vec2::new(node.size.width * scale, node.size.height * scale)
                        .max(Vec2::splat(2.0)),
                ),
                1.0,
                self.style.header,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graph_rejects_wrong_direction() {
        let mut graph = Graph::new();
        let input = graph.add_node(Node::new("Input", Point::default()).input("in", "text"));
        let output =
            graph.add_node(Node::new("Output", Point::new(200.0, 0.0)).output("out", "text"));
        let input_port = graph.node(input).unwrap().ports[0].id;
        let output_port = graph.node(output).unwrap().ports[0].id;
        assert_eq!(
            graph.connect(
                PortRef {
                    node: input,
                    port: input_port
                },
                PortRef {
                    node: output,
                    port: output_port
                },
                EdgeStyle::Straight
            ),
            Err(ConnectionError::WrongDirection)
        );
    }
    #[test]
    fn graph_round_trips_json() {
        let mut graph = Graph::new();
        graph.add_node(Node::new("Node", Point::new(12.0, 8.0)).output("value", "number"));
        let json = graph.to_json().unwrap();
        assert_eq!(Graph::from_json(&json).unwrap(), graph);
    }
    #[test]
    fn history_restores_graph() {
        let mut editor = Editor::new();
        editor
            .execute(Command::AddNode(Node::new("Node", Point::default())))
            .unwrap();
        assert_eq!(editor.graph.nodes.len(), 1);
        assert!(editor.undo());
        assert!(editor.graph.nodes.is_empty());
        assert!(editor.redo());
        assert_eq!(editor.graph.nodes.len(), 1);
    }
}
