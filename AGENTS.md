# AGENTS.md

## Project Status
**Current Phase:** Phase 2 (The Sifter/Intelligence)

### Completed
- Initialized Tauri + React + TypeScript project in `nexus/`.
- Configured Tailwind CSS v4.
- Created `PRD.md` with project requirements.
- Cloned `radkit` documentation to `radkit_docs/`.
- **Phase 1: The Skeleton**
    - Implemented `chromiumoxide` background process (`browser.rs`).
    - Implemented Basic "Fetch and Search" command (`fetch_and_search`).
    - Added `radkit` and other dependencies to `Cargo.toml`.

## Instructions for Next Agent
Your goal is to start Phase 2 of the Roadmap: The Sifter (Intelligence).

### Tasks
1. **Integrate `html-to-markdown-rs`:**
   - Add the dependency.
   - Convert fetched HTML content to Markdown before processing/searching to reduce tokens.

2. **Connect `radkit`:**
   - Initialize `radkit` with an LLM provider (e.g., Anthropic or OpenAI).
   - Implement the "Reason + Act" loop using `radkit`.
   - Create a simple agent that can decide to search or navigate based on user input.

3. **Build "Thought Stream" UI:**
   - Create a frontend component to display the agent's reasoning logs (Events).
   - Connect backend events to the frontend.

### Radkit Integration
**Important:** Use `radkit` for all agentic logic.
- Reference `radkit_docs/` for implementation details.
- See `radkit_docs/core-concepts` for Thread, Event, and Content models.
- Use `radkit`'s `LlmWorker` or `LlmFunction` to implement the agent loop.

## Documentation
- **PRD:** See `PRD.md` for full requirements.
- **Radkit:** See `radkit_docs/` for the "bible" on agent implementation.

## Build Instructions
- Frontend: `cd nexus && npm install && npm run build`
- Tauri: `cd nexus && npm run tauri dev` (or `cargo tauri dev`)
