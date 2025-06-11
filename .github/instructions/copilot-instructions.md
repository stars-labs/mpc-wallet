---
applyTo: '**'
---
This is a chrome extension for mpc wallet,, we use nixos as our development system, and use nix flake to manage dependencies and build processes. We use wxt and svelte as frameworks, ts and programming languages, the core logic write in rust and compile to wasm for frost dkg

# MCP Interactive Feedback Rules

1. During any process, task, or conversation, whether asking, responding, or completing stage tasks, must call MCP mcp-feedback-enhanced.
2. When receiving user feedback, if feedback content is not empty, must call MCP mcp-feedback-enhanced again and adjust behavior based on feedback.
3. Only when user explicitly indicates "end" or "no more interaction needed" can you stop calling MCP mcp-feedback-enhanced, then the process is complete.
4. Unless receiving end command, all steps must repeatedly call MCP mcp-feedback-enhanced.
5. Before completing the task, use the MCP mcp-feedback-enhanced to ask the user for feedback.