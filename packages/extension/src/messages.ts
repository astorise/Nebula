export type NebulaMessageType = "EVENT" | "COMMAND";

export interface NebulaEnvelope<TPayload = unknown> {
  type: NebulaMessageType;
  action: string;
  payload: TPayload;
}

export interface DatasetState {
  total: number;
  escalated: number;
  direct: number;
}

export type TrainingStatus = "waiting" | "backward" | "merge" | "published";

export interface ValidationSample {
  prompt: string;
  before: string[];
  after: string[];
  diverged: boolean;
}

export interface ValidationResult {
  artifact_ref: string;
  output_model: string;
  pass_rate: number;
  samples: ValidationSample[];
}

export interface FederationPeer {
  nodeId: string;
  recordCount: number;
  status: string;
}

export interface FederationContribution {
  source: string;
  rows: number;
}

export interface FederationState {
  paused: boolean;
  peers: FederationPeer[];
  contributions: FederationContribution[];
}

export interface ArtifactVariant {
  title: string;
  artifact: string;
  sizeBytes: number;
  minVramGb: number;
}

export interface DeploymentArtifacts {
  hostVramGb: number;
  maxVariant?: string;
  variants: ArtifactVariant[];
}

export interface DriftMetric {
  topic: string;
  confidenceScore: number;
  threshold: number;
  sampleCount: number;
  uncertainCount: number;
}

export interface DriftState {
  metrics: DriftMetric[];
  triggers: DriftMetric[];
}

export interface CanaryMetric {
  modelVersion: string;
  rolloutTrack: string;
  divergenceRate: number;
  threshold: number;
  rollback: boolean;
}

export interface CanaryState {
  status: "healthy" | "rollback" | "unknown";
  metrics: CanaryMetric[];
}

export interface PrivacyState {
  sandboxInput: string;
  sandboxOutput: string;
  totalMasked: number;
  byRule: Record<string, number>;
}

export interface AlignmentState {
  rules: string[];
  pendingPreferences: Array<{ prompt: string; chosen: string; rejected: string }>;
}

export interface TenantState {
  activeTenantId: string;
  tenants: Array<{ tenantId: string; rows: number; quota: number }>;
}

export interface GoldenState {
  replayRatio: number;
  rows: Array<{ prompt: string; answer: string; locked: boolean }>;
}

export interface FoundryState {
  pendingTools: Array<{ toolId: string; capability: string; status: string }>;
}

export interface FinOpsState {
  dailyCostUsd: number;
  monthlyCostUsd: number;
  tokenBudget: number;
  tokensUsed: number;
  tokensSaved: number;
  deduplicatedRequests: number;
}

export interface DashboardState {
  connectionStatus: string;
  dataset: DatasetState;
  trainingStatus: TrainingStatus;
  validation?: ValidationResult;
  deploymentStatus?: string;
  deploymentArtifacts: DeploymentArtifacts;
  canary: CanaryState;
  privacy: PrivacyState;
  alignment: AlignmentState;
  tenants: TenantState;
  golden: GoldenState;
  foundry: FoundryState;
  finops: FinOpsState;
  drift: DriftState;
  federation: FederationState;
  logs: string[];
}

export interface CurriculumCommand {
  subject: string;
  count: number;
}

export type WebviewToExtensionMessage =
  | { type: "COMMAND"; action: "curriculum.generate"; payload: CurriculumCommand }
  | { type: "COMMAND"; action: "training.forceMerge"; payload: Record<string, never> }
  | { type: "COMMAND"; action: "DEPLOY_LORA"; payload: { artifact: string } }
  | { type: "COMMAND"; action: "federation.sync.setPaused"; payload: { paused: boolean } }
  | { type: "COMMAND"; action: "deployment.variant.setMax"; payload: { maxVariant: string } }
  | { type: "COMMAND"; action: "privacy.sandbox.test"; payload: { text: string } }
  | { type: "COMMAND"; action: "alignment.constitution.save"; payload: { rules: string[] } }
  | { type: "COMMAND"; action: "alignment.preference.review"; payload: { accepted: boolean; prompt: string } }
  | { type: "COMMAND"; action: "tenant.setActive"; payload: { tenantId: string } }
  | { type: "COMMAND"; action: "tenant.purge"; payload: { tenantId: string } }
  | { type: "COMMAND"; action: "golden.replayRatio.set"; payload: { ratio: number } }
  | { type: "COMMAND"; action: "golden.pin"; payload: { prompt: string; locked: boolean } }
  | { type: "COMMAND"; action: "foundry.approve"; payload: { toolId: string } }
  | { type: "COMMAND"; action: "finops.budget.set"; payload: { tenantId: string; tokenBudget: number } };

export type ExtensionToWebviewMessage =
  | { type: "STATE"; payload: DashboardState }
  | { type: "LOG"; payload: string };
