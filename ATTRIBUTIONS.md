# Attributions

This project builds upon the excellent work of the kdapp community and incorporates patterns and insights from various contributors.

## kdapp Framework
- **Author**: Michael Sutton (michaelsutton)
- **Source**: https://github.com/michaelsutton/kdapp
- **License**: ISC License
- **Description**: The core framework for building high-frequency, interactive decentralized applications on Kaspa

## kaspa-auth Implementation
- **Author**: kasperience
- **Source**: https://github.com/kasperience/kdapp/tree/master/examples/kaspa-auth
- **License**: ISC License
- **Description**: A sophisticated peer-to-peer authentication system built on kdapp that demonstrates advanced Episode patterns

### Components Inspired by kaspa-auth:
The following architectural patterns and implementations are adapted from kasperience's kaspa-auth example:

1. **Hybrid Architecture Pattern**
   - Combination of on-chain Episodes with off-chain HTTP/WebSocket coordination
   - Real-time event propagation to web clients
   - Session management within Kaspa's pruning window

2. **Episode Structure Patterns**
   - Multi-command Episode design (RequestChallenge, SubmitResponse, RevokeSession)
   - Event handler implementation for state change notifications
   - Cryptographic challenge-response authentication flow

3. **Web Integration Approach**
   - Axum-based HTTP server for API endpoints
   - WebSocket integration for real-time updates
   - Browser-friendly session token management

### Modifications and Extensions:
Our natural language interface project extends these patterns by:
- Generalizing Episode patterns for AI-driven code generation
- Creating template-based Episode generation from natural language prompts
- Implementing a code generation pipeline that produces kdapp-compatible Episodes
- Adding support for multiple Episode types beyond authentication

## Community Contributions
We welcome contributions and encourage the community to build upon this work. If you use patterns or code from this project, please provide appropriate attribution.

## License
This project is licensed under the ISC License, consistent with the broader kdapp ecosystem.

---

For detailed attribution information about specific code adaptations, see `claude-docs/resources/attributions-detail.md`.