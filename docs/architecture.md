# Architecture

The crate has four layers. `Graph`, `Node`, `Port`, and `Edge` are the serializable data model and do not contain egui values. `EditorState` owns the viewport, selection, snapping, clipboard, and coordinate transforms. `FlowEditor` translates that state into egui paint and input operations. `Editor` applies commands through graph snapshots so undo and redo do not require application data to implement a hidden trait.

Applications can use only the model, only the editor controller, or the complete widget. `ConnectionValidator` is the boundary for domain-specific typing rules. `NodeRenderer` and `PortRenderer` let a host replace the default visual layer without changing persistence.
