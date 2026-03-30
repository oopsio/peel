---
icon: lucide/rocket
---

# The Peel Programming Language

Welcome to the official documentation for **Peel**, a modern, fast, and safe systems and applications programming language. 

Peel combines the safety guarantees of functional programming with the performance of systems languages, offering an ergonomic syntax that empowers developers to write reliable, thread-safe, and highly concurrent code. Let's peel back the layers and build incredible software.

## Philosophy

- **Thread-safe by default**: Built-in constructs like `Arc` and `Mutex` allow for fearless concurrency.
- **Predictable Performance**: No hidden allocations or invisible overhead.
- **Expressive Syntax**: Pattern matching, advanced type inference, and traits enable you to describe complex logic cleanly.
- **Batteries-included tooling**: Comes with `pepm`, a modern dependency management and build tool.

## Installation

You can install the Peel toolchain via the official installer script:

```powershell
irm https://raw.githubusercontent.com/oopsio/peel/master/tools/installation/install.ps1 | iex
```

or for Bash users

```bash
curl -fsSL https://raw.githubusercontent.com/oopsio/peel/master/tools/installation/install.sh | sh
```

Verify your installation:

```bash
peel --version
```

## First Steps

Start learning Peel by creating your first project:

```bash
pepm new my_project
cd my_project
pepm run
```

Dive into the [Basics](basics/variables.md) to understand the core concepts.
