# Natural Language kdapp Interface

Transform natural language prompts into deployable blockchain applications on Kaspa.

## Overview

This project enables anyone to create kdapp Episodes through simple text descriptions like "Make me a chess game". It combines AI-powered code generation with the kdapp framework to democratize blockchain development.

## Architecture

The system follows a hybrid on-chain/off-chain architecture inspired by kasperience's kaspa-auth:

- **On-chain**: Generated kdapp Episodes handle game logic and state
- **Off-chain**: Natural language processing, code generation, and web interface
- **Real-time**: WebSocket connections for instant game updates

## Quick Start

### Prerequisites

- Rust 1.83.0 or higher
- Access to a Kaspa node (defaults to public nodes)
- kdapp framework (in parent directory)

### Installation

```bash
# From the kdapp root directory
cd nl-kdapp
cargo build --release
```

### Running

```bash
# Start the server
cargo run -- --port 3000 --network testnet-10

# With debug logging
cargo run -- --debug

# With custom Kaspa node
cargo run -- --wrpc-url wss://localhost:17110
```

### Usage

1. Open http://localhost:3000 in your browser
2. Enter a natural language prompt (e.g., "Make me a tic-tac-toe game")
3. Click Generate
4. Share the generated link to play!

## Project Structure

```
nl-kdapp/
├── src/
│   ├── nlp/          # Natural language processing
│   ├── generation/   # Code generation engine
│   ├── templates/    # Episode templates (based on kasperience patterns)
│   ├── web/          # HTTP/WebSocket server
│   ├── deployment/   # Compilation and deployment
│   ├── session/      # Session management
│   └── utils/        # Utilities
└── tests/            # Test suite
```

## Development Status

### Phase 1 (Current): Proof of Concept
- [x] Basic project structure
- [x] Simple NLP pattern matching
- [x] Tic-tac-toe template
- [x] Web interface
- [ ] Code compilation
- [ ] Actual deployment
- [ ] WebSocket game updates

### Phase 2: MVP
- [ ] AI model integration
- [ ] Multiple game templates
- [ ] User sessions
- [ ] Game history

### Phase 3: Production
- [ ] Advanced NLP understanding
- [ ] Custom game generation
- [ ] Tournament support
- [ ] Analytics

## Contributing

See the main [claude-docs](../claude-docs/) for detailed documentation and contribution guidelines.

## Attribution

This project builds upon:
- kdapp framework by Michael Sutton
- kaspa-auth patterns by kasperience
- The Kaspa developer community

## License

ISC License (same as kdapp)