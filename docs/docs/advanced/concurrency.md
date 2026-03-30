# Concurrency in Peel

Peel is designed with concurrency in mind, prioritizing safety and predictability. The core runtime focuses on robust synchronous execution as we refine our asynchronous engine.

## Core Concepts

- **Thread-safe**: Built with the intent of eliminating common data races.
- **Async Awareness**: The runtime is built on an asynchronous foundation (`tokio`), paving the way for advanced async primitives.

For current high-performance I/O or parallel workloads, we recommend structuring applications into modular services that interact via standard protocols.
