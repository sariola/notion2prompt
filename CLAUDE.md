# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

notion2prompt is a Rust CLI tool that converts Notion pages and databases into structured prompts for AI models. It fetches content recursively from the Notion API and formats it using Handlebars templates.

This document serves as both a development guide and an architectural preservation document. The codebase embraces functional programming patterns while maintaining pragmatic performance considerations.

## Development Commands

### Build and Run
```bash
# Development build
cargo build

# Release build  
cargo build --release

# Run with Notion URL or ID (requires NOTION_API_KEY env var)
cargo run -- <notion-url-or-id>

# Run with options
cargo run -- <notion-id> -o output.txt --verbose
cargo run -- <notion-id> --clipboard --template claude-xml
cargo run -- <notion-id> --pipe > output.md
cargo run -- <notion-id> --depth 10 --limit 1000
cargo run -- <notion-id> --always-fetch-databases
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Type check without building
cargo check

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration

# Run ignored tests (requires API key)
cargo test --ignored
```

### Linting Commands to Run Before Committing
```bash
cargo fmt
cargo clippy
```

## Library Support

This project can be used both as a CLI tool and as a library. The library exposes key types and functionality through `src/lib.rs` for external consumption.

## Architecture Overview

### Pipeline Architecture
The application follows a strict 3-stage pipeline pattern in `src/main.rs`:

1. **Fetch Stage** (`fetch_stage()`) - Async operation that recursively fetches Notion content
   - Uses parallel work-stealing with priority queue
   - Embeds child databases into parent blocks
   - Tracks work completion with atomic counters
2. **Transform Stage** (`transform_stage()`) - Synchronous transformation of fetched data into structured format
   - Applies visitor pattern for block processing
   - Handles embedded database formatting
3. **Output Stage** (`output_stage()`) - Generates final output using templates and handles file/clipboard/pipe operations
   - Separates planning from execution

Each stage returns a specific result type that flows into the next stage, ensuring type safety and clear data flow.

### Core Design Principles
- **Functional Programming First**: Immutability and pure functions
- **Type Safety Through Newtypes**: Domain concepts wrapped for compile-time validation
- **Effect Separation**: I/O operations clearly separated from business logic
- **Resource Efficiency**: String interning, object pooling, lazy evaluation
- **Work-Stealing Concurrency**: Optimal CPU utilization with priority queues

## Architectural Preservation

### Core Patterns to Preserve
1. **Three-Stage Pipeline Integrity**
   - NEVER merge stages or blur boundaries
   - Fetch → Transform → Output must remain distinct
   - Each stage must have clear input/output types
   - Stage results flow unidirectionally
   - Data transformations must be explicit and traceable

2. **Data-Oriented Design Principles**
   - Data structures must accurately reflect the domain (NotionObject, Block, Database)
   - Prioritize data locality - keep related data together (BlockCommon pattern)
   - Minimize data copies - use references and borrowing effectively
   - Transform data, don't manage state - focus on input → output flows
   - Make data dependencies explicit through type signatures

3. **Functional Programming Principles**
   - All formatting functions must remain pure and referentially transparent
   - State mutations only allowed in visitor pattern implementations (prefer pure visitor)
   - Prefer immutable data structures (use im-rc collections)
   - Side effects isolated to clearly marked boundaries (IO at the edges)
   - Functions should focus on value transformation, not state management
   - Embrace higher-order functions and function composition
   - Use Result/Option types for control flow instead of exceptions
   - Leverage iterator combinators over imperative loops
   - Apply monadic patterns (Result chaining) for error propagation
   - Ensure functions are total - handle all possible inputs
   - Prefer expressions over statements
   - Use currying and partial application where it improves composability

4. **Type Safety Requirements**
   - Every string parameter must have a newtype wrapper
   - Parse, don't validate - constructors must ensure validity
   - No stringly-typed APIs
   - Phantom types for compile-time state tracking
   - Types should encode domain constraints (e.g., NotionId format validation)
   - Pre/post-conditions enforced through type constructors
   - Leverage algebraic data types (enums) to make invalid states unrepresentable
   - Use generics and trait bounds to enforce compile-time guarantees
   - Push validation to system boundaries (API entry points)
   - Prefer type-level programming where it enhances safety without obscuring intent
   - Ensure referential transparency - same inputs always produce same outputs

5. **Concurrency Model Protection**
   - Work-stealing queue must maintain priority ordering
   - Atomic counters for work tracking must be SeqCst
   - Never use blocking operations in async contexts
   - Child database fetches always get Critical priority
   - Leverage immutable data structures to avoid race conditions
   - Use message passing over shared mutable state where possible

### Anti-Patterns to Avoid
- ❌ Mixing I/O with business logic
- ❌ Using String instead of domain types (NotionId, ApiKey, etc.)
- ❌ Synchronous fetching in any form
- ❌ Direct mutation of shared state without synchronization
- ❌ Circular dependencies between modules
- ❌ God objects or manager classes
- ❌ Stringly-typed APIs (use enums and newtypes)
- ❌ Unwrap() in production code
- ❌ Public fields on structs (use accessor methods)
- ❌ Cross-layer imports (e.g., formatting importing from api)
- ❌ Hidden data dependencies or implicit state transitions
- ❌ Excessive data copying instead of borrowing
- ❌ Shared mutable state without clear ownership
- ❌ Blocking operations in async contexts
- ❌ Complex abstractions that hide data flow
- ❌ Functions with unclear value production
- ❌ Type signatures that don't express intent
- ❌ Missing precondition validation in constructors
- ❌ Hidden complexity behind simple interfaces
- ❌ Violations of referential transparency
- ❌ Unidiomatic Rust patterns (manual error handling, etc.)
- ❌ Performance anti-patterns (unnecessary allocations, cloning)
- ❌ Partial functions that panic on valid inputs
- ❌ Implicit dependencies or global state access
- ❌ Time-dependent functions without explicit context
- ❌ Leaky abstractions exposing implementation details

