# Security model

The library is an in-process editor. It treats graph metadata as opaque strings and performs no code execution, network access, or filesystem access. Deserialization is not a trust boundary by itself: applications loading files from users should cap input size, handle errors, and validate metadata before passing it to plugins or command runners.
