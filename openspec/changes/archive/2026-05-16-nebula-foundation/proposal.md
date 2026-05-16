# Proposal: Nebula Architecture Foundation

## Context

Nebula acts as the ecosystem training forge. The architecture needs a familiar user interface (VS Code) paired with a local orchestration engine (Node.js CLI) that can interface with the Tachyon mesh and read contextual data without risking accidental mutation.

## Objectives

- **Node.js CLI (Engine):**
  - Expose a **read-only WebDAV server**. It provides a virtual mount point for ingesting local documentation (PDF, Markdown, DOCX) for the Tier 3 LLM (Teacher), blocking any deletion or modification by design.
  - Expose a **WebSocket server** to maintain real-time bidirectional communication with Wasm FaaS functions deployed on Tachyon.
- **VS Code Extension (Interface):**
  - Reuse Pulsar-style ergonomics to drive learning workflows and visualize dataset generation and LoRA fine-tuning.
- **Security (Learning Workflow):**
  - Strictly secure the WebDAV and WebSocket endpoints with a **client certificate (mTLS)** immediately, while leaving room for a future OAuth2 flow.