### Module Organization

#### Core Modules
- **api/** - Notion API interaction
  - `client.rs` - HTTP client with automatic rate limiting and retries
  - `parallel_fetcher.rs` - Work-stealing parallel fetcher with priority queue
  - `parser.rs` - Pure functions for parsing API responses
  - `types.rs` - API-specific types (FetchContext, FetchResult, etc.)
  - `object_graph.rs` - Immutable graph for parent-child relationships
  - `concurrent_queue.rs` - Work-stealing queue with priority support
  - `fetch_queue.rs` - Work items with priority levels (Critical/High/Normal/Low)
  - `connection_pool.rs` - Connection pooling for HTTP clients
  - `optimized_client.rs` - Optimized client implementations
  - `pagination.rs` & `simple_pagination.rs` - Pagination handling strategies
  - `responses.rs`, `responses_new.rs`, `responses_old.rs` - API response types
  - `notion_client_adapter.rs` - Client adapter pattern

- **model/** - Domain models split into submodules
  - `mod.rs` - Main model definitions (NotionObject enum)
  - `blocks.rs` - Block types with BlockCommon pattern
  - `properties.rs` - Property types and values
  - `common.rs` - Shared types (titles, parents)
  - Child database blocks include optional embedded database field

- **formatting/** - Content transformation (data-oriented)
  - `core.rs` - Recursive block formatting with immutable FormatContext
  - `state.rs` - Immutable formatting state transitions
  - `visitor.rs` & `pure_visitor.rs` - Visitor pattern for markdown generation
  - `properties/` - Property formatters organized by type
  - `databases/` - Table building with LinkConfig for URL resolution
  - `rich_text/` - Text formatting with mention handlers
  - `registry.rs` - Template and formatter registry
  - `streaming.rs` - Streaming output support
  - `parallel.rs`, `parallel_context.rs`, `parallel_rayon.rs` - Parallel formatting
  - `direct_template.rs` - Direct template rendering
  - `effects.rs` - Effect handling for formatting
  - `pure_examples.rs` - Pure functional examples
  - `tests_child_database.rs` - Child database formatting tests

#### Supporting Modules
- **presentation/** - Abstract output interface (NotionPresenter trait)
- **analytics/** - Content statistics and metrics
- **output/** - File I/O with separation of planning and execution
  - `plan.rs` - Output planning phase
  - `writer.rs` - File writing operations
  - `clipboard.rs` - Clipboard operations
  - `paths.rs` - Path handling utilities
  - `pure.rs` - Pure output functions
  - `types.rs` - Output type definitions
- **error/** - Comprehensive error handling
  - `mod.rs` - Main error types using thiserror
  - `domain.rs` - Domain-specific errors
- **error_context.rs** - Error context utilities
- **error_recovery.rs** - Error recovery strategies
- **performance/** - Resource management and optimizations
  - `string_builder.rs` - Efficient string building
  - `object_pool.rs` - Object pooling for memory efficiency
  - `interning.rs` - String interning for deduplication
  - `lazy.rs` - Lazy evaluation patterns
  - `memory.rs` - Memory management utilities
  - `efficient_blocks.rs` - Optimized block representations
  - `zero_copy.rs` - Zero-copy optimizations
- **type_safety/** - Advanced type safety patterns
  - `phantom_types.rs` - Phantom type implementations
  - `builders.rs` - Type-safe builders
  - `invariants.rs` - Type invariant enforcement
  - `examples.rs` - Type safety examples
- **types/** - Domain types and type safety
  - `domain_types.rs` - Core domain type definitions
  - `ids.rs` - ID type definitions (NotionId, etc.)
  - `properties.rs` - Property type definitions
  - `collections.rs` - Collection type utilities
  - `colors.rs` - Color type definitions
  - `compat.rs` - Compatibility types
- **di/** - Dependency injection container
  - `mod.rs` - DI container setup
  - `services.rs` - Service definitions and traits
  - `implementations.rs` - Service implementations
- **config.rs** - Application configuration and CLI parsing
- **constants.rs** - Application constants
- **result.rs** - Result type utilities

### Module Access Rules

#### Layer Dependencies (Strict)
```
main.rs
   ↓ (can import from)
api/, model/, formatting/, output/, di/
   ↓
types/, error/, performance/
   ↓
std library only
```

#### Forbidden Dependencies
- api/ → formatting/ (must go through model/)
- formatting/ → api/ (unidirectional flow)
- output/ → api/ (only through DI)
- Any module → main.rs (no circular deps)
- model/ → api/ (model is pure domain)
- types/ → any module except std (foundation layer)

#### Cross-Module Communication
- Use DI container for service dependencies
- Pass data through well-defined interfaces
- No direct module coupling except through traits
- All async operations confined to api/ and output/
- Pure business logic in model/ and formatting/

### Architectural Invariants (MUST PRESERVE)

1. **Pipeline Integrity** - Each stage must have distinct input/output types
2. **Type Safety Invariants** - All string-like domain concepts must be newtypes
3. **Concurrency Invariants** - Child databases MUST have Critical priority
4. **Memory Safety Invariants** - Recursion must be bounded, cycle detection required
5. **Error Handling Invariants** - No panic! in production code paths
6. **API Boundary Invariants** - Public APIs must be stable
