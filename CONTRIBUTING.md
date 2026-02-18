# Contributing to notion2prompt

Thank you for your interest in contributing to notion2prompt! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites
- Rust 1.70.0 or later
- A Notion account and API key for testing (get one at [notion.so/developers](https://www.notion.so/developers))

### Setup
1. Fork and clone the repository
2. Run `cargo build` to ensure everything compiles
3. Set up your Notion API key: `export NOTION_API_KEY=your_key_here`
4. Run tests: `cargo test`

## Development Workflow

### Code Quality Standards
Before submitting any changes, please ensure:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run all tests
cargo test

# Check compilation
cargo check
```

### Architecture Guidelines
This project follows strict architectural principles. Please read [CLAUDE.md](./CLAUDE.md) for detailed guidance on:
- Functional programming patterns
- Type safety requirements  
- Module organization rules
- Concurrency patterns

Key principles:
- **Three-stage pipeline**: Fetch → Transform → Output stages must remain distinct
- **Pure functions**: All formatting functions must be referentially transparent
- **Type safety**: Use newtypes for all string-like domain concepts
- **No blocking operations** in async contexts

### Testing
- Write tests for new functionality
- Integration tests go in `tests/` directory
- Unit tests can be inline with modules
- Use `cargo test --ignored` for tests requiring API keys

### Documentation
- Add doc comments (`///`) for all public APIs
- Update README.md if adding new features
- Follow existing documentation patterns

## Submitting Changes

### Pull Request Process
1. Create a feature branch from `main`
2. Make your changes following the code quality standards
3. Add tests for new functionality
4. Update documentation as needed
5. Submit a pull request with a clear description

### Pull Request Guidelines
- Use clear, descriptive commit messages
- Keep changes focused and atomic
- Reference any related issues
- Ensure all CI checks pass

### Commit Message Format
```
type: short description

Longer description if needed, explaining the why and what.

Fixes #123
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`

## Code Review

All contributions go through code review. Reviewers will check:
- Adherence to architectural principles
- Code quality and testing
- Performance implications
- Documentation completeness

## Getting Help

- Check existing [issues](https://github.com/flowaicom/notion2prompt/issues)
- Read the [architecture documentation](./CLAUDE.md)
- Start discussions for major changes

## Recognition

Contributors will be recognized in release notes and the project README. Thank you for helping make notion2prompt better!