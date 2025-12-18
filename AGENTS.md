# AGENTS.md

## Project Status
**Current Phase:** Phase 3 (The Weaver/Synthesis)

### Completed
- Initialized Tauri + React + TypeScript project in `nexus/`.
- Configured Tailwind CSS v4.
- Created `PRD.md` with project requirements.
- Cloned `radkit` documentation to `radkit_docs/`.
- **Phase 1: The Skeleton**
    - Implemented `chromiumoxide` background process (`browser.rs`).
    - Implemented Basic "Fetch and Search" command (`fetch_and_search`).
    - Added `radkit` and other dependencies to `Cargo.toml`.
- **Phase 2: The Sifter (Intelligence)**
    - Integrated `html-to-markdown-rs` to convert fetched HTML to Markdown.
    - Connected `radkit` to implement the "Reason + Act" loop.
    - Implemented `agent.rs` with `navigate`, `search`, `click`, `type`, and `scroll` tools.
    - Built "Thought Stream" UI (`ThoughtStream.tsx`) to display agent events in real-time.
    - Refined `BrowserManager` to support stateful browsing.
- **Phase 3: The Weaver (Synthesis)**
    - Implemented `Memory` system (`nexus/src-tauri/src/memory.rs`) backed by `GLOBAL_MEMORY`.
    - Added `memorize` and `recall` tools to the agent.
    - Refactored `agent.rs` to support dynamic LLM providers via environment variables.

## Configuration (CRITICAL)
The agent configuration is **strictly** controlled by environment variables. **DO NOT HARDCODE MODELS OR KEYS.**

### Environment Variables
- `ANTHROPIC_API_KEY`: API Key for Anthropic.
- `OPENAI_API_KEY`: API Key for OpenAI.
- `OPENROUTER_API_KEY`: API Key for OpenRouter.
- `LLM_PROVIDER`: The LLM provider to use. Supported: `anthropic`, `openai`, `openrouter`. Default: `anthropic`.
- `LLM_MODEL`: The specific model string (e.g., `claude-3-5-sonnet-20240620`, `gpt-4o`). If not set, reasonable defaults are used.

## Instructions for Next Agent
Your goal is to continue Phase 3 and start Phase 4 (The Builder).

### Tasks
1. **Refine Memory**:
   - The current memory is a simple list of strings (`Vec<String>`). Consider upgrading to a more structured format or vector store if complexity increases.
   - Implement `synthesize` tool if needed (currently the agent can `recall` and then output a summary, but a dedicated tool might be better for long contexts).
2. **Phase 4: The Builder (Action)**
   - Allow the agent to perform more complex multi-step actions autonomously.
   - Improve error recovery (if a tool fails, retry or try an alternative).

### Warnings
- **DO NOT HARDCODE** model names or providers in the code. Always use `std::env::var`.
- **Verify** your changes. If you change a tool signature, update the `JsonSchema` struct.

## Robust Testing Strategy Plan

### 1. Unit Testing (Rust Backend)
- **Tool Logic:** Test `navigate` and `search` tools in isolation by mocking the `BrowserManager`. Since `BrowserManager` is hard to mock (struct), refactor tools to accept a trait `BrowserInterface` and mock that.
- **Agent Loop:** Test `run_agent_loop` with a mock LLM provider. Create a `MockLlm` that implements `radkit`'s `Llm` trait (if public) or wrapper, to simulate LLM responses without network calls.
- **Search:** Continue testing `search_content` (already exists).

### 2. Integration Testing (Backend)
- **Browser Automation:** Use a local test server (e.g. `mockito` or simple `warp` server) to serve static HTML pages. Point `BrowserManager` to these local pages to verify navigation and content extraction/conversion (HTML -> MD) works correctly.
- **Radkit Integration:** Verify `LlmWorker` constructs requests correctly (serialization tests).

### 3. Frontend Testing
- **Component Tests:** Use `Vitest` + `React Testing Library` to test `ThoughtStream` component rendering and event handling. Mock `@tauri-apps/api/event`.
- **E2E Testing:** Use `Playwright` to test the full application flow.
  - Launch the Tauri app (or web version).
  - Simulate user input in the Omnibox.
  - Verify "Thought Stream" updates (mocking the backend invoke if needed, or running full backend in dev mode).

### 4. Agent Evaluation (Evals)
- Create a dataset of "User Intents" vs "Expected Actions".
  - Example: "Find the price of BTC" -> Expected: `navigate("google.com")`, `search("BTC price")`.
- Run the agent against these prompts (mocking the web execution) to verify the *reasoning* capability (i.e. does it choose the right tools?).

## Documentation
- **PRD:** See `PRD.md` for full requirements.
- **Radkit:** See `radkit_docs/` for the "bible" on agent implementation.

## Build Instructions
- Frontend: `cd nexus && npm install && npm run build`
- Tauri: `cd nexus && npm run tauri dev` (or `cargo tauri dev`)
