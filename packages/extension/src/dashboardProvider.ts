import * as vscode from "vscode";
import { randomBytes } from "node:crypto";
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
    deploymentArtifacts: {
      hostVramGb: 8,
      variants: []
    },
    canary: {
      status: "unknown",
      metrics: []
    },
    privacy: {
      sandboxInput: "",
      sandboxOutput: "",
      totalMasked: 0,
      byRule: {}
    },
    alignment: {
      rules: ["Do not use unwrap() in generated Rust.", "Prefer fallible APIs and explicit errors."],
      pendingPreferences: []
    },
    tenants: {
      activeTenantId: "default",
      tenants: []
    },
    golden: {
      replayRatio: 0.2,
      rows: []
    },
    foundry: {
      pendingTools: []
    },
    finops: {
      dailyCostUsd: 0,
      monthlyCostUsd: 0,
      tokenBudget: 100_000,
      tokensUsed: 0,
      tokensSaved: 0,
      deduplicatedRequests: 0
    },
    drift: {
      metrics: [],
      triggers: []
    },
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
        this.sendCommand("deployment.artifacts.list", {});
        this.sendCommand("deployment.canary.metrics", {});
        this.sendCommand("tenant.list", {});
        this.sendCommand("golden.list", {});
        this.sendCommand("finops.metrics", {});
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

    if (message.action === "deployment.variant.setMax") {
      this.sendCommand("deployment.variant.setMax", message.payload);
      this.state.deploymentArtifacts.maxVariant = message.payload.maxVariant;
      this.appendLog(`Deployment variant ceiling set to ${message.payload.maxVariant}`);
      this.postState();
    }

    if (message.action === "privacy.sandbox.test") {
      this.state.privacy.sandboxInput = message.payload.text;
      this.sendCommand("privacy.sandbox.test", message.payload);
      this.postState();
    }

    if (message.action === "alignment.constitution.save") {
      this.state.alignment.rules = message.payload.rules;
      this.sendCommand("alignment.constitution.save", message.payload);
      this.appendLog("Alignment constitution updated");
      this.postState();
    }

    if (message.action === "alignment.preference.review") {
      this.sendCommand("alignment.preference.review", message.payload);
      this.state.alignment.pendingPreferences = this.state.alignment.pendingPreferences.filter(
        (item) => item.prompt !== message.payload.prompt
      );
      this.postState();
    }

    if (message.action === "tenant.setActive") {
      this.state.tenants.activeTenantId = message.payload.tenantId;
      this.sendCommand("tenant.setActive", message.payload);
      this.appendLog(`Active tenant set to ${message.payload.tenantId}`);
      this.postState();
    }

    if (message.action === "tenant.purge") {
      this.sendCommand("tenant.purge", message.payload);
      this.appendLog(`Tenant purge requested: ${message.payload.tenantId}`);
    }

    if (message.action === "golden.replayRatio.set") {
      this.state.golden.replayRatio = message.payload.ratio;
      this.sendCommand("golden.replayRatio.set", message.payload);
      this.postState();
    }

    if (message.action === "golden.pin") {
      this.sendCommand("golden.pin", message.payload);
      this.state.golden.rows = this.state.golden.rows.map((row) =>
        row.prompt === message.payload.prompt ? { ...row, locked: message.payload.locked } : row
      );
      this.postState();
    }

    if (message.action === "foundry.approve") {
      this.sendCommand("foundry.approve", message.payload);
      this.state.foundry.pendingTools = this.state.foundry.pendingTools.map((tool) =>
        tool.toolId === message.payload.toolId ? { ...tool, status: "approved" } : tool
      );
      this.postState();
    }

    if (message.action === "finops.budget.set") {
      this.state.finops.tokenBudget = message.payload.tokenBudget;
      this.sendCommand("finops.budget.set", message.payload);
      this.appendLog(`FinOps budget updated for ${message.payload.tenantId}`);
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

    if (envelope.action === "nebula.quantization.completed") {
      const payload = envelope.payload as Partial<{ variants: unknown[] }>;
      this.state.deploymentArtifacts.variants = normalizeArtifactVariants(payload.variants);
      this.appendLog("Quantization complete: deployment variants updated");
      this.postState();
      return;
    }

    if (envelope.action === "nebula.deployment.artifacts") {
      const payload = envelope.payload as Partial<{ hostVramGb: number; variants: unknown[] }>;
      this.state.deploymentArtifacts = {
        ...this.state.deploymentArtifacts,
        hostVramGb: typeof payload.hostVramGb === "number" ? payload.hostVramGb : this.state.deploymentArtifacts.hostVramGb,
        variants: normalizeArtifactVariants(payload.variants)
      };
      this.postState();
      return;
    }

    if (envelope.action === "nebula.deployment.variant_ceiling") {
      const payload = envelope.payload as Partial<{ maxVariant: string }>;
      this.state.deploymentArtifacts.maxVariant = payload.maxVariant;
      this.postState();
      return;
    }

    if (envelope.action === "nebula.canary.metrics") {
      const payload = envelope.payload as Partial<{ metrics: unknown[]; status: "healthy" | "rollback" | "unknown" }>;
      this.state.canary = {
        status: payload.status ?? "unknown",
        metrics: normalizeCanaryMetrics(payload.metrics)
      };
      this.postState();
      return;
    }

    if (envelope.action === "nebula.privacy.metrics") {
      const payload = envelope.payload as Partial<{ totalMasked: number; total_masked: number; byRule: Record<string, number>; by_rule: Record<string, number> }>;
      this.state.privacy.totalMasked = payload.totalMasked ?? payload.total_masked ?? this.state.privacy.totalMasked;
      this.state.privacy.byRule = payload.byRule ?? payload.by_rule ?? this.state.privacy.byRule;
      this.postState();
      return;
    }

    if (envelope.action === "nebula.privacy.sandbox.result") {
      const payload = envelope.payload as Partial<{ maskedText: string; masked_text: string; totalMasked: number; total_masked: number; byRule: Record<string, number>; by_rule: Record<string, number> }>;
      this.state.privacy.sandboxOutput = payload.maskedText ?? payload.masked_text ?? "";
      this.state.privacy.totalMasked = payload.totalMasked ?? payload.total_masked ?? 0;
      this.state.privacy.byRule = payload.byRule ?? payload.by_rule ?? {};
      this.postState();
      return;
    }

    if (envelope.action === "nebula.drift.metrics") {
      this.state.drift.metrics = normalizeDriftMetrics(envelope.payload);
      this.postState();
      return;
    }

    if (envelope.action === "nebula.drift.detected") {
      const drift = normalizeDriftMetric(envelope.payload);
      this.state.drift.triggers = [drift, ...this.state.drift.triggers].slice(0, 10);
      this.state.drift.metrics = [drift, ...this.state.drift.metrics.filter((item) => item.topic !== drift.topic)].slice(0, 10);
      this.appendLog(`Drift detected: ${drift.topic} confidence ${Math.round(drift.confidenceScore * 100)}%`);
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

    if (envelope.action === "nebula.alignment.preference") {
      const payload = envelope.payload as Partial<{ prompt: string; chosen: string; rejected: string }>;
      this.state.alignment.pendingPreferences = [
        {
          prompt: payload.prompt ?? "unknown prompt",
          chosen: payload.chosen ?? "",
          rejected: payload.rejected ?? ""
        },
        ...this.state.alignment.pendingPreferences
      ].slice(0, 20);
      this.postState();
      return;
    }

    if (envelope.action === "nebula.tenant.list") {
      const payload = envelope.payload as Partial<{ activeTenantId: string; active_tenant_id: string; tenants: unknown[] }>;
      this.state.tenants = {
        activeTenantId: payload.activeTenantId ?? payload.active_tenant_id ?? this.state.tenants.activeTenantId,
        tenants: normalizeTenants(payload.tenants)
      };
      this.postState();
      return;
    }

    if (envelope.action === "nebula.golden.rows") {
      const payload = envelope.payload as Partial<{ replayRatio: number; replay_ratio: number; rows: unknown[] }>;
      this.state.golden = {
        replayRatio: payload.replayRatio ?? payload.replay_ratio ?? this.state.golden.replayRatio,
        rows: normalizeGoldenRows(payload.rows)
      };
      this.postState();
      return;
    }

    if (envelope.action === "nebula.foundry.approval_required") {
      const payload = envelope.payload as Partial<{ toolId: string; tool_id: string; capability: string }>;
      this.state.foundry.pendingTools = [
        {
          toolId: payload.toolId ?? payload.tool_id ?? "unknown",
          capability: payload.capability ?? "unknown",
          status: "pending"
        },
        ...this.state.foundry.pendingTools
      ].slice(0, 20);
      this.postState();
      return;
    }

    if (envelope.action === "nebula.finops.metrics") {
      this.state.finops = normalizeFinOps(envelope.payload, this.state.finops);
      this.postState();
      return;
    }

    if (envelope.action === "nebula.finops.deduplicated") {
      const payload = envelope.payload as Partial<{ savedTokens: number; saved_tokens: number }>;
      this.state.finops.deduplicatedRequests += 1;
      this.state.finops.tokensSaved += payload.savedTokens ?? payload.saved_tokens ?? 0;
      this.postState();
      return;
    }

    if (envelope.action === "nebula.finops.token_usage") {
      const payload = envelope.payload as Partial<{ totalTokens: number; total_tokens: number; costUsd: number; cost_usd: number }>;
      this.state.finops.tokensUsed += payload.totalTokens ?? payload.total_tokens ?? 0;
      this.state.finops.dailyCostUsd += payload.costUsd ?? payload.cost_usd ?? 0;
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
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource}; script-src 'nonce-${nonce}'; style-src 'nonce-${nonce}';">
  <title>Nebula</title>
</head>
<body>
  <div id="root" data-style-nonce="${nonce}"></div>
  <script nonce="${nonce}" src="${scriptUri}"></script>
</body>
</html>`;
  }
}

function getNonce(): string {
  return randomBytes(16).toString("base64");
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

function normalizeArtifactVariants(payload: unknown): DashboardState["deploymentArtifacts"]["variants"] {
  if (!Array.isArray(payload)) {
    return [];
  }

  return payload.map((item) => {
    const variant = item as Partial<{ title: string; artifact: string; artifact_ref: string; sizeBytes: number; size_bytes: number; minVramGb: number; min_vram_gb: number }>;
    return {
      title: typeof variant.title === "string" ? variant.title : "unknown",
      artifact: typeof variant.artifact === "string" ? variant.artifact : typeof variant.artifact_ref === "string" ? variant.artifact_ref : "",
      sizeBytes: typeof variant.sizeBytes === "number" ? variant.sizeBytes : typeof variant.size_bytes === "number" ? variant.size_bytes : 0,
      minVramGb: typeof variant.minVramGb === "number" ? variant.minVramGb : typeof variant.min_vram_gb === "number" ? variant.min_vram_gb : 0
    };
  });
}

function normalizeDriftMetrics(payload: unknown): DashboardState["drift"]["metrics"] {
  if (!Array.isArray(payload)) {
    return [];
  }

  return payload.map(normalizeDriftMetric);
}

function normalizeDriftMetric(payload: unknown): DashboardState["drift"]["metrics"][number] {
  const metric = payload as Partial<{
    topic: string;
    confidenceScore: number;
    confidence_score: number;
    threshold: number;
    sampleCount: number;
    sample_count: number;
    uncertainCount: number;
    uncertain_count: number;
  }>;
  return {
    topic: typeof metric.topic === "string" ? metric.topic : "unknown",
    confidenceScore:
      typeof metric.confidenceScore === "number"
        ? metric.confidenceScore
        : typeof metric.confidence_score === "number"
          ? metric.confidence_score
          : 0,
    threshold: typeof metric.threshold === "number" ? metric.threshold : 0,
    sampleCount:
      typeof metric.sampleCount === "number"
        ? metric.sampleCount
        : typeof metric.sample_count === "number"
          ? metric.sample_count
          : 0,
    uncertainCount:
      typeof metric.uncertainCount === "number"
        ? metric.uncertainCount
        : typeof metric.uncertain_count === "number"
          ? metric.uncertain_count
          : 0
  };
}

function normalizeCanaryMetrics(payload: unknown): DashboardState["canary"]["metrics"] {
  if (!Array.isArray(payload)) {
    return [];
  }

  return payload.map((item) => {
    const metric = item as Partial<{
      modelVersion: string;
      model_version: string;
      rolloutTrack: string;
      rollout_track: string;
      divergenceRate: number;
      divergence_rate: number;
      threshold: number;
      rollback: boolean;
    }>;
    return {
      modelVersion: typeof metric.modelVersion === "string" ? metric.modelVersion : typeof metric.model_version === "string" ? metric.model_version : "unknown",
      rolloutTrack: typeof metric.rolloutTrack === "string" ? metric.rolloutTrack : typeof metric.rollout_track === "string" ? metric.rollout_track : "unknown",
      divergenceRate: typeof metric.divergenceRate === "number" ? metric.divergenceRate : typeof metric.divergence_rate === "number" ? metric.divergence_rate : 0,
      threshold: typeof metric.threshold === "number" ? metric.threshold : 0,
      rollback: metric.rollback === true
    };
  });
}

function normalizeTenants(payload: unknown): DashboardState["tenants"]["tenants"] {
  if (!Array.isArray(payload)) {
    return [];
  }

  return payload.map((item) => {
    const tenant = item as Partial<{ tenantId: string; tenant_id: string; rows: number; quota: number }>;
    return {
      tenantId: tenant.tenantId ?? tenant.tenant_id ?? "default",
      rows: tenant.rows ?? 0,
      quota: tenant.quota ?? 50_000
    };
  });
}

function normalizeGoldenRows(payload: unknown): DashboardState["golden"]["rows"] {
  if (!Array.isArray(payload)) {
    return [];
  }

  return payload.map((item) => {
    const row = item as Partial<{ prompt: string; answer: string; locked: boolean }>;
    return {
      prompt: row.prompt ?? "",
      answer: row.answer ?? "",
      locked: row.locked === true
    };
  });
}

function normalizeFinOps(payload: unknown, fallback: DashboardState["finops"]): DashboardState["finops"] {
  const metrics = payload as Partial<{
    dailyCostUsd: number;
    daily_cost_usd: number;
    monthlyCostUsd: number;
    monthly_cost_usd: number;
    tokenBudget: number;
    token_budget: number;
    tokensUsed: number;
    tokens_used: number;
    tokensSaved: number;
    tokens_saved: number;
    deduplicatedRequests: number;
    deduplicated_requests: number;
  }>;
  return {
    dailyCostUsd: metrics.dailyCostUsd ?? metrics.daily_cost_usd ?? fallback.dailyCostUsd,
    monthlyCostUsd: metrics.monthlyCostUsd ?? metrics.monthly_cost_usd ?? fallback.monthlyCostUsd,
    tokenBudget: metrics.tokenBudget ?? metrics.token_budget ?? fallback.tokenBudget,
    tokensUsed: metrics.tokensUsed ?? metrics.tokens_used ?? fallback.tokensUsed,
    tokensSaved: metrics.tokensSaved ?? metrics.tokens_saved ?? fallback.tokensSaved,
    deduplicatedRequests:
      metrics.deduplicatedRequests ?? metrics.deduplicated_requests ?? fallback.deduplicatedRequests
  };
}
