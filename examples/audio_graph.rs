use egui_flow_editor::{Graph, Node, Point};

fn main() {
    let mut graph = Graph::new();
    graph.add_node(Node::new("Microphone", Point::new(20.0, 20.0)).output("samples", "audio"));
    graph.add_node(
        Node::new("Filter", Point::new(260.0, 20.0))
            .input("samples", "audio")
            .output("filtered", "audio"),
    );
    graph.add_node(Node::new("Speaker", Point::new(500.0, 20.0)).input("samples", "audio"));
    println!("Audio graph contains {} nodes", graph.nodes().count());
}
