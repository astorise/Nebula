import * as vscode from "vscode";
import { createNebulaSocket, readConnectionConfig } from "./connection";

let activeSocket: ReturnType<typeof createNebulaSocket> | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const command = vscode.commands.registerCommand("nebula.openDashboard", () => {
    const panel = vscode.window.createWebviewPanel(
      "nebulaDashboard",
      "Nebula",
      vscode.ViewColumn.One,
      { enableScripts: true }
    );

    panel.webview.html = renderDashboard();
    connect(panel);
  });

  context.subscriptions.push(command, {
    dispose: () => activeSocket?.close()
  });
}

export function deactivate(): void {
  activeSocket?.close();
}

function connect(panel: vscode.WebviewPanel): void {
  try {
    activeSocket?.close();
    activeSocket = createNebulaSocket(readConnectionConfig());

    activeSocket.on("open", () => {
      activeSocket?.send(JSON.stringify({ type: "nebula.dashboard.ready" }));
      panel.webview.postMessage({ type: "status", value: "connected" });
    });

    activeSocket.on("message", (raw) => {
      panel.webview.postMessage({ type: "event", value: raw.toString() });
    });

    activeSocket.on("close", () => {
      panel.webview.postMessage({ type: "status", value: "disconnected" });
    });

    activeSocket.on("error", (error) => {
      panel.webview.postMessage({ type: "status", value: error.message });
    });
  } catch (error) {
    panel.webview.postMessage({
      type: "status",
      value: error instanceof Error ? error.message : "Unable to connect"
    });
  }
}

function renderDashboard(): string {
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Nebula</title>
  <style>
    body { margin: 0; padding: 24px; font-family: var(--vscode-font-family); color: var(--vscode-foreground); background: var(--vscode-editor-background); }
    main { display: grid; gap: 18px; max-width: 960px; }
    section { border: 1px solid var(--vscode-panel-border); padding: 16px; }
    button { padding: 8px 12px; }
    pre { white-space: pre-wrap; overflow-wrap: anywhere; }
  </style>
</head>
<body>
  <main>
    <h1>Nebula</h1>
    <section>
      <h2>Workflow</h2>
      <button>Start</button>
      <button>Pause</button>
      <button>Monitor</button>
    </section>
    <section>
      <h2>Pipeline</h2>
      <p>Tier 3 -> LoRA -> safetensors</p>
      <pre id="events"></pre>
    </section>
  </main>
  <script>
    const events = document.getElementById('events');
    window.addEventListener('message', ({ data }) => {
      events.textContent += JSON.stringify(data) + '\\n';
    });
  </script>
</body>
</html>`;
}
