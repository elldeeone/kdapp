# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

kdapp is a Rust framework for building high-frequency, interactive decentralized applications (dApps) on the Kaspa blockDAG. The project uses a unique "Episode" pattern that enables rapid development of on-chain interactive applications like games, betting systems, and multi-participant protocols.

**Status**: Alpha software with unstable API (as of 2025)

## Build and Development Commands

```bash
# Build the entire workspace
cargo build

# Build release version (optimized)
cargo build --release

# Build the TicTacToe example
cargo build --release --bin ttt

# Run tests (currently minimal coverage)
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter (configured in clippy.toml)
cargo clippy
```

## Architecture

The framework follows a clear data flow pattern:

1. **Generator** → Creates specially-formatted transactions with embedded commands
2. **Proxy** → Listens to Kaspa network for pattern-matched transactions
3. **Engine** → Manages episode lifecycles and handles DAG reorganizations
4. **Episode** → Developer-implemented application logic

### Core Abstractions

- **Episode trait** (`src/episode.rs`): The main abstraction developers implement
  - Define Command, CommandRollback, and CommandError types
  - Implement `initialize()`, `execute()`, and `rollback()` methods
  - All types must implement Borsh serialization

- **Engine** (`src/engine.rs`): Central controller managing multiple episodes
  - Handles DAG reorganizations via rollback stack
  - Filters episodes by DAA score
  - Single-threaded for simplicity

- **TransactionGenerator** (`src/generator.rs`): Creates Kaspa transactions
  - Uses pattern matching (incrementing nonces) for discoverable TX IDs
  - Embeds Borsh-serialized episode commands in transaction payloads

- **Proxy** (`src/proxy.rs`): Network listener
  - Connects via Kaspa wRPC
  - Filters transactions by bit pattern and prefix
  - Extracts and forwards payloads to engine

### Key Patterns

1. **Serialization**: All on-chain data uses Borsh (Binary Object Representation Serializer)
2. **Authorization**: secp256k1 ECDSA signatures for command authorization
3. **Reorg Safety**: Every state change produces a rollback object
4. **Message Passing**: Async proxy communicates with sync engine via channels

## Working with Episodes

When implementing a new dApp:

1. Define your episode's command types with Borsh derives
2. Define rollback types to capture state changes
3. Implement the Episode trait
4. Create an event handler to track state changes
5. Set up engine and proxy with your pattern/prefix
6. Use TransactionGenerator to submit commands

Example implementation: `examples/tictactoe/src/game.rs`

## Testing Approach

- Unit test episode logic directly
- Test rollback scenarios explicitly
- Use mock messages for engine integration tests
- Current test coverage is minimal - prioritize testing when adding features

## Environment Requirements

- Rust 1.83.0 or higher
- Access to Kaspa node (defaults to public PNN nodes)
- Tokio async runtime (multi-threaded)

## Important Considerations

1. **No Persistence**: Episodes are in-memory only - consider persistence needs for production
2. **Pattern Matching**: Transaction discovery uses bit patterns - choose patterns carefully
3. **DAG Reorgs**: Always implement proper rollback logic to handle reorganizations
4. **Gas Efficiency**: Borsh serialization is optimized for on-chain storage costs
5. **Episode Isolation**: Each episode type should run in its own engine instance

## Common Development Tasks

When working on the core library:
- Main entry point: `kdapp/src/lib.rs`
- Focus areas from README: persistence, client-server architecture, oracle integration

When creating a new example:
- Follow the pattern in `examples/tictactoe/`
- Include both episode implementation and client logic
- Add comprehensive tests for the episode logic