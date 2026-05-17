interface DatasetState {
  total: number;
  escalated: number;
  direct: number;
}

interface DashboardState {
  connectionStatus: string;
  dataset: DatasetState;
  trainingStatus: "waiting" | "backward" | "merge" | "published";
  validation?: ValidationResult;
  deploymentStatus?: string;
  deploymentArtifacts: DeploymentArtifacts;
  canary: CanaryState;
  privacy: PrivacyState;
  drift: DriftState;
  federation: FederationState;
  logs: string[];
}

interface ValidationSample {
  prompt: string;
  before: string[];
  after: string[];
  diverged: boolean;
}

interface ValidationResult {
  artifact_ref: string;
  output_model: string;
  pass_rate: number;
  samples: ValidationSample[];
}

interface FederationState {
  paused: boolean;
  peers: FederationPeer[];
  contributions: FederationContribution[];
}

interface FederationPeer {
  nodeId: string;
  recordCount: number;
  status: string;
}

interface FederationContribution {
  source: string;
  rows: number;
}

interface DeploymentArtifacts {
  hostVramGb: number;
  maxVariant?: string;
  variants: ArtifactVariant[];
}

interface ArtifactVariant {
  title: string;
  artifact: string;
  sizeBytes: number;
  minVramGb: number;
}

interface DriftState {
  metrics: DriftMetric[];
  triggers: DriftMetric[];
}

interface DriftMetric {
  topic: string;
  confidenceScore: number;
  threshold: number;
  sampleCount: number;
  uncertainCount: number;
}

interface CanaryState {
  status: "healthy" | "rollback" | "unknown";
  metrics: CanaryMetric[];
}

interface CanaryMetric {
  modelVersion: string;
  rolloutTrack: string;
  divergenceRate: number;
  threshold: number;
  rollback: boolean;
}

interface PrivacyState {
  sandboxInput: string;
  sandboxOutput: string;
  totalMasked: number;
  byRule: Record<string, number>;
}

interface VsCodeApi {
  postMessage(message: unknown): void;
}

declare const acquireVsCodeApi: () => VsCodeApi;

const vscode = acquireVsCodeApi();
const root = document.getElementById("root");

