# AGENTS.md

## Project Status
**Current Phase:** Phase 1 (In Progress)

### Completed
- Initialized Tauri + React + TypeScript project in `nexus/`.
- Configured Tailwind CSS v4.
- Created `PRD.md` with project requirements.
- Cloned `radkit` documentation to `radkit_docs/`.

## Instructions for Next Agent
Your goal is to continue Phase 1 of the Roadmap.

### Tasks
1. **Implement `chromiumoxide` background process:**
   - Add `chromiumoxide` to `nexus/src-tauri/Cargo.toml`.
   - Create a Rust module to manage the headless browser instance.
   - Ensure it can run in the background.

2. **Basic "Fetch and Search" command:**
   - Implement a Tauri command that takes a URL and a search query.
   - Use `reqwest` (or `chromiumoxide` navigation) to fetch the page.
   - Use `grep` crate (or similar text search) to find occurrences.
   - Return results to the frontend.

### Radkit Integration
**Important:** Use `radkit` for all agentic logic. The PRD mentions `rig-core`, but we are using `radkit` instead.
- Reference `radkit_docs/` for implementation details.
- See `radkit_docs/core-concepts` for Thread, Event, and Content models.
- When you reach Phase 2 (The Sifter/Intelligence), use `radkit`'s `LlmWorker` or `LlmFunction` to implement the agent loop.

## Documentation
- **PRD:** See `PRD.md` for full requirements.
- **Radkit:** See `radkit_docs/` for the "bible" on agent implementation.

## Build Instructions
- Frontend: `cd nexus && npm install && npm run build`
- Tauri: `cd nexus && npm run tauri dev` (or `cargo tauri dev`)
