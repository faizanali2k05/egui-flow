use egui_flow_editor::{Graph, Node, Point};

fn main() {
    let mut graph = Graph::new();
    graph.add_node(
        Node::new("Add", Point::new(20.0, 20.0))
            .input("left", "number")
            .input("right", "number")
            .output("sum", "number"),
    );
    graph.add_node(Node::new("Print", Point::new(300.0, 20.0)).input("value", "number"));
    println!("Visual program contains {} nodes", graph.nodes().count());
}
