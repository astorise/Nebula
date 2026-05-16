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

export interface DashboardState {
  connectionStatus: string;
  dataset: DatasetState;
  trainingStatus: TrainingStatus;
  logs: string[];
}

export interface CurriculumCommand {
  subject: string;
  count: number;
}

export type WebviewToExtensionMessage =
  | { type: "COMMAND"; action: "curriculum.generate"; payload: CurriculumCommand }
  | { type: "COMMAND"; action: "training.forceMerge"; payload: Record<string, never> };

export type ExtensionToWebviewMessage =
  | { type: "STATE"; payload: DashboardState }
  | { type: "LOG"; payload: string };
