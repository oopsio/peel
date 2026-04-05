# Contributing to Peel

Welcome to the Peel project! We're excited that you're interested in helping out. As a community-driven programming language, your contributions—whether they are bug reports, documentation improvements, or new features—are highly valued.

## Ways to Contribute

1.  **Reporting Bugs:** If you find a bug, please use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md) to let us know.
2.  **Suggesting Features:** Have a great idea for Peel? Open a [Feature Request](.github/ISSUE_TEMPLATE/feature_request.md).
3.  **Improving Documentation:** Documentation is just as important as code. Feel free to submit PRs for any typos or missing information.
4.  **Submitting Code:** Ready to write some Rust? Check out the "Developing Peel" section below.

## Developing Peel

### Prerequisites

- **Rust Toolchain**: You'll need the latest stable version of Rust installed.
- **VS Code**: Recommended for development with the Peel extension.

### Setup

1.  Clone the repository:
    ```bash
    git clone https://github.com/oopsio/peel.git
    cd peel
    ```
2.  Build the project:
    ```bash
    cargo build
    ```
3.  Run tests:
    ```bash
    cargo test
    ```

## Development Workflow

1.  **Fork** the repository and create your branch from `master`.
2.  **Code** your changes. Ensure you follow the existing style and add tests where applicable.
3.  **Commit** with clear, descriptive messages.
4.  **Push** to your fork and submit a **Pull Request**.

### Pull Request Guidelines

- Describe the problem being solved or the feature being added.
- Include relevant issue numbers.
- Ensure all CI checks pass before the maintainers review.

## Code of Conduct

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

Thank you for contributing to Peel!
