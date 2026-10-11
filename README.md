# egui-flow

`egui-flow` is a node-based editor widget for [egui](https://github.com/emilk/egui). Its graph model is independent of egui, so the same nodes and edges can be validated, persisted, tested, and rendered in different front ends.

Use it for workflow builders, audio routing, state machines, visual programming tools, and AI pipelines. It is a UI component rather than a workflow runtime, layout solver, or access-control system; applications still own execution, persistence policy, and authorization.

## Installation

```toml
[dependencies]
egui-flow = "0.1"
```

The default `persistence` feature adds JSON graph persistence through `serde_json`. Disable default features when an application supplies its own persistence format.

## 30-second example

```rust
use egui_flow::{EdgeStyle, FlowEditor, Graph, Node, Point, PortRef};

let mut graph = Graph::new();
let source = graph.add_node(Node::new("Source", Point::new(20.0, 20.0)).output("value", "number"));
let sink = graph.add_node(Node::new("Sink", Point::new(280.0, 20.0)).input("value", "number"));
let from = PortRef { node: source, port: graph.node(source).unwrap().ports[0].id };
let to = PortRef { node: sink, port: graph.node(sink).unwrap().ports[0].id };
graph.connect(from, to, EdgeStyle::Bezier)?;

let mut editor = FlowEditor::new();
*editor.editor.graph_mut() = graph;
// In an egui panel: editor.show(ui);
# Ok::<(), egui_flow::ConnectionError>(())
```

The complete runnable model examples are in `examples/`; an application normally calls `editor.show(ui)` from its egui panel.

## Features

The model provides typed input/output ports, connection validation hooks, three edge styles, stable IDs, node metadata, JSON persistence, selection, multi-selection, box selection APIs, node resizing and collapsing, copy/paste, keyboard deletion, undo/redo, grid snapping, viewport transforms, fit-to-view, minimap rendering, context-click reporting, and custom node/port painting hooks. The benchmark covers constructing a 1,000-node graph.

## Tutorial: custom nodes and persistence

Create a node with `Node::new`, add ports with `.input` and `.output`, and place application metadata in `node.metadata`. Keep runtime state outside the graph model. After editing, persist with `Graph::to_json()` and restore with `Graph::from_json()`:

```rust
# #[cfg(feature = "persistence")]
fn save_and_restore(graph: &egui_flow::Graph) -> Result<egui_flow::Graph, Box<dyn std::error::Error>> {
    let json = graph.to_json()?;
    Ok(egui_flow::Graph::from_json(&json)?)
}
```

Implement `ConnectionValidator` when port `kind` strings need application-specific compatibility rules. Implement `NodeRenderer` or `PortRenderer` when the default painter does not match the application’s visual language.

## Compatibility and security

The MSRV is Rust 1.88 and the crate targets platforms supported by egui. Graph JSON is application data: validate its size and contents before loading untrusted files, and do not treat node metadata as executable code. The library uses no unsafe Rust and does not perform I/O, networking, or code execution.

For large graphs, keep expensive application work outside `show`, use stable metadata rather than cloning external resources into nodes, and benchmark the graph shape that matters to your product. The included stress example is a construction smoke test, not a claim about a particular frame rate.

## Alternatives and stability

Use egui’s low-level painter APIs for a one-off diagram, a specialized graph library for automatic layout, or a full workflow engine when execution and scheduling are the primary problem. `egui-flow` follows semantic versioning; the 0.1 API may receive additive changes and will document breaking changes before 1.0.

See [architecture](docs/architecture.md), [design decisions](docs/design-decisions.md), [security](docs/security.md), and [performance](docs/performance.md). The project is dual-licensed under MIT or Apache-2.0.
