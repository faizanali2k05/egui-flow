use criterion::{Criterion, black_box, criterion_group, criterion_main};
use egui_flow::{Graph, Node, Point};

fn build_graph() -> Graph {
    let mut graph = Graph::new();
    for index in 0..1_000 {
        graph.add_node(Node::new(
            format!("Node {index}"),
            Point::new((index % 40) as f32 * 220.0, (index / 40) as f32 * 120.0),
        ));
    }
    graph
}

fn graph_benchmark(c: &mut Criterion) {
    c.bench_function("build 1000-node graph", |bench| {
        bench.iter(|| black_box(build_graph()))
    });
}

criterion_group!(benches, graph_benchmark);
criterion_main!(benches);
