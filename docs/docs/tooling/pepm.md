# The Peel Package Manager (`pepm`)

`pepm` (pronounced _pep-um_) is the official package manager and build system for the Peel programming language. It handles dependency resolution and project management.

## Creating a New Project

To start a new project, navigate to your desired directory and run:

```bash
pepm init
```

This will create a `peel.toml` manifest in the root of your project with your project name.

## The Manifest file: `peel.toml`

The `peel.toml` file contains configuration for your project and dependencies.

```toml
name = "my_project"
version = "0.1.0"

[dependencies]
http = "1.0.2"
```

## Installing Dependencies

To install all dependencies listed in your `peel.toml`, simply run:

```bash
pepm install
```

This will download the required modules into the `.peel/modules` directory.

## Adding a Dependency

To quickly add a new dependency and install it simultaneously:

```bash
pepm add regex
pepm add regex --version 1.2.3
```

## Registries

`pepm` fetches packages from the default official registry hosted on GitHub.

## Building and Running

You can compile your project using the `peel` compiler (separate tool):

```bash
peel run main.pel
```
