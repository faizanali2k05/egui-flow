# Performance notes

The model uses ordered maps for deterministic serialization and stable iteration. Rendering visits each visible node and edge once per frame; applications with very large documents should add viewport culling around `FlowEditor` or use the model APIs to render only visible nodes. The `graph_1000` Criterion benchmark measures graph construction only, so its result must not be interpreted as a frame-rate guarantee.
