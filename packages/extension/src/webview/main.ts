interface DatasetState {
  total: number;
  escalated: number;
  direct: number;
}

interface DashboardState {
  connectionStatus: string;
  dataset: DatasetState;
  trainingStatus: "waiting" | "backward" | "merge" | "published";
  logs: string[];
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

const style = document.createElement("style");
style.textContent = `
  body { margin: 0; color: var(--vscode-foreground); background: var(--vscode-editor-background); font-family: var(--vscode-font-family); }
  main { padding: 20px; display: grid; gap: 16px; max-width: 1080px; }
  header { display: flex; justify-content: space-between; gap: 16px; align-items: flex-start; }
  h1, h2, p { margin: 0; }
  h1 { font-size: 28px; }
  h2 { font-size: 14px; text-transform: uppercase; color: var(--vscode-descriptionForeground); }
  .status { border: 1px solid var(--vscode-panel-border); padding: 4px 8px; font-size: 12px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; }
  article { border: 1px solid var(--vscode-panel-border); padding: 14px; display: grid; gap: 12px; }
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
  button { background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: 0; padding: 9px 12px; cursor: pointer; }
  .logs { display: grid; gap: 6px; max-height: 260px; overflow: auto; }
  .logs p { font-family: var(--vscode-editor-font-family); font-size: 12px; padding: 6px; background: var(--vscode-input-background); }
`;
document.head.appendChild(style);
render();
