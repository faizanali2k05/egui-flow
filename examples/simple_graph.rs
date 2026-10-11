use egui_flow::{EdgeStyle, Graph, Node, Point, PortRef};

fn main() {
    let mut graph = Graph::new();
    let source =
        graph.add_node(Node::new("Source", Point::new(32.0, 40.0)).output("value", "number"));
    let sink = graph.add_node(Node::new("Sink", Point::new(280.0, 40.0)).input("value", "number"));
    let from = PortRef {
        node: source,
        port: graph.node(source).expect("source exists").ports[0].id,
    };
    let to = PortRef {
        node: sink,
        port: graph.node(sink).expect("sink exists").ports[0].id,
    };
    graph
        .connect(from, to, EdgeStyle::Bezier)
        .expect("compatible ports");
    println!(
        "{} nodes, {} edge",
        graph.nodes().count(),
        graph.edges().count()
    );
}
