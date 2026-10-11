use egui_flow_editor::{Graph, Node, Point};

fn main() {
    let mut graph = Graph::new();
    graph.add_node(Node::new("Prompt", Point::new(20.0, 20.0)).output("text", "string"));
    graph.add_node(
        Node::new("Model", Point::new(260.0, 20.0))
            .input("prompt", "string")
            .output("answer", "string"),
    );
    graph.add_node(Node::new("Memory", Point::new(500.0, 20.0)).input("answer", "string"));
    println!("AI workflow contains {} nodes", graph.nodes().count());
}
