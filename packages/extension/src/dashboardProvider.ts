import * as vscode from "vscode";
import { createNebulaSocket, readConnectionConfig } from "./connection";
import type {
  DashboardState,
  ExtensionToWebviewMessage,
  NebulaEnvelope,
  ValidationResult,
  WebviewToExtensionMessage
} from "./messages";

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
    federation: {
      paused: false,
      peers: [],
      contributions: []
    },
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

    if (message.action === "DEPLOY_LORA") {
      this.sendCommand("DEPLOY_LORA", message.payload);
      this.state.deploymentStatus = `Deploying ${message.payload.artifact}`;
      this.appendLog(`LoRA deployment requested: ${message.payload.artifact}`);
      this.postState();
    }

    if (message.action === "federation.sync.setPaused") {
      this.sendCommand("federation.sync.setPaused", message.payload);
      this.state.federation.paused = message.payload.paused;
      this.appendLog(`Federated sync ${message.payload.paused ? "paused" : "resumed"}`);
      this.postState();
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

    if (envelope.action === "nebula.validation.success" || envelope.action === "nebula.validation.failed") {
      this.state.validation = normalizeValidationResult(envelope.payload);
      this.appendLog(
        `Validation ${envelope.action.endsWith("success") ? "passed" : "failed"}: ${this.state.validation.artifact_ref}`
      );
      this.postState();
      return;
    }

    if (envelope.action === "nebula.deployment.started") {
      const payload = envelope.payload as Partial<{ artifact: string; status: string }>;
      this.state.deploymentStatus = `${payload.status ?? "deploying"}: ${payload.artifact ?? "unknown artifact"}`;
      this.postState();
      return;
    }

    if (envelope.action === "nebula.federation.peer") {
      const payload = envelope.payload as Partial<{ nodeId: string; node_id: string; recordCount: number; record_count: number }>;
      const nodeId = payload.nodeId ?? payload.node_id ?? "unknown";
      const recordCount = payload.recordCount ?? payload.record_count ?? 0;
      this.state.federation.peers = [
        { nodeId, recordCount, status: "active" },
        ...this.state.federation.peers.filter((peer) => peer.nodeId !== nodeId)
      ].slice(0, 12);
      this.postState();
      return;
    }

    if (envelope.action === "nebula.federation.contribution") {
      const payload = envelope.payload as Partial<{ source: string; rows: number }>;
      const source = payload.source ?? "remote";
      const rows = payload.rows ?? 0;
      const existing = this.state.federation.contributions.find((item) => item.source === source);
      if (existing) {
        existing.rows += rows;
      } else {
        this.state.federation.contributions = [{ source, rows }, ...this.state.federation.contributions].slice(0, 12);
      }
      this.postState();
      return;
    }

    if (envelope.action === "nebula.federation.status") {
      const payload = envelope.payload as Partial<{ paused: boolean }>;
      this.state.federation.paused = payload.paused ?? this.state.federation.paused;
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

function normalizeValidationResult(payload: unknown): ValidationResult {
  const result = payload as Partial<ValidationResult>;
  return {
    artifact_ref: typeof result.artifact_ref === "string" ? result.artifact_ref : "",
    output_model: typeof result.output_model === "string" ? result.output_model : "",
    pass_rate: typeof result.pass_rate === "number" ? result.pass_rate : 0,
    samples: Array.isArray(result.samples) ? result.samples : []
  };
}
