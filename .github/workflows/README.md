# GitHub Actions Workflows for Matrix Language

This directory contains comprehensive GitHub Actions workflows for continuous integration, testing, security, and deployment of the Matrix Language project.

## Workflows Overview

### 1. CI (`ci.yml`)
**Triggers**: Push/PR to main branches  
**Purpose**: Core continuous integration pipeline
- **Check**: Validates code compilation with all features
- **Test**: Runs complete test suite including doc tests  
- **Lints**: Enforces code quality with rustfmt and clippy
- **Security**: Runs cargo-audit for vulnerability scanning
- **Coverage**: Generates code coverage reports via tarpaulin

### 2. Cross Platform (`cross-platform.yml`)
**Triggers**: Push/PR to main branches  
**Purpose**: Ensures compatibility across platforms and Rust versions
- Tests on Ubuntu, Windows, and macOS
- Tests with stable and beta Rust versions
- Validates minimal dependency versions
- Tests feature combinations (default, no-default, jit)
- Runs performance benchmarks

### 3. Release (`release.yml`)
**Triggers**: Git tags starting with 'v'  
**Purpose**: Automated release management
- Creates GitHub releases
- Builds binaries for Linux, Windows, and macOS
- Optionally publishes to crates.io
- Strips binaries and prepares release assets

### 4. Documentation (`docs.yml`)
**Triggers**: Push/PR to main branches  
**Purpose**: Documentation generation and validation
- Builds and deploys Rust documentation to GitHub Pages
- Validates README.md links
- Performs spell checking on source files and documentation

### 5. Dependencies (`dependencies.yml`)
**Triggers**: Weekly schedule + dependency file changes  
**Purpose**: Dependency management and security
- Checks for outdated dependencies with cargo-outdated
- Detects unused dependencies with cargo-udeps
- Validates dependency licenses
- Runs supply chain security checks with cargo-deny
- Automatically creates PRs for dependency updates

### 6. Performance (`performance.yml`)
**Triggers**: Push/PR to main branches  
**Purpose**: Performance monitoring and analysis
- Runs performance test suites
- Memory usage analysis with Valgrind
- Compile time tracking
- Binary size analysis and bloat detection

### 7. Nightly (`nightly.yml`)
**Triggers**: Daily schedule + manual dispatch  
**Purpose**: Testing with experimental Rust features
- Tests with nightly Rust toolchain
- Experimental feature validation
- Future compatibility checks
- Miri undefined behavior detection
- Sanitizer testing (AddressSanitizer, ThreadSanitizer)

## Security Configuration

### `deny.toml`
Configuration for cargo-deny that enforces:
- **Advisories**: Denies known vulnerabilities
- **Licenses**: Allows only approved licenses (MIT, Apache-2.0, BSD, etc.)
- **Bans**: Prevents problematic dependencies
- **Sources**: Restricts dependency sources to trusted registries

## Required Secrets

To fully utilize all workflows, configure these GitHub repository secrets:

| Secret | Purpose | Required For |
|--------|---------|--------------|
| `CODECOV_TOKEN` | Code coverage reporting | CI workflow |
| `CRATES_IO_TOKEN` | Publishing to crates.io | Release workflow |
| `GITHUB_TOKEN` | Built-in token | Most workflows (auto-provided) |

## Workflow Features

### Caching Strategy
All workflows use multi-layered caching:
- Cargo registry cache
- Git dependencies cache  
- Target directory cache
- Keyed by OS and Cargo.lock hash

### System Dependencies
Automatically installs required system packages:
- **Linux**: pkg-config, xcb libraries, SSL
- **macOS**: pkg-config via Homebrew
- **Windows**: Native dependencies

### Error Handling
- Non-critical workflows use `continue-on-error: true`
- Comprehensive error reporting
- Graceful degradation for optional features

### Performance Optimizations
- Parallel job execution where possible
- Conditional workflow execution
- Efficient caching strategies
- Target-specific optimizations

## Usage Examples

### Triggering Workflows

```bash
# Trigger CI on feature branch
git push origin feature/new-feature

# Create a release
git tag v1.0.0
git push origin v1.0.0

# Manual nightly test
gh workflow run nightly.yml
```

### Local Testing

```bash
# Run the same checks locally
cargo check --all-targets --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo audit
```

## Maintenance

### Adding New Workflows
1. Create `.yml` file in `.github/workflows/`
2. Follow existing naming conventions
3. Include proper caching and dependencies
4. Test thoroughly with different triggers

### Updating Dependencies
Dependencies are automatically updated weekly, but manual updates:
```bash
cargo update
cargo test --all-features  # Ensure tests pass
```

### Monitoring
- Check Actions tab for workflow results
- Review security advisories in dependency workflow
- Monitor performance trends in performance workflow
- Check documentation deployment status

This comprehensive CI/CD setup ensures code quality, security, and cross-platform compatibility while providing automated releases and thorough testing coverage.
