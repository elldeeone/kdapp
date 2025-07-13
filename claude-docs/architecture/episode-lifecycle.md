# Episode Lifecycle and Current Limitations

## Understanding kdapp Episodes

Episodes in kdapp are currently **ephemeral, session-based experiences**. This document explains the lifecycle, limitations, and future evolution path.

## Current State: Ephemeral Episodes

### Lifecycle
```
Birth → Active → Death
  │        │        │
Create   Play    Server restart
         │       OR 3+ days
         │       OR completion
         └─ All in memory
```

### Key Characteristics

1. **Memory-Only State**
   - Episodes exist only in RAM while Engine runs
   - No disk persistence
   - Server restart = all Episodes lost

2. **3-Day Window**
   - Kaspa network prunes transactions after ~3 days
   - Episodes older than 3 days cannot be recovered
   - Natural expiration for all games

3. **Reorg Safety via Rollback**
   - Every command must be reversible
   - Engine maintains rollback stack
   - DAG reorganizations handled automatically

### What This Means for Users

✅ **Perfect for:**
- Quick games (complete in one session)
- Live events (auctions, votes)
- Real-time competitions
- Casual gaming experiences

❌ **Not suitable for:**
- Long tournaments
- Persistent game worlds
- Historical leaderboards
- Save-and-resume gameplay

## Proper Episode Implementation

Every generated Episode MUST implement proper rollback:

```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub struct GameRollback {
    previous_state: GameState,
    move_made: Move,
    timestamp: u64,
}

impl Episode for MyGame {
    type CommandRollback = GameRollback;
    
    fn execute(&mut self, cmd: Command) -> Result<GameRollback, Error> {
        // Capture current state
        let rollback = GameRollback {
            previous_state: self.state.clone(),
            move_made: cmd.clone(),
            timestamp: self.timestamp,
        };
        
        // Make the change
        self.apply_move(cmd)?;
        
        // Return rollback info
        Ok(rollback)
    }
    
    fn rollback(&mut self, rb: GameRollback) {
        // Restore exact previous state
        self.state = rb.previous_state;
        self.timestamp = rb.timestamp;
    }
}
```

## kdapp.fun Implementation Strategy

### 1. User Messaging
```
🎮 Session Game - Complete in one sitting!
⏱️ Expires in 3 days
💾 Persistence coming soon!
```

### 2. Game Selection
Focus on experiences that complete quickly:
- Tic-tac-toe (minutes)
- Chess/Checkers (< 1 hour)
- Quick votes (hours to 1 day)
- Flash auctions (minutes to hours)

### 3. Architecture Preparation
Design interfaces that can adapt when persistence arrives:

```rust
trait EpisodeStore {
    fn save(&self, id: &str, state: &[u8]) -> Result<()>;
    fn load(&self, id: &str) -> Result<Vec<u8>>;
}

// Current implementation
struct MemoryStore;

// Future implementation
struct PersistentStore {
    db: RocksDB,
}
```

## Future Evolution: Persistence Roadmap

### Stage 1: Short-Term Sync
- Store last sync point
- Recover from brief downtimes
- Still within 3-day window

### Stage 2: Full Reorg Protection
- Persistent rollback storage
- Survive server restarts
- Handle deep reorganizations

### Stage 3: Complete Archival
- Full Episode history
- New nodes can sync from genesis
- True game persistence!

## When Persistence Arrives

kdapp.fun will seamlessly upgrade to offer:

### New Game Types
- 🏆 Multi-week tournaments
- 🌍 Persistent game worlds
- 📊 Historical statistics
- 🎯 Long-term achievements

### Enhanced Features
- Save and resume games
- Replay historic matches
- Persistent leaderboards
- Game state export/import

### User Choice
```
Create Episode:
[ ] Session Mode (Current)
    Quick games, no persistence
    
[ ] Persistent Mode (Future)
    Save progress, resume anytime
    Additional storage fees apply
```

## Development Guidelines

1. **Don't hard-code limitations**
   - Use interfaces that can evolve
   - Prepare for persistence

2. **Educate users gently**
   - Explain current limitations
   - Highlight future capabilities

3. **Design for both modes**
   - Quick session games now
   - Persistent worlds later

4. **Test reorg handling**
   - Every Episode must handle rollback
   - Simulate DAG reorganizations

## Conclusion

The ephemeral nature of current Episodes is not a limitation—it's a starting point. By embracing session-based gameplay while preparing for persistence, kdapp.fun can deliver immediate value while building toward a more permanent future.

As the kdapp framework evolves, so will the experiences we can create. Today's quick chess game could become tomorrow's persistent tournament system!