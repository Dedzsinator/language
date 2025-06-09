# Contributing to Matrix Language

Thank you for your interest in contributing to Matrix Language! This document provides guidelines and information for contributors.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project adheres to a code of conduct adapted from the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you are expected to uphold this code.

### Our Pledge
We are committed to making participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

## Getting Started

### Prerequisites
- **Rust**: Latest stable version (install via [rustup](https://rustup.rs/))
- **Git**: Version control
- **System Dependencies**: See [README.md](README.md) for platform-specific requirements

### First Steps
1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/matrix-lang.git`
3. Set up the development environment
4. Run tests to ensure everything works
5. Look for issues labeled `good-first-issue`

## Development Setup

### Building the Project
```bash
# Clone the repository
git clone https://github.com/your-username/matrix-lang.git
cd matrix-lang

# Install system dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y pkg-config libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev

# Build the project
cargo build

# Run tests
cargo test

# Run with all features
cargo build --all-features
cargo test --all-features
```

### VSCode Setup
If you use VSCode, the repository includes configuration files:
- `.vscode/settings.json` - Editor settings optimized for Rust
- `.vscode/tasks.json` - Common build and test tasks
- `.vscode/launch.json` - Debug configurations

### Recommended Tools
```bash
# Install useful development tools
cargo install cargo-watch    # Auto-rebuild on file changes
cargo install cargo-audit    # Security vulnerability scanner
cargo install cargo-tarpaulin # Code coverage
cargo install cargo-deny     # License and dependency checker
cargo install cargo-outdated # Check for outdated dependencies
```

## How to Contribute

### Types of Contributions
We welcome various types of contributions:

#### Code Contributions
- **Bug fixes**: Fix existing issues
- **Features**: Implement new language features
- **Performance improvements**: Optimize existing code
- **Refactoring**: Improve code structure and maintainability

#### Non-Code Contributions
- **Documentation**: Improve or add documentation
- **Testing**: Add test cases and improve test coverage
- **Bug reports**: Report issues with detailed information
- **Feature requests**: Suggest new features
- **Community support**: Help other users and contributors

### Finding Issues to Work On
- Browse [open issues](https://github.com/your-username/matrix-lang/issues)
- Look for `good-first-issue` labels for newcomers
- Check `help-wanted` labels for areas needing assistance
- Issues labeled `bug` are good for fixing problems
- Issues labeled `enhancement` are good for adding features

### Before Starting Work
1. **Check existing issues and PRs** to avoid duplicate work
2. **Comment on the issue** to let others know you're working on it
3. **Ask questions** if you need clarification
4. **Discuss approach** for large changes before implementing

## Pull Request Process

### 1. Create a Branch
```bash
# Create a new branch for your work
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Changes
- Write clear, focused commits
- Follow the coding standards
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes
```bash
# Run all tests
cargo test --all-features

# Run specific test suites
cargo test comprehensive_tests
cargo test lexer_tests
cargo test parser_tests

# Check formatting and linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
cargo audit
```

### 4. Commit Guidelines
- Write clear, descriptive commit messages
- Use the imperative mood ("Add feature" not "Added feature")
- Reference issue numbers when applicable

```
feat: add matrix multiplication operator

- Implement * operator for matrix types
- Add comprehensive tests for matrix operations
- Update documentation with examples

Fixes #123
```

### 5. Submit Pull Request
- **Title**: Clear, descriptive title
- **Description**: Explain what you changed and why
- **Testing**: Describe how you tested your changes
- **Issues**: Link related issues

### PR Template
```markdown
## Summary
Brief description of changes

## Changes Made
- Change 1
- Change 2
- Change 3

## Testing
- [ ] All existing tests pass
- [ ] Added new tests for new functionality
- [ ] Manual testing performed

## Related Issues
Fixes #issue_number

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review of code completed
- [ ] Documentation updated (if applicable)
- [ ] Tests added/updated (if applicable)
```

## Coding Standards

### Rust Style
- Follow the [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
- Use `cargo fmt` for automatic formatting
- Use `cargo clippy` for linting

### Code Organization
```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library root (if applicable)
├── lexer/            # Lexical analysis
├── parser/           # Syntax parsing
├── ast/              # Abstract syntax tree
├── eval/             # Interpreter/evaluator
├── types/            # Type system
├── physics/          # Physics simulation
├── gui/              # GUI components
└── tests/            # Test modules
```

### Documentation Standards
- **Public APIs**: Must have rustdoc comments
- **Modules**: Should have module-level documentation
- **Complex logic**: Should have inline comments
- **Examples**: Include usage examples in docs

```rust
/// Parses a Matrix Language expression into an AST node.
///
/// This function takes a string containing Matrix Language code and
/// returns the corresponding abstract syntax tree representation.
///
/// # Arguments
/// * `input` - The source code to parse
///
/// # Returns
/// A `Result` containing the parsed `Expression` or a `ParseError`
///
/// # Examples
/// ```
/// use matrix_lang::parser::parse_expression;
/// 
/// let ast = parse_expression("2 + 3 * 4").unwrap();
/// assert_eq!(ast.evaluate(), 14);
/// ```
pub fn parse_expression(input: &str) -> Result<Expression, ParseError> {
    // Implementation
}
```

### Error Handling
- Use `Result<T, E>` for recoverable errors
- Use custom error types with `thiserror`
- Provide meaningful error messages
- Include context and suggestions when possible

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token '{token}' at line {line}, column {column}")]
    UnexpectedToken { token: String, line: usize, column: usize },
    
    #[error("Unterminated string literal starting at line {line}")]
    UnterminatedString { line: usize },
}
```

## Testing Guidelines

### Test Organization
- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test component interactions
- **End-to-end tests**: Test complete workflows
- **Performance tests**: Verify performance characteristics

### Test Categories
```bash
# Run different test categories
cargo test unit_tests
cargo test integration_tests
cargo test comprehensive_tests
cargo test performance_tests
```

### Writing Tests
- **Clear test names**: Describe what is being tested
- **Arrange-Act-Assert**: Structure tests clearly
- **Edge cases**: Test boundary conditions and error cases
- **Performance**: Include performance regression tests

```rust
#[test]
fn test_lexer_handles_unterminated_string() {
    let input = "\"unterminated string";
    let lexer = Lexer::new(input);
    let result = lexer.tokenize();
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LexError::UnterminatedString { .. }));
}
```

### Coverage Requirements
- Aim for >80% code coverage
- Critical paths should have 100% coverage
- Use `cargo tarpaulin` to measure coverage

## Documentation

### Types of Documentation
1. **API Documentation**: Rustdoc for public APIs
2. **User Guide**: How to use Matrix Language
3. **Developer Guide**: How to contribute and extend
4. **Examples**: Practical usage examples

### Documentation Standards
- Write for your audience (users vs. contributors)
- Include practical examples
- Keep documentation up to date
- Use clear, concise language

### Building Documentation
```bash
# Build and open documentation
cargo doc --all-features --open

# Check for documentation warnings
cargo doc --all-features 2>&1 | grep warning
```

## Community

### Communication Channels
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and community discussion
- **Pull Requests**: Code review and collaboration

### Getting Help
- **New contributor questions**: Comment on issues or start a discussion
- **Technical questions**: Use GitHub discussions
- **Bug reports**: Create a detailed issue
- **Feature ideas**: Start with a discussion, then create an issue

### Code Review Process
- All changes require review before merging
- Reviews focus on:
  - Correctness and functionality
  - Code quality and maintainability
  - Test coverage
  - Documentation completeness
  - Performance implications

### Recognition
- Contributors are recognized in release notes
- Significant contributions may be highlighted
- First-time contributors receive special acknowledgment

## Release Process

### Versioning
We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible functionality additions
- **PATCH**: Backward-compatible bug fixes

### Release Cycle
- Regular releases every 4-6 weeks
- Patch releases as needed for critical bugs
- Pre-releases for testing major changes

Thank you for contributing to Matrix Language! Your efforts help make this project better for everyone.