let state: DashboardState = {
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

window.addEventListener("message", ({ data }) => {
  if (data.type === "STATE") {
    state = data.payload;
    render();
  }
});

function render(): void {
  if (!root) {
    return;
  }

  const escalatedPercent = state.dataset.total === 0 ? 0 : Math.round((state.dataset.escalated / state.dataset.total) * 100);
  const directPercent = state.dataset.total === 0 ? 0 : 100 - escalatedPercent;
  const validation = state.validation;
  const primarySample = validation?.samples[0];
  const passRate = validation ? Math.round(validation.pass_rate * 100) : 0;

  root.innerHTML = `
    <main>
      <header>
        <div>
          <h1>Nebula</h1>
          <p>Teacher forge dashboard</p>
        </div>
        <span class="status">${escapeHtml(state.connectionStatus)}</span>
      </header>
      <section class="grid">
        <article>
          <h2>Dataset</h2>
          <div class="metric">${state.dataset.total}</div>
          <div class="ratio" aria-label="Dataset ratio">
            <span class="escalated" style="width:${escalatedPercent}%"></span>
            <span class="direct" style="width:${directPercent}%"></span>
          </div>
          <div class="legend">
            <span>Escalated ${escalatedPercent}%</span>
            <span>Direct ${directPercent}%</span>
          </div>
        </article>
        <article>
          <h2>Training</h2>
          <ol class="steps">
            ${step("waiting", "Waiting")}
            ${step("backward", "Backward")}
            ${step("merge", "Merge")}
            ${step("published", "Published")}
          </ol>
          <button id="forceMerge" type="button">Force merge</button>
        </article>
      </section>
      <section class="grid">
        <article>
          <h2>Federation</h2>
          <div class="deployHeader">
            <p class="muted">${state.federation.paused ? "Sync paused" : "Sync active"}</p>
            <button id="toggleFederation" type="button">${state.federation.paused ? "Resume" : "Pause"}</button>
          </div>
          <div class="peers">
            ${
              state.federation.peers.length === 0
                ? `<p class="muted">No peers discovered</p>`
                : state.federation.peers
                    .map(
                      (peer) => `
                        <div class="peer">
                          <span>${escapeHtml(peer.nodeId)}</span>
                          <strong>${peer.recordCount}</strong>
                        </div>
                      `
                    )
                    .join("")
            }
          </div>
        </article>
        <article>
          <h2>Contributions</h2>
          <div class="peers">
            ${
              state.federation.contributions.length === 0
                ? `<p class="muted">Waiting for federated rows</p>`
                : state.federation.contributions
                    .map(
                      (item) => `
                        <div class="peer">
                          <span>${escapeHtml(item.source)}</span>
                          <strong>${item.rows}</strong>
                        </div>
                      `
                    )
                    .join("")
            }
          </div>
        </article>
      </section>
      <section class="grid">
        <article class="deployment">
          <h2>Privacy</h2>
          <div class="privacyGrid">
            <label>
              Sandbox
              <textarea id="privacyInput">${escapeHtml(state.privacy.sandboxInput)}</textarea>
            </label>
            <label>
              Masked
              <textarea readonly>${escapeHtml(state.privacy.sandboxOutput)}</textarea>
            </label>
          </div>
          <div class="deployHeader">
            <p class="muted">${state.privacy.totalMasked} entities masked</p>
            <button id="runPrivacySandbox" type="button">Run sandbox</button>
          </div>
          ${privacyRuleRows()}
        </article>
      </section>
      <section class="grid">
        <article class="deployment">
          <h2>Drift</h2>
          <div class="driftGrid">
            <div>
              <h3>Current Metrics</h3>
              ${driftRows(state.drift.metrics, "No drift metrics")}
            </div>
            <div>
              <h3>Retraining Triggers</h3>
              ${driftRows(state.drift.triggers, "No retraining triggers")}
            </div>
          </div>
        </article>
      </section>
      <section class="grid">
        <article class="deployment">
          <h2>Deployment</h2>
          ${
            validation
              ? `
                <div class="deployHeader">
                  <div>
                    <div class="metric">${passRate}%</div>
                    <p>${escapeHtml(validation.artifact_ref)}</p>
                  </div>
                  <button id="deployLora" type="button">Deploy to Swarm</button>
                </div>
                <div class="diff">
                  <div>
                    <h3>Before</h3>
                    <pre>${escapeHtml((primarySample?.before ?? []).join("\\n\\n"))}</pre>
                  </div>
                  <div>
                    <h3>After</h3>
                    <pre>${escapeHtml((primarySample?.after ?? []).join("\\n\\n"))}</pre>
                  </div>
                </div>
                <p class="muted">${escapeHtml(state.deploymentStatus ?? validation.output_model)}</p>
                ${canaryHealth()}
                ${artifactTable()}
              `
              : `<p class="muted">Waiting for LoRA validation results</p>`
          }
        </article>
        <article>
          <h2>Curriculum</h2>
          <form id="curriculumForm">
            <label>
              Subject
              <input id="subject" name="subject" type="text" value="Cobol" required>
            </label>
            <label>
              Exercises
              <input id="count" name="count" type="number" min="1" max="500" value="100" required>
            </label>
            <button type="submit">Launch evaluation</button>
          </form>
        </article>
        <article>
          <h2>Tier 3 Logs</h2>
          <div class="logs">${state.logs.map((log) => `<p>${escapeHtml(log)}</p>`).join("")}</div>
        </article>
      </section>
    </main>
  `;

  document.getElementById("curriculumForm")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const subject = (document.getElementById("subject") as HTMLInputElement).value;
    const count = Number.parseInt((document.getElementById("count") as HTMLInputElement).value, 10);
    vscode.postMessage({
      type: "COMMAND",
      action: "curriculum.generate",
      payload: { subject, count }
    });
  });

  document.getElementById("forceMerge")?.addEventListener("click", () => {
    vscode.postMessage({
      type: "COMMAND",
      action: "training.forceMerge",
      payload: {}
    });
  });

  document.getElementById("deployLora")?.addEventListener("click", () => {
    if (!state.validation?.artifact_ref) {
      return;
    }

    vscode.postMessage({
      type: "COMMAND",
      action: "DEPLOY_LORA",
      payload: { artifact: state.validation.artifact_ref }
    });
  });

  document.getElementById("toggleFederation")?.addEventListener("click", () => {
    vscode.postMessage({
      type: "COMMAND",
      action: "federation.sync.setPaused",
      payload: { paused: !state.federation.paused }
    });
  });

  document.getElementById("maxVariant")?.addEventListener("change", (event) => {
    const maxVariant = (event.target as HTMLSelectElement).value;
    vscode.postMessage({
      type: "COMMAND",
      action: "deployment.variant.setMax",
      payload: { maxVariant }
    });
  });

  document.getElementById("runPrivacySandbox")?.addEventListener("click", () => {
    const text = (document.getElementById("privacyInput") as HTMLTextAreaElement | null)?.value ?? "";
    vscode.postMessage({
      type: "COMMAND",
      action: "privacy.sandbox.test",
      payload: { text }
    });
  });
}

