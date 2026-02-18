# notion2prompt

A high-performance Rust CLI tool that converts Notion pages and databases into structured prompts for AI models. Transform your Notion content into perfectly formatted AI inputs with recursive fetching, parallel processing, and smart child database embedding.

> **Quick Demo**: `notion2prompt https://notion.so/your-page` → AI-ready structured prompt in seconds

## Why notion2prompt?

Tired of manually copying and formatting Notion content for AI tools? This CLI automates the entire process, preserving structure, handling complex databases, and generating prompts optimized for AI models like Claude and GPT.

## Features

- ⚡ **High Performance**: Parallel fetching with work-stealing queues and priority-based scheduling
- 🗃️ **Complete Database Support**: Automatically embeds child databases within parent pages
- = **Recursive Fetching**: Configurable depth limits with cycle detection
- 📝 **Multiple Output Formats**: Markdown with customizable Handlebars templates
- 📋 **Flexible Output**: File, clipboard, or pipe output options
- 🔒 **Type Safety**: Strong typing with domain-specific newtypes
- 🚀 **Smart Caching**: String interning and object pooling for efficiency
- 🛡️ **Robust Error Handling**: Automatic retries with exponential backoff

## Quick Start

### 1. Install
```bash
git clone https://github.com/sariola/notion2prompt.git
cd notion2prompt
cargo build --release
```

