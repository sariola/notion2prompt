# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of notion2prompt CLI tool
- High-performance parallel fetching with work-stealing queues
- Complete child database embedding support
- Recursive page/database content fetching
- Multiple output formats (file, clipboard, pipe)
- Configurable Handlebars templates
- Type-safe domain models with newtypes
- Robust error handling with automatic retries
- String interning and object pooling optimizations
- Comprehensive CLI interface with validation
- Professional repository setup with CI/CD workflows

### Changed
- Repository cleaned up for publishing readiness
- Dead code removed (35+ compiler warnings resolved)
- Duplicate response files consolidated
- Unused dependencies removed (tree-sitter, md5, ammonia, base64)
- File organization improved with proper .gitignore patterns

### Technical Details
- Three-stage pipeline architecture (Fetch → Transform → Output)
- Work-stealing concurrency with priority scheduling
- Critical priority for child database fetches
- Atomic work tracking for completion guarantee
- Pure functional formatting with visitor patterns
- Comprehensive type safety with domain-specific newtypes

### Dependencies
- tokio for async runtime
- reqwest for HTTP client
- handlebars for template rendering
- clap for CLI parsing
- serde for serialization
- crossbeam for work-stealing
- rayon for parallel processing

## Release Process

Releases follow semantic versioning:
- **MAJOR**: Breaking changes to CLI or API
- **MINOR**: New features, non-breaking changes
- **PATCH**: Bug fixes, documentation updates

### Release Checklist
- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md with release notes
- [ ] Run full test suite: `cargo test`
- [ ] Run linting: `cargo clippy`
- [ ] Format code: `cargo fmt`
- [ ] Tag release: `git tag v0.1.0`
- [ ] Push tag: `git push origin v0.1.0`
- [ ] GitHub Actions will automatically build and publish

### Upcoming Features (v0.2.0)
- Docker support
- Configuration file support
- Additional output formats (JSON, YAML)
- Plugin system for custom formatters
- Performance metrics and profiling
- Database schema caching
- Incremental fetch support