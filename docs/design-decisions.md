# Design decisions

- Stable numeric IDs make edges compact and allow applications to persist references without using UI coordinates.
- `Point` and `Size` remain independent of egui so serialized documents are portable across front ends.
- History stores graph snapshots. This uses more memory than an inverse-operation log, but it keeps undo behavior deterministic for arbitrary application edits and provides a clear initial 100-entry bound.
- Port `kind` values are application-defined strings; the crate validates direction while `ConnectionValidator` controls domain compatibility.
- JSON is an optional integration. Serde derives remain on the model so applications can select another format without an egui dependency in their documents.
