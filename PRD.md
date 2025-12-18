# PRD: Project Nexus
**Subtitle:** The Sovereign AI-Native Agentic Browser
**Status:** Draft / Concept
**Author:** AI-Rust Architecture Team

---

## 1. Executive Summary
Nexus is not a traditional web browser. It is a **headless-first agentic engine** that allows users to interact with the web through natural language. By bypassing the traditional "tabs and URL bars" model, Nexus uses Rust-based automation, high-speed text filtering (Ripgrep), and local indexing (Tantivy) to execute complex tasks autonomously.

## 2. Problem Statement
*   **Context Bloat:** Sending raw HTML to LLMs is expensive and slow.
*   **Sandbox Limitations:** Chrome extensions are restricted by browser security policies and cannot access local file systems or perform high-performance multi-threading.
*   **Passive Browsing:** Current browsers require manual navigation. Users want **outcomes** (e.g., "Book this flight"), not just "pages."

## 3. Target Audience
*   **Power Users:** Developers and researchers who need to automate complex web workflows.
*   **Privacy-Conscious Users:** Individuals who want their browsing history indexed locally rather than in the cloud.
*   **AI Engineers:** Developers looking for a "sovereign" stack to build web agents.

## 4. Functional Requirements

### 4.1. The "Hands" (Automation)
*   **Direct Control:** Use `chromiumoxide` to control Chromium via the DevTools Protocol (CDP).
*   **Multi-mode:** Support for **Headless** (background tasks) and **Headed** (human-in-the-loop for 2FA/Captchas).
*   **Network Interception:** Ability to intercept JSON API calls before they render to the DOM for 100x faster data extraction.

### 4.2. The "Sifter" (Context Optimization)
*   **Live Filtering:** Use the `grep` (Ripgrep) engine to scan page source in microseconds.
*   **Markdown Conversion:** Convert HTML to Markdown using `html-to-markdown-rs` to reduce LLM token usage by ~70%.
*   **DOM Pruning:** Automatically strip scripts, styles, and SVG data before sending context to the LLM.

### 4.3. The "Brain" (Agentic Logic)
*   **ReAct Loop:** Implement a "Reason + Act" loop using `rig-core`.
*   **Tool Calling:** The AI must be able to "call" Rust functions to click, type, or scroll.
*   **Structured Output:** Use `serde_json` to force the LLM to return valid action schemas.

### 4.4. The "Memory" (Long-term Storage)
*   **Local Indexing:** Use `Tantivy` to build a full-text search index of every "sifted" page.
*   **Personal RAG:** Allow the LLM to query the local Tantivy index to answer questions about the user's past browsing history.

---

## 5. Technical Stack

| Layer | Technology |
| :--- | :--- |
| **App Shell** | **Tauri (Rust)** |
| **Frontend** | **React + Tailwind CSS** (Glassmorphism UI) |
| **Browser Engine** | **Chromiumoxide** (CDP-based) |
| **Text Filtering** | **Grep crate** (Ripgrep Engine) |
| **Local Search** | **Tantivy** |
| **AI Framework** | **Rig-core** (Agentic Orchestration) |
| **Communication** | **Tauri Events & Commands** (IPC) |

---

## 6. User Experience & Interface (UX/UI)

### 6.1. The Omnibox (Primary Interface)
*   A floating, translucent command bar (Cmd+K).
*   No URL bar by default; only a "Goal" input.

### 6.2. The Thought Stream
*   A terminal-style sidebar showing real-time logs:
    *   `[Sifting]` Finding checkout button...
    *   `[Memory]` Found similar item in Tantivy index (last Tuesday).
    *   `[Action]` Clicking "Confirm Purchase."

### 6.3. The Portal (PIP)
*   A small, collapsible window showing the live Chromium stream when the agent requires supervision.

---

## 7. Roadmap

### Phase 1: The Skeleton (Foundation)
*   Set up Tauri + React boilerplate.
*   Implement `chromiumoxide` background process.
*   Basic "Fetch and Search" command using `reqwest` and `grep`.

### Phase 2: The Sifter (Intelligence)
*   Integrate `html-to-markdown-rs`.
*   Connect `rig-core` with OpenAI/Claude API.
*   Build the "Thought Stream" UI to display agent reasoning.

### Phase 3: The Library (Memory)
*   Implement `Tantivy` schema for page indexing.
*   Create a "Flashback" feature to search past sessions.
*   Enable Local RAG (Retrieval-Augmented Generation).

### Phase 4: Sovereignty (Autonomy)
*   Support for local LLMs (via Ollama integration).
*   Multi-agent workflows (running 3 browsers in parallel).
*   Desktop-level MCP (Model Context Protocol) support for local file access.

---

## 8. Success Metrics
*   **Token Efficiency:** Reduce average tokens-per-page-action by >60% compared to raw HTML scraping.
*   **Latency:** Achieve sub-100ms "sifting" of pages larger than 1MB.
*   **Resource Usage:** Maintain <150MB RAM usage for the Tauri shell (excluding the Chromium process).
