# Nexus

**The Sovereign AI-Native Agentic Browser**

Nexus is a headless-first agentic engine that allows users to interact with the web through natural language. Built with Tauri (Rust) and React, it bypasses the traditional "tabs and URL bars" model to execute complex tasks autonomously using Rust-based automation and LLM intelligence.

## Features

- **Headless Automation**: Controls Chromium via DevTools Protocol (using `chromiumoxide`).
- **Context Optimization**: Converts HTML to Markdown and prunes DOM to save tokens.
- **Agentic Logic**: Uses `radkit` for "Reason + Act" loops.
- **Memory System**: Stores findings and observations for later recall.
- **Real-time Thought Stream**: Visualizes the agent's reasoning process.

## Prerequisites

### Linux (Ubuntu/Debian)
Building the Tauri backend requires system dependencies:

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

## Configuration

Nexus configuration is managed via environment variables.

**Supported Providers:** Anthropic (default), OpenAI, OpenRouter.

### Required Variables

Depending on your chosen provider, set the appropriate API key:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# OR
export OPENAI_API_KEY="sk-..."
# OR
export OPENROUTER_API_KEY="sk-or-..."
```

### Optional Variables

- `LLM_PROVIDER`: Set the LLM provider. Options: `anthropic`, `openai`, `openrouter`. (Default: `anthropic`)
- `LLM_MODEL`: Override the default model (e.g., `gpt-4o`, `claude-3-5-sonnet-20240620`).

## Development

1.  **Install Frontend Dependencies**:
    ```bash
    cd nexus
    npm install
    ```

2.  **Run Development Server**:
    ```bash
    npm run tauri dev
    ```
    Or if you prefer cargo:
    ```bash
    cargo tauri dev
    ```

## Project Structure

- `nexus/src-tauri`: Rust backend (Tauri).
- `nexus/src`: React frontend.
- `radkit_docs`: Documentation for the `radkit` agent framework.
