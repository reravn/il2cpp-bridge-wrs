# Contributing to il2cpp-bridge-rs

Thanks for your interest in contributing! Here's how you can help.

## Reporting Issues

Found a bug or have a suggestion? [Open an issue](https://github.com/Batchhh/il2cpp-bridge-rs/issues/new/choose) using the appropriate template:

- **Bug Report** — for unexpected behavior or errors
- **Feature Request** — for new features or improvements
- **Question** — for usage questions

## Getting Started

1. Fork the repository
2. Clone your fork:
  ```bash
   git clone https://github.com/Batchhh/il2cpp-bridge-rs.git
   cd il2cpp-bridge-rs
  ```
3. Create a branch for your changes:
  ```bash
   git checkout -b feature/your-feature-name
  ```
4. Make sure it builds:
  ```bash
   make build
  ```

## Development

### Building

See the [README](README.md#building) for platform-specific build commands.

### Code Style

- Follow standard Rust conventions (`rustfmt` defaults)
- Run `make clippy` before submitting and address any warnings
- Keep unsafe code to a minimum and document why it's needed

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in imperative mood (e.g., "Add", "Fix", "Update")

## Submitting Changes

1. Make sure your code compiles on at least one target platform
2. Run `make clippy` and fix any warnings
3. Push your branch and open a Pull Request
4. Describe what your PR does and link any related issues

## Pull Request Guidelines

- Keep PRs focused — one feature or fix per PR
- Update documentation if your change affects the public API
- Add context in the PR description for non-obvious changes

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).