function step(value: DashboardState["trainingStatus"], label: string): string {
  const current = ["waiting", "backward", "merge", "published"].indexOf(state.trainingStatus);
  const index = ["waiting", "backward", "merge", "published"].indexOf(value);
  return `<li class="${index <= current ? "active" : ""}">${label}</li>`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function artifactTable(): string {
  const variants = state.deploymentArtifacts.variants;
  if (variants.length === 0) {
    return `<p class="muted">Waiting for quantized artifact metadata</p>`;
  }

  return `
    <div class="artifactTools">
      <label>
        Max variant
        <select id="maxVariant">
          ${variants
            .map(
              (variant) =>
                `<option value="${escapeHtml(variant.title)}" ${state.deploymentArtifacts.maxVariant === variant.title ? "selected" : ""}>${escapeHtml(variant.title)}</option>`
            )
            .join("")}
        </select>
      </label>
    </div>
    <table>
      <thead>
        <tr><th>Variant</th><th>Size</th><th>Min VRAM</th><th>Safety</th></tr>
      </thead>
      <tbody>
        ${variants
          .map(
            (variant) => `
              <tr>
                <td>${escapeHtml(variant.title)}</td>
                <td>${formatBytes(variant.sizeBytes)}</td>
                <td>${variant.minVramGb}GB</td>
                <td><span class="safety ${safetyClass(variant.minVramGb)}">${safetyLabel(variant.minVramGb)}</span></td>
              </tr>
            `
          )
          .join("")}
      </tbody>
    </table>
  `;
}

function canaryHealth(): string {
  if (state.canary.metrics.length === 0) {
    return `<p class="muted">Waiting for canary metrics</p>`;
  }

  return `
    <div>
      <h3>Canary Health</h3>
      <div class="peers">
        ${state.canary.metrics
          .map(
            (metric) => `
              <div class="peer">
                <span>${escapeHtml(metric.rolloutTrack)} · ${escapeHtml(metric.modelVersion)}</span>
                <strong class="${metric.rollback ? "rollback" : "healthy"}">${formatPercent(metric.divergenceRate)} / ${formatPercent(metric.threshold)}</strong>
              </div>
            `
          )
          .join("")}
      </div>
    </div>
  `;
}

function privacyRuleRows(): string {
  const entries = Object.entries(state.privacy.byRule);
  if (entries.length === 0) {
    return `<p class="muted">No masking activity yet</p>`;
  }

  return `
    <div class="peers">
      ${entries
        .map(
          ([rule, count]) => `
            <div class="peer">
              <span>${escapeHtml(rule)}</span>
              <strong>${count}</strong>
            </div>
          `
        )
        .join("")}
    </div>
  `;
}

function driftRows(metrics: DriftMetric[], empty: string): string {
  if (metrics.length === 0) {
    return `<p class="muted">${empty}</p>`;
  }

  return `
    <div class="peers">
      ${metrics
        .map(
          (metric) => `
            <div class="peer">
              <span>${escapeHtml(metric.topic)}</span>
              <strong>${Math.round(metric.confidenceScore * 100)}% / ${metric.sampleCount}</strong>
            </div>
          `
        )
        .join("")}
    </div>
  `;
}

function formatBytes(bytes: number): string {
  if (bytes >= 1_000_000_000) {
    return `${(bytes / 1_000_000_000).toFixed(1)}GB`;
  }
  if (bytes >= 1_000_000) {
    return `${(bytes / 1_000_000).toFixed(1)}MB`;
  }
  return `${bytes}B`;
}

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

function safetyLabel(minVramGb: number): string {
  const available = state.deploymentArtifacts.hostVramGb;
  if (available >= minVramGb + 4) {
    return "Green";
  }
  if (available >= minVramGb) {
    return "Yellow";
  }
  return "Red";
}

function safetyClass(minVramGb: number): string {
  return safetyLabel(minVramGb).toLowerCase();
}

const style = document.createElement("style");
style.textContent = `
  body { margin: 0; color: var(--vscode-foreground); background: var(--vscode-editor-background); font-family: var(--vscode-font-family); }
  main { padding: 20px; display: grid; gap: 16px; max-width: 1080px; }
  header { display: flex; justify-content: space-between; gap: 16px; align-items: flex-start; }
  h1, h2, p { margin: 0; }
  h1 { font-size: 28px; }
  h2 { font-size: 14px; text-transform: uppercase; color: var(--vscode-descriptionForeground); }
  h3 { margin: 0; font-size: 12px; color: var(--vscode-descriptionForeground); }
  .status { border: 1px solid var(--vscode-panel-border); padding: 4px 8px; font-size: 12px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; }
  article { border: 1px solid var(--vscode-panel-border); padding: 14px; display: grid; gap: 12px; }
  .deployment { grid-column: 1 / -1; }
  .deployHeader { display: flex; justify-content: space-between; gap: 12px; align-items: start; }
  .metric { font-size: 34px; font-weight: 700; }
  .ratio { height: 12px; display: flex; background: var(--vscode-input-background); overflow: hidden; }
  .ratio span { display: block; }
  .escalated { background: var(--vscode-charts-red); }
  .direct { background: var(--vscode-charts-green); }
  .legend { display: flex; justify-content: space-between; color: var(--vscode-descriptionForeground); font-size: 12px; }
  .steps { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 6px; padding: 0; margin: 0; list-style: none; }
  .steps li { border: 1px solid var(--vscode-panel-border); padding: 8px; text-align: center; font-size: 12px; }
  .steps li.active { background: var(--vscode-button-background); color: var(--vscode-button-foreground); }
  form { display: grid; gap: 10px; }
  label { display: grid; gap: 4px; }
  input { background: var(--vscode-input-background); color: var(--vscode-input-foreground); border: 1px solid var(--vscode-input-border); padding: 8px; }
  textarea { min-height: 120px; resize: vertical; background: var(--vscode-input-background); color: var(--vscode-input-foreground); border: 1px solid var(--vscode-input-border); padding: 8px; font-family: var(--vscode-editor-font-family); }
  select { background: var(--vscode-input-background); color: var(--vscode-input-foreground); border: 1px solid var(--vscode-input-border); padding: 8px; }
  button { background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: 0; padding: 9px 12px; cursor: pointer; }
  .diff { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 10px; }
  .driftGrid { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; }
  .privacyGrid { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; }
  .peers { display: grid; gap: 8px; }
  .peer { display: flex; justify-content: space-between; gap: 10px; border: 1px solid var(--vscode-panel-border); padding: 8px; }
  .peer span { overflow-wrap: anywhere; }
  pre { margin: 0; min-height: 120px; max-height: 240px; overflow: auto; white-space: pre-wrap; word-break: break-word; background: var(--vscode-input-background); border: 1px solid var(--vscode-panel-border); padding: 10px; font-family: var(--vscode-editor-font-family); font-size: 12px; }
  .muted { color: var(--vscode-descriptionForeground); }
  .artifactTools { display: flex; justify-content: flex-end; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th, td { border: 1px solid var(--vscode-panel-border); padding: 8px; text-align: left; }
  th { color: var(--vscode-descriptionForeground); font-weight: 600; }
  .safety { display: inline-block; min-width: 54px; text-align: center; padding: 3px 6px; }
  .green { background: var(--vscode-charts-green); color: var(--vscode-editor-background); }
  .yellow { background: var(--vscode-charts-yellow); color: var(--vscode-editor-background); }
  .red { background: var(--vscode-charts-red); color: var(--vscode-editor-background); }
  .healthy { color: var(--vscode-charts-green); }
  .rollback { color: var(--vscode-charts-red); }
  .logs { display: grid; gap: 6px; max-height: 260px; overflow: auto; }
  .logs p { font-family: var(--vscode-editor-font-family); font-size: 12px; padding: 6px; background: var(--vscode-input-background); }
`;
document.head.appendChild(style);
render();
