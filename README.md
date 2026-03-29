# Peel Language

Peel is a modern, high-performance programming language built with Rust. It features an async-first runtime, robust object-oriented programming, and a rich, ECMAScript-inspired standard library.

## Features

- **Async Native**: Built-in support for `async`/`await` powered by `tokio`.
- **Structured OOP**: Defined types with `struct` and shared behaviors via `impl` blocks.
- **Rich StdLib**: Comprehensive sets for `console`, `Math`, and `JSON`.
- **Integrated Tooling**: Comes with **pepm**, a modern package manager.
- **Fast Performance**: Compiled and optimized in Rust for maximum execution speed.

## Quick Start

### Building the Project
Use the unified build script to compile the runtime and package manager:
```bash
bun scripts/build.js
```
The binaries will be placed in the `dist/` directory.

### Running a Script
```bash
./dist/peel run examples/hello.pel
```

### Using the Package Manager (pepm)
Initialize a new project and add dependencies:
```bash
./dist/pepm init
./dist/pepm add some-package
```

## Project Structure

- `src/`: Core Peel runtime, lexer, parser, and interpreter.
- `crates/pepm/`: The Peel Package Manager implementation.
- `stdlib/`: Built-in native modules and prototypes.
- `examples/`: Guided demonstrations of async, OOP, and standard library features.

## License
Peel is licensed under the [MIT License](LICENSE).