### 2. Get Notion API Key
1. Go to [https://www.notion.so/my-integrations](https://www.notion.so/my-integrations)
2. Click "New integration" → Name it → Copy the token

### 3. Share Your Page
1. Open your Notion page → Click "Share" → Invite your integration

### 4. Convert!
```bash
export NOTION_API_KEY="secret_your_api_key_here"
./target/release/notion2prompt https://notion.so/your-page --clipboard
```

## Installation

### Python (via uv)

The fastest way to install — pre-compiled wheels for Linux, macOS, and Windows:

```bash
# Install as a tool (adds `notion2prompt` to your PATH)
uv tool install notion2prompt

# Or run directly without installing
uvx notion2prompt https://notion.so/your-page
```

### Python (via pip)

```bash
pip install notion2prompt
```

### Python Library Usage

```python
import asyncio
import notion2prompt

# One-shot: fetch and render
prompt = asyncio.run(notion2prompt.fetch_and_render(
    "https://www.notion.so/your-page-id",
    api_key="secret_...",
))
print(prompt)

# Two-stage: fetch first, render later with different templates
content = asyncio.run(notion2prompt.fetch_content("your-page-id"))
prompt = notion2prompt.render_content(content, template="claude-xml")
```

### Rust (via Cargo)

```bash
# Install directly from the repository
cargo install --git https://github.com/sariola/notion2prompt.git
```

### From Source

Requires Rust (latest stable) and Cargo:

```bash
git clone https://github.com/sariola/notion2prompt.git
cd notion2prompt
cargo build --release
# Binary at target/release/notion2prompt
```

## Getting Started

### 1. Obtain a Notion API Key

1. Go to [https://www.notion.so/my-integrations](https://www.notion.so/my-integrations)
2. Click "New integration"
3. Give it a name and select the workspace
4. Copy the "Internal Integration Token" (starts with `secret_`)

### 2. Share Pages with Your Integration

1. Open the Notion page you want to convert
2. Click "Share" in the top right
3. Invite your integration by name
4. The integration needs read access

### 3. Set Your API Key

```bash
export NOTION_API_KEY="secret_your_api_key_here"
```

### 4. Run notion2prompt

```bash
# Using a Notion URL
notion2prompt https://www.notion.so/Your-Page-Title-abc123

# Using just the page ID
notion2prompt abc123def456ghi789

# With options
notion2prompt abc123 --output my-prompt.md --verbose
```

## Usage

```
notion2prompt [OPTIONS] <NOTION_INPUT>

Arguments:
  <NOTION_INPUT>  Notion page/database ID or URL

Options:
  -o, --output <FILE>           Output file path
  -t, --template <NAME>         Template name [default: claude-xml]
  -b, --clipboard              Copy output to clipboard
  -p, --pipe                   Output to stdout for piping
  -d, --depth <N>              Max recursion depth [default: 5]
  -l, --limit <N>              Max items to fetch [default: 1000]
  -v, --verbose                Enable verbose output
      --content-dir <DIR>      Content directory path
      --instruction <TEXT>     Additional instructions
      --parse-child-pages      Parse child pages recursively
      --separate-child-page    Keep child pages separate
      --always-fetch-databases Always fetch database content
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

```bash
# Convert a page and save to file
notion2prompt https://notion.so/My-Page-123abc -o my-page.md

# Copy to clipboard with custom depth
notion2prompt 123abc --clipboard --depth 10

# Pipe to another command
notion2prompt 123abc --pipe | wc -l

# Use verbose mode for debugging
notion2prompt 123abc --verbose

# Parse child pages with custom template
notion2prompt 123abc --parse-child-pages --template default
```

## Templates

Templates use Handlebars syntax and are stored in the `templates/` directory. The default template is `claude-xml.hbs`.

### Available Templates

- `claude-xml.hbs` - Optimized for Claude AI with XML-style sections
- `default.hbs` - Simple markdown output

### Template Variables

Available variables in your templates:
- `{{absolute_content_path}}` - Full path to content
- `{{source_tree}}` - File tree representation

*Note: Check template files for the complete list of available variables*

## Architecture

notion2prompt uses a three-stage pipeline architecture optimized for performance and reliability:

### 1. Fetch Stage (Async)
Parallel content retrieval with intelligent work distribution:
- Work-stealing queues with priority scheduling
- Child database fetches get critical priority
- Atomic work tracking ensures completion
- Rate limiting and automatic retries

### 2. Transform Stage (Sync)
Pure data transformation without side effects:
- Visitor pattern for block processing
- Embedded database formatting
- Rich text and mention handling
- Property value formatting

### 3. Output Stage (Async)
Template rendering and output generation:
- Handlebars template rendering
- File writing, clipboard, or pipe output
- Path validation and sandboxing

## Key Features Explained

### Child Database Embedding

Child databases (inline databases in Notion) are automatically detected and embedded within their parent pages, ensuring complete content capture. This critical feature uses priority scheduling to guarantee child content is fetched before worker threads terminate.

### Work-Stealing Concurrency

The parallel fetcher uses work-stealing to optimize CPU utilization. Multiple workers process API requests concurrently, automatically balancing load by stealing work from busy queues.

### Type Safety

Every domain concept is wrapped in a newtype for compile-time validation:

```rust
let id = NotionId::parse("https://notion.so/page-123")?;  // Validates format
let key = ApiKey::parse("secret_abc123")?;  // Ensures proper prefix
```

## Performance

Performance characteristics (typical results may vary based on network conditions and content complexity):
- **Network throughput**: Limited by Notion API rate limits
- **CPU processing**: Optimized for high-throughput data transformation
- **Memory efficiency**: String interning and object pooling reduce memory usage
- **Concurrency**: Work-stealing parallelism improves utilization over sequential processing

## Configuration

### Environment Variables

- `NOTION_API_KEY` - Your Notion API key (required)
- `RUST_LOG` - Log level (debug, info, warn, error)

### Default Limits

- Default recursion depth: 5 levels
- Default item limit: 1000 items
- Maximum safe recursion depth: 50 levels

## Development

### Building

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- <notion-id>
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Type check
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test api::

# Run integration tests
cargo test --test integration

# Run with output
cargo test -- --nocapture
```

## Troubleshooting

### Common Issues

1. **"Invalid API key"**
   - Ensure your key starts with `secret_`
   - Verify the environment variable is set: `echo $NOTION_API_KEY`

2. **"Object not found"**
   - Verify the page is shared with your integration
   - Check the page/database ID is correct
   - Ensure your integration has read access

3. **"Rate limit exceeded"**
   - The tool automatically retries with exponential backoff
   - Try reducing parallelism if issues persist

4. **Missing child databases**
   - Ensure depth is greater than 0
   - Use `--verbose` to see fetch details
   - Check that child databases are properly linked in Notion

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug notion2prompt <id> --verbose

# For development debugging, logs may be written to temporary files
# Check your system's temp directory if debug logging is enabled
```

## Architecture Details

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`cargo test`)
4. Format code (`cargo fmt`)
5. Commit changes (`git commit -m 'Add amazing feature'`)
6. Push to branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

This project is built on the shoulders of excellent open-source libraries. Thanks to all their maintainers and contributors:

### Core Runtime & Networking
- [tokio](https://github.com/tokio-rs/tokio) by the Tokio team — async runtime powering all concurrent operations
- [reqwest](https://github.com/seanmonstar/reqwest) by Sean McArthur (@seanmonstar) — ergonomic HTTP client
- [hyper](https://github.com/hyperium/hyper) by Sean McArthur — the HTTP implementation beneath reqwest

### Concurrency & Data Structures
- [crossbeam](https://github.com/crossbeam-rs/crossbeam) by the Crossbeam team — work-stealing queues and concurrent primitives
- [rayon](https://github.com/rayon-rs/rayon) by Niko Matsakis & Josh Stone — parallel iterators
- [dashmap](https://github.com/xacrimon/dashmap) by Acrimon (@xacrimon) — concurrent hash map
- [parking_lot](https://github.com/Amanieu/parking_lot) by Amanieu d'Antras — fast mutex and RwLock
- [im](https://github.com/bodil/im-rs) by Bodil Stokke (@bodil) — immutable data structures

### Serialization & Templating
- [serde](https://github.com/serde-rs/serde) by David Tolnay (@dtolnay) & Erick Tryzelaar — serialization framework
- [serde_json](https://github.com/serde-rs/json) by David Tolnay — JSON support
- [handlebars-rust](https://github.com/sunng87/handlebars-rust) by Ning Sun (@sunng87) — template engine

### CLI & Error Handling
- [clap](https://github.com/clap-rs/clap) by the clap contributors — command-line argument parsing
- [thiserror](https://github.com/dtolnay/thiserror) by David Tolnay — derive macro for error types
- [anyhow](https://github.com/dtolnay/anyhow) by David Tolnay — flexible error handling

### Notion API
- [notion-client](https://github.com/jhamill34/notion-client) by jhamill34 — Notion API client for Rust

### Python Bindings
- [PyO3](https://github.com/PyO3/pyo3) by the PyO3 team — Rust ↔ Python FFI bindings
- [maturin](https://github.com/PyO3/maturin) by the PyO3 team — build system for Rust Python extensions
- [pyo3-async-runtimes](https://github.com/PyO3/pyo3-async-runtimes) — async/await bridge between Rust and Python

### Utilities
- [chrono](https://github.com/chronotope/chrono) by the Chronotope team — date and time handling
- [regex](https://github.com/rust-lang/regex) by Andrew Gallant (@BurntSushi) — regular expressions
- [indexmap](https://github.com/indexmap-rs/indexmap) by the indexmap contributors — insertion-ordered map
- [lru](https://github.com/jeromefroe/lru-rs) by Jerome Froelich — LRU cache for API response caching
- [arboard](https://github.com/1Password/arboard) by 1Password — cross-platform clipboard access
- [uuid](https://github.com/uuid-rs/uuid) by the uuid contributors — UUID generation and parsing
- [url](https://github.com/servo/rust-url) by the Servo project — URL parsing and validation