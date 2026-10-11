use egui_flow_editor::{Graph, Node, Point};

fn main() {
    let mut graph = Graph::new();
    graph.add_node(Node::new("Idle", Point::new(20.0, 20.0)).output("transition", "event"));
    graph.add_node(
        Node::new("Running", Point::new(280.0, 20.0))
            .input("transition", "event")
            .output("transition", "event"),
    );
    graph.add_node(Node::new("Stopped", Point::new(540.0, 20.0)).input("transition", "event"));
    println!("State machine contains {} states", graph.nodes().count());
}
