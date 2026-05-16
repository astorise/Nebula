# Design: VS Code Dashboard

## Architecture

The dashboard is separated from the VS Code host:

- `NebulaDashboardProvider` manages the webview panel, application state, VS Code message passing, and the mTLS WebSocket relay.
- `src/webview/main.ts` contains the interface compiled by esbuild into `media/webview.js`.
- `messages.ts` formalizes the `EVENT|COMMAND` envelopes and shared state.

## Flow

The webview sends user commands through `acquireVsCodeApi().postMessage()`. The provider normalizes them into `{ type: "COMMAND", action, payload }` envelopes and forwards them to the CLI over WSS.

Incoming WSS events update local state:

- `nebula.dataset.append` increments the dataset gauge and the 60/40 ratio.
- `nebula.eval.results` feeds the divergence log.
- `nebula.training.ready` and `nebula.training.complete` drive the LoRA state.

## UI

The interface remains native to VS Code by using theme variables. It exposes a dataset gauge, the curriculum form, LoRA stages, and a Tier 3 log stream.
