# Contributing to RustProxy

Thank you for your interest in contributing to **RustProxy**! We welcome contributions that improve reliability, performance, protocol compatibility, and defensive security workflows.

---

## Code of Conduct

All contributors and participants are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## How to Contribute

### 1. Reporting Bugs
- Search existing GitHub issues to ensure the bug hasn't already been reported.
- Open a new issue including:
  - RustProxy version (`rustproxy --version`).
  - Operating system and desktop environment.
  - Clear steps to reproduce the issue.
  - Expected vs. actual behavior.

### 2. Suggesting Enhancements
- Propose new features or protocol enhancements by opening an issue labeled `enhancement`.
- Clearly explain the problem solved, proposed user workflow, and technical architecture.

### 3. Pull Request Process
1. Fork the repository and create your branch from `main`:
   ```bash
   git checkout -b feature/my-feature-name
   ```
2. Implement your changes adhering to existing architectural boundaries.
3. Ensure all tests and lint checks pass:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cd desktop && npm run build
   ```
4. Write unit and integration tests covering your changes.
5. Submit your pull request with a descriptive title and summary of changes.

---

## License

By contributing to RustProxy, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
