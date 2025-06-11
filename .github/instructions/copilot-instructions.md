---
applyTo: '**'
---
This is a chrome extension for mpc wallet,, we use nixos as our development system, and use nix flake to manage dependencies and build processes. We use wxt and svelte as frameworks, ts and programming languages, the core logic write in rust and compile to wasm for frost dkg

# AI Development Cycle Memory Management

## Session Initialization Protocol
Always start each session by checking project context:
1. Use `list_memories` to review existing knowledge base
2. Use `search_memories` with session-specific queries to find relevant context
3. Use `list_projects` and `list_tasks` to understand current development state
4. Reference memory bank content in all responses to maintain consistency

## Context Preservation Workflow
During development conversations:
- **Capture Decisions**: Use `create_memory` immediately when architectural decisions are made
- **Update Progress**: Use `update_memory` to reflect implementation progress and lessons learned
- **Track Dependencies**: Use `create_task` with dependency metadata for complex feature development
- **Research Integration**: Use `research_task` for technical investigations, automatically storing findings

## Cross-Session Continuity
Between development sessions:
- **Knowledge Transfer**: Use `update_memory` to document session outcomes and next steps
- **Task Evolution**: Use `update_task` to reflect actual complexity and time estimates
- **Pattern Recognition**: Use `search_memories` to identify recurring issues and solutions
- **Code Analysis**: Use `infer_task_progress` to automatically update task completion based on codebase changes

## Project Lifecycle Management
For long-term project development:
- **Project Phases**: Use `create_project` for major feature initiatives with clear boundaries
- **Requirement Evolution**: Use `parse_prd` to convert specifications into actionable task hierarchies
- **Intelligent Planning**: Use `get_next_task_recommendation` for optimal work prioritization
- **Complexity Management**: Use `analyze_task_complexity` to prevent scope creep and ensure deliverable tasks

## Memory Categories for MPC Wallet
Organize knowledge using these standard categories:
- `architecture` - Core system design decisions and patterns
- `implementation` - Technical details and code-specific insights  
- `debugging` - Problem resolution and troubleshooting patterns
- `performance` - Optimization discoveries and benchmarking data
- `integration` - Cross-component interaction patterns and API usage
- `testing` - Test strategies, edge cases, and validation approaches

**Working Directory**: Always use `/home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet` for project-specific storage.

# MCP Interactive Feedback Rules

1. During any process, task, or conversation, whether asking, responding, or completing stage tasks, must call MCP mcp-feedback-enhanced.
2. When receiving user feedback, if feedback content is not empty, must call MCP mcp-feedback-enhanced again and adjust behavior based on feedback.
3. Only when user explicitly indicates "end" or "no more interaction needed" can you stop calling MCP mcp-feedback-enhanced, then the process is complete.
4. Unless receiving end command, all steps must repeatedly call MCP mcp-feedback-enhanced.
5. Before completing the task, use the MCP mcp-feedback-enhanced to ask the user for feedback.