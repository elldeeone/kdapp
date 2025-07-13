# Natural Language Processing Interface Specification

## Overview

The NLP Interface is the front door to our system, transforming human-readable descriptions into structured requirements that can be converted into kdapp Episodes. This specification defines how users interact with the system and how their intent is processed.

## Supported Input Types

### 1. Simple Game Requests
```
"Make me a chess game"
"Create tic-tac-toe"
"I want to play poker"
```

### 2. Parameterized Requests
```
"Create a poker game for 6 players"
"Make chess with 5 minute time limits"
"Build a betting game with 100 KAS buy-in"
```

### 3. Complex Descriptions
```
"I want a Texas Hold'em poker game where players can join with 50-500 KAS, 
with blinds that increase every 10 minutes, supporting up to 9 players"
```

### 4. Custom Rule Requests
```
"Create checkers but pieces can move backwards"
"Make chess where pawns can move 3 squares on first move"
"Build a card game like blackjack but with goal of 25 instead of 21"
```

## NLP Processing Pipeline

### Stage 1: Intent Classification
Determine the type of application requested:
- **Game**: Chess, poker, tic-tac-toe, etc.
- **Market**: Betting, prediction, auction
- **Governance**: Voting, proposals
- **Financial**: Escrow, payment splitting

### Stage 2: Entity Extraction
Extract key parameters from the request:
- **Player Count**: Number of participants
- **Game Rules**: Specific modifications
- **Economic Parameters**: Buy-ins, stakes, rewards
- **Time Constraints**: Turn timers, game duration

### Stage 3: Template Mapping
Match the processed request to available templates:
```json
{
  "intent": "game",
  "type": "poker",
  "variant": "texas_holdem",
  "parameters": {
    "max_players": 6,
    "buy_in_min": 50,
    "buy_in_max": 500,
    "blind_increase_interval": 600
  }
}
```

## Input Format Specifications

### Primary Input
- **Type**: Plain text
- **Length**: 10-500 characters
- **Language**: English (initially)
- **Format**: Natural conversational style

### Structured Alternatives
For power users, support structured input:
```yaml
type: game
name: poker
variant: texas_holdem
players:
  min: 2
  max: 9
economy:
  buy_in: [50, 500]
  blinds:
    small: 5
    big: 10
    increase_every: 10m
```

## Output Specifications

### Success Response
```json
{
  "status": "success",
  "interpretation": {
    "type": "game",
    "template": "poker_holdem",
    "parameters": {...}
  },
  "episode_id": "ep_123abc",
  "deployment_status": "in_progress",
  "estimated_time": 45,
  "share_link": "https://play.kdapp.io/ep_123abc"
}
```

### Clarification Request
```json
{
  "status": "needs_clarification",
  "interpretation": {
    "understood": "poker game with betting",
    "unclear": ["number of players", "betting limits"]
  },
  "suggestions": [
    "How many players should the game support?",
    "What should the minimum and maximum bets be?"
  ]
}
```

### Error Response
```json
{
  "status": "error",
  "reason": "unsupported_game_type",
  "message": "I don't know how to create 'quantum chess' yet",
  "suggestions": ["chess", "3d chess", "speed chess"]
}
```

## Supported Application Types

### Phase 1 (MVP)
1. **Turn-Based Games**
   - Tic-tac-toe
   - Chess
   - Checkers
   - Connect Four

2. **Simple Card Games**
   - Blackjack
   - High Card
   - War

3. **Basic Betting**
   - Coin flip
   - Dice roll
   - Number guessing

### Phase 2
1. **Complex Card Games**
   - Texas Hold'em
   - Omaha
   - Rummy variants

2. **Strategy Games**
   - Go
   - Reversi
   - Stratego

3. **Market Systems**
   - Simple auctions
   - Prediction markets
   - Betting pools

### Phase 3
1. **Custom Games**
   - User-defined rules
   - Hybrid game types
   - Tournament systems

2. **Complex Markets**
   - Options trading
   - Multi-asset auctions
   - Governance systems

## Natural Language Understanding Rules

### 1. Synonym Recognition
```
"poker" = "texas holdem" = "holdem" = "hold'em"
"betting" = "wagering" = "gambling"
"players" = "participants" = "people"
```

### 2. Default Assumptions
When parameters are not specified:
- Player count: 2 (minimum viable)
- Betting: Disabled unless mentioned
- Time limits: None unless specified
- Entry fee: Free unless stated

### 3. Context Understanding
```
"Make me chess like the one we played yesterday"
→ Retrieve user's previous game configurations

"Create the same poker game but with 8 players"
→ Clone previous game, modify player count
```

### 4. Error Tolerance
Handle common mistakes:
- Spelling errors: "pokr" → "poker"
- Grammar variations: "make poker" = "create a poker game"
- Casual language: "yo make me some chess" → "create chess game"

## AI Model Integration

### Primary Model
- **Model**: Claude 3 or GPT-4
- **Task**: Convert natural language to structured format
- **Fallback**: Rule-based parsing for common requests

### Prompt Template
```
User request: "{user_input}"

Parse this request for a blockchain game/application. Extract:
1. Application type (game/market/governance)
2. Specific variant (e.g., poker/chess/auction)
3. Parameters:
   - Number of participants
   - Economic rules (betting, buy-ins)
   - Time constraints
   - Custom rules

Output as JSON:
{
  "type": "...",
  "variant": "...",
  "parameters": {...}
}
```

### Confidence Scoring
Rate interpretation confidence:
- **High** (>0.9): Proceed automatically
- **Medium** (0.7-0.9): Show interpretation for confirmation
- **Low** (<0.7): Request clarification

## Privacy and Security

### Input Sanitization
- Remove personal information
- Filter inappropriate content
- Prevent injection attacks
- Limit request frequency

### Data Handling
- Don't store raw user inputs long-term
- Anonymous analytics only
- No PII in generated Episodes
- Secure transmission (HTTPS/WSS)

## Performance Requirements

### Response Times
- Intent classification: <100ms
- Full processing: <500ms
- User feedback: <1 second
- Episode deployment: <60 seconds

### Scalability
- Handle 1000 concurrent requests
- Queue system for generation
- Graceful degradation
- Clear user feedback on delays

## Extensibility

### Plugin System
Allow community to add:
- New game types
- Custom templates
- Language packs
- Rule variants

### API Access
Provide programmatic access:
```typescript
const response = await nlpAPI.process({
  text: "Create a chess game",
  user_context: { previous_games: [...] },
  preferences: { style: "competitive" }
});
```

## Testing Strategy

### Unit Tests
- Each intent type
- Parameter extraction
- Error handling
- Edge cases

### Integration Tests
- Full pipeline processing
- AI model integration
- Template selection
- Response generation

### User Testing
- Real user inputs
- Ambiguous requests
- Error scenarios
- Performance under load

## Future Enhancements

1. **Multi-language Support**
   - Spanish, Chinese, Japanese, etc.
   - RTL language handling
   - Cultural adaptations

2. **Voice Input**
   - Speech-to-text integration
   - Voice assistants
   - Accessibility features

3. **Learning System**
   - Improve from user corrections
   - Adapt to user preferences
   - Community-driven improvements

4. **Advanced Understanding**
   - Multi-turn conversations
   - Complex rule inference
   - Creative interpretations

## Conclusion

The NLP Interface is the key to democratizing blockchain development. By accepting natural language and converting it to structured requirements, we remove the technical barriers that prevent most people from creating decentralized applications. This specification provides the foundation for building an intuitive, powerful, and extensible system that brings blockchain development to everyone.