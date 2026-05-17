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

export interface DashboardState {
  connectionStatus: string;
  dataset: DatasetState;
  trainingStatus: TrainingStatus;
  validation?: ValidationResult;
  deploymentStatus?: string;
  deploymentArtifacts: DeploymentArtifacts;
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
  | { type: "COMMAND"; action: "deployment.variant.setMax"; payload: { maxVariant: string } };

export type ExtensionToWebviewMessage =
  | { type: "STATE"; payload: DashboardState }
  | { type: "LOG"; payload: string };
