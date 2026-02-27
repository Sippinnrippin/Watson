# Contributing to Watson

> By contributing to Watson, you agree to the terms outlined in [CLA.md](CLA.md).

## How to Contribute

### Reporting Bugs

1. Check if the issue already exists
2. Open a new issue with:
   - Clear title
   - Steps to reproduce
   - Expected vs actual behavior
   - Your environment (OS, Rust version, etc.)

### Suggesting Features

1. Open a discussion first to gauge interest
2. Submit a PR with the implementation
3. Include tests and documentation

### Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Make** your changes
4. **Test** your changes: `cargo build --release && cargo test`
5. **Commit** with clear messages (follow [conventional commits](https://www.conventionalcommits.org/))
6. **Push** to your fork
7. **Submit** a Pull Request
8. **Wait** for review - all PRs require approval before merging

## Code Standards

- Run `cargo clippy` before submitting
- Ensure code compiles without warnings
- Add tests for new features
- Update documentation for any changes

## License

By contributing, you agree that your contributions will be licensed under the terms of [CLA.md](CLA.md).
