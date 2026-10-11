use egui_flow::{Graph, Node, Point};

fn main() {
    let mut graph = Graph::new();
    for index in 0..1_000 {
        graph.add_node(Node::new(
            format!("Node {index}"),
            Point::new((index % 40) as f32 * 220.0, (index / 40) as f32 * 120.0),
        ));
    }
    println!("Stress graph contains {} nodes", graph.nodes().count());
}
