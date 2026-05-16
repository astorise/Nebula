import * as vscode from "vscode";
import { createNebulaSocket, readConnectionConfig } from "./connection";
import type { DashboardState, ExtensionToWebviewMessage, NebulaEnvelope, WebviewToExtensionMessage } from "./messages";

type NebulaSocket = ReturnType<typeof createNebulaSocket>;

export class NebulaDashboardProvider implements vscode.Disposable {
  private panel: vscode.WebviewPanel | undefined;
  private socket: NebulaSocket | undefined;
  private readonly disposables: vscode.Disposable[] = [];
  private state: DashboardState = {
    connectionStatus: "disconnected",
    dataset: {
      total: 0,
      escalated: 0,
      direct: 0
    },
    trainingStatus: "waiting",
    logs: []
  };

  constructor(private readonly context: vscode.ExtensionContext) {}

  open(): void {
    this.panel = vscode.window.createWebviewPanel(
      "nebulaDashboard",
      "Nebula",
      vscode.ViewColumn.One,
      {
        enableScripts: true,
        localResourceRoots: [vscode.Uri.joinPath(this.context.extensionUri, "media")]
      }
    );

    this.panel.webview.html = this.renderDashboard(this.panel.webview);
    this.disposables.push(
      this.panel.webview.onDidReceiveMessage((message: WebviewToExtensionMessage) => this.handleWebviewMessage(message)),
      this.panel.onDidDispose(() => {
        this.panel = undefined;
      })
    );

    this.connect();
    this.postState();
  }

  dispose(): void {
    this.socket?.close();
    this.disposables.forEach((disposable) => disposable.dispose());
  }

  private connect(): void {
    try {
      this.socket?.close();
      this.socket = createNebulaSocket(readConnectionConfig());

      this.socket.on("open", () => {
        this.setConnectionStatus("connected");
        this.sendCommand("dashboard.ready", {});
      });

      this.socket.on("message", (raw) => {
        this.handleSocketMessage(raw.toString());
      });

      this.socket.on("close", () => {
        this.setConnectionStatus("disconnected");
      });

      this.socket.on("error", (error) => {
        this.setConnectionStatus(error.message);
      });
    } catch (error) {
      this.setConnectionStatus(error instanceof Error ? error.message : "Unable to connect");
    }
  }

  private handleWebviewMessage(message: WebviewToExtensionMessage): void {
    if (message.type !== "COMMAND") {
      return;
    }

    if (message.action === "curriculum.generate") {
      this.sendCommand("curriculum.generate", message.payload);
      this.appendLog(`Curriculum request: ${message.payload.count} exercises for ${message.payload.subject}`);
    }

    if (message.action === "training.forceMerge") {
      this.sendCommand("training.forceMerge", {});
      this.appendLog("Manual model merge requested");
    }
  }

  private handleSocketMessage(raw: string): void {
    let envelope: NebulaEnvelope | undefined;
    try {
      envelope = JSON.parse(raw) as NebulaEnvelope;
    } catch {
      this.appendLog(raw);
      return;
    }

    if (envelope.action === "nebula.dataset.append") {
      const payload = envelope.payload as Partial<{ source: "Escalated" | "Direct" }>;
      this.state.dataset.total += 1;
      if (payload.source === "Direct") {
        this.state.dataset.direct += 1;
      } else {
        this.state.dataset.escalated += 1;
      }
      this.postState();
      return;
    }

    if (envelope.action === "nebula.eval.results") {
      this.appendLog(`[DIVERGENCE DETECTED] ${JSON.stringify(envelope.payload)}`);
      return;
    }

    if (envelope.action === "nebula.training.ready") {
      this.state.trainingStatus = "backward";
      this.appendLog("Training ready: starting LoRA backward pass");
      this.postState();
      return;
    }

    if (envelope.action === "nebula.training.complete") {
      this.state.trainingStatus = "published";
      this.appendLog("Training complete and model published");
      this.postState();
      return;
    }

    this.appendLog(`${envelope.action}: ${JSON.stringify(envelope.payload)}`);
  }

  private sendCommand(action: string, payload: unknown): void {
    const message: NebulaEnvelope = {
      type: "COMMAND",
      action,
      payload
    };
    this.socket?.send(JSON.stringify(message));
  }

  private setConnectionStatus(connectionStatus: string): void {
    this.state = {
      ...this.state,
      connectionStatus
    };
    this.postState();
  }

  private appendLog(log: string): void {
    this.state = {
      ...this.state,
      logs: [log, ...this.state.logs].slice(0, 100)
    };
    this.postMessage({ type: "LOG", payload: log });
    this.postState();
  }

  private postState(): void {
    this.postMessage({ type: "STATE", payload: this.state });
  }

  private postMessage(message: ExtensionToWebviewMessage): void {
    void this.panel?.webview.postMessage(message);
  }

  private renderDashboard(webview: vscode.Webview): string {
    const scriptUri = webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, "media", "webview.js"));
    const nonce = getNonce();

    return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-${nonce}'; style-src 'unsafe-inline';">
  <title>Nebula</title>
</head>
<body>
  <div id="root"></div>
  <script nonce="${nonce}" src="${scriptUri}"></script>
</body>
</html>`;
  }
}

function getNonce(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let nonce = "";
  for (let i = 0; i < 32; i += 1) {
    nonce += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return nonce;
}
