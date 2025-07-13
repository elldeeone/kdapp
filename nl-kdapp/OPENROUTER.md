# OpenRouter Integration

The Natural Language kdapp Interface supports multiple LLMs through [OpenRouter](https://openrouter.ai/), allowing you to easily switch between different AI models.

## Setup

1. Get an API key from [OpenRouter](https://openrouter.ai/)

2. Set your API key using one of these methods:
   ```bash
   # Environment variable (recommended)
   export OPENROUTER_API_KEY="your-api-key-here"
   
   # Or pass via command line
   cargo run -- --openrouter-api-key "your-api-key-here"
   ```

## Available Models

The following models are preconfigured:

- `claude3-opus` - Anthropic Claude 3 Opus (most capable)
- `claude3-sonnet` - Anthropic Claude 3 Sonnet (default, good balance)
- `gpt4` - OpenAI GPT-4
- `gpt4-turbo` - OpenAI GPT-4 Turbo
- `llama3-70b` - Meta Llama 3 70B
- `mixtral` - Mistral Mixtral 8x7B

## Usage Examples

```bash
# Use default model (Claude 3 Sonnet)
cargo run -- --openrouter-api-key "your-key"

# Use GPT-4
cargo run -- --openrouter-api-key "your-key" --model gpt4

# Use Llama 3 70B
cargo run -- --openrouter-api-key "your-key" --model llama3-70b

# Use any OpenRouter model by ID
cargo run -- --openrouter-api-key "your-key" --model "anthropic/claude-3-haiku"
```

## Model Selection Guide

- **Claude 3 Sonnet** (default): Best balance of speed, cost, and quality for parsing game requests
- **Claude 3 Opus**: Most capable but slower and more expensive
- **GPT-4**: Excellent alternative, very capable
- **GPT-4 Turbo**: Faster and cheaper than GPT-4, still very good
- **Llama 3 70B**: Open source, good for cost-conscious usage
- **Mixtral**: Fast and cheap, good for simple requests

## Fallback Behavior

If OpenRouter is not configured or fails, the system automatically falls back to simple pattern matching that supports:
- Tic-tac-toe
- Chess
- Checkers
- Basic poker

## Cost Considerations

OpenRouter charges per token based on the model used. For typical game requests:
- Simple requests: ~100-200 tokens
- Complex requests: ~200-500 tokens

Check [OpenRouter pricing](https://openrouter.ai/models) for current rates.

## Debugging

Enable debug logging to see which model is being used:
```bash
cargo run -- --debug --openrouter-api-key "your-key"
```

## Advanced Usage

You can use any model available on OpenRouter:
```bash
# Use a specific model by its full ID
cargo run -- --openrouter-api-key "your-key" --model "anthropic/claude-2.1"
```

## Environment Variables

- `OPENROUTER_API_KEY`: Your OpenRouter API key
- `RUST_LOG`: Set to `debug` for detailed logging