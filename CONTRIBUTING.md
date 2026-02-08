# Contributing to zParse

First off, thanks for taking the time to contribute! 🎉

The following is a set of guidelines for contributing to zParse. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Getting Started

1. Fork the repository
2. Create a new branch for your changes
3. Make your changes
4. Submit a pull request

## Pull Request Process

### Prerequisites

- **Issues First**: Each pull request MUST be associated with an issue

  - If an issue doesn't exist, create one first
  - Reference the issue in your PR using GitHub's keywords (e.g., "Fixes #123")

- **Code Formatting**: All code must be formatted using `rustfmt`

  ```bash
  cargo +nightly fmt --all
  ```

- **Clippy Lints**: ALl code must pass clippy lints

  ```bash
  cargo clippy --all-features --all-targets -- -D warnings
  ```

- **Commit Messages**: Must follow the [Conventional Commits](https://www.conventionalcommits.org/) format

  ```
  type(scope): description

  [optional body]

  [optional footer]
  ```

  Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

  Examples:

  ```
  feat(parser): add support for nested arrays
  fix(converter): handle null values in JSON to TOML conversion
  docs(README): update installation instructions
  ```

### Requirements

1. Update tests

   - Add new tests for new features
   - Modify existing tests for changes to existing functionality

2. Update documentation

   - Update relevant README sections
   - Add/update doc comments for public API changes
   - Update CHANGELOG.md following Keep a Changelog format

3. Pass all checks

   - All tests must pass: `cargo test`
   - Benchmarks must run successfully: `cargo bench`
   - No clippy warnings: `cargo clippy --all-features -- -D warnings`
   - No unsafe code: `#![forbid(unsafe_code)]`

## Development Workflow

1. **Create Issue**

   - Describe the problem/feature
   - Get feedback from maintainers
   - Wait for approval/assignment

2. **Development**

   ```bash
   # Update your fork
   git checkout master
   git pull upstream master

   # Create feature branch
   git checkout -b feat/your-feature

   # Make changes
   # Write tests
   # Update docs
   ```

3. **Testing**

   ```bash
   # Run tests
   cargo test --all-features

   # Run clippy
   cargo clippy --all-features -- -D warnings

   # Run benchmarks
   cargo bench

   # Run fuzzing (requires nightly)
   cargo +nightly fuzz run json_parser
   cargo +nightly fuzz run toml_parser
   ```

4. **Submit PR**
   - Follow PR template
   - Respond to review comments
   - Keep your PR updated with the target branch

## Code Style

- Follow Rust standard conventions
- Use meaningful variable names
- Comment complex logic
- Keep functions focused and small
- Use type safety where possible
- Avoid unnecessary allocations
- Handle errors appropriately

## Questions?

Feel free to:

- Open an issue for discussion
- Ask questions in PR comments

## License

By contributing, you agree that your contributions will be licensed under the GPL-3.0 License.
