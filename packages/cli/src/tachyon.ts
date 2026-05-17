import { EventEmitter } from "node:events";

export interface TachyonMessage {
  type: string;
  action?: string;
  payload?: unknown;
  requestId?: string;
}

export interface TachyonRouter {
  route(message: TachyonMessage): Promise<TachyonMessage>;
  onEvent(listener: (message: TachyonMessage) => void): () => void;
  emitEvent(message: TachyonMessage): void;
}

export interface TachyonConfigClient {
  deployLora(artifact: string): Promise<void>;
  listArtifactVariants(): Promise<ArtifactVariant[]>;
  listCanaryMetrics(): Promise<CanaryMetric[]>;
  testPrivacySandbox(text: string): Promise<PrivacySandboxResult>;
  setMaxVariant(maxVariant: string): Promise<void>;
}

export class StubTachyonConfigClient implements TachyonConfigClient {
  readonly deployments: string[] = [];
  readonly variantCeilings: string[] = [];
  private readonly variants: ArtifactVariant[] = [
    { title: "fp16", artifact: "oci://localhost:5000/pulsar-lora:v3-fp16", sizeBytes: 14_000_000_000, minVramGb: 16 },
    { title: "q8_0", artifact: "oci://localhost:5000/pulsar-lora:v3-q8_0", sizeBytes: 7_200_000_000, minVramGb: 8 },
    { title: "q4_k", artifact: "oci://localhost:5000/pulsar-lora:v3-q4_k", sizeBytes: 3_800_000_000, minVramGb: 4 }
  ];
  private readonly canaryMetrics: CanaryMetric[] = [
    { modelVersion: "pulsar-base:v2", rolloutTrack: "stable", divergenceRate: 0.012, threshold: 0.04, rollback: false },
    { modelVersion: "pulsar-base:v2-canary", rolloutTrack: "canary", divergenceRate: 0.018, threshold: 0.04, rollback: false }
  ];

  async deployLora(artifact: string): Promise<void> {
    this.deployments.push(artifact);
  }

  async listArtifactVariants(): Promise<ArtifactVariant[]> {
    return this.variants;
  }

  async listCanaryMetrics(): Promise<CanaryMetric[]> {
    return this.canaryMetrics;
  }

  async testPrivacySandbox(text: string): Promise<PrivacySandboxResult> {
    return runPrivacySandbox(text);
  }

  async setMaxVariant(maxVariant: string): Promise<void> {
    this.variantCeilings.push(maxVariant);
  }
}

export interface ArtifactVariant {
  title: string;
  artifact: string;
  sizeBytes: number;
  minVramGb: number;
}

export interface CanaryMetric {
  modelVersion: string;
  rolloutTrack: "stable" | "canary" | string;
  divergenceRate: number;
  threshold: number;
  rollback: boolean;
}

export interface PrivacySandboxResult {
  maskedText: string;
  totalMasked: number;
  byRule: Record<string, number>;
}

export class StubTachyonRouter implements TachyonRouter {
  private readonly events = new EventEmitter();

  constructor(private readonly config: TachyonConfigClient = new StubTachyonConfigClient()) {}

  async route(message: TachyonMessage): Promise<TachyonMessage> {
    if (message.action === "DEPLOY_LORA") {
      return this.deployLora(message);
    }

    if (message.action === "federation.sync.setPaused") {
      return this.setFederationPaused(message);
    }

    if (message.action === "deployment.artifacts.list") {
      return this.listArtifactVariants(message);
    }

    if (message.action === "deployment.canary.metrics") {
      return this.listCanaryMetrics(message);
    }

    if (message.action === "deployment.variant.setMax") {
      return this.setMaxVariant(message);
    }

    if (message.action === "privacy.sandbox.test") {
      return this.testPrivacySandbox(message);
    }

    const routed: TachyonMessage = {
      type: "tachyon.stub.routed",
      requestId: message.requestId,
      payload: {
        originalType: message.type,
        accepted: true
      }
    };

    queueMicrotask(() => this.events.emit("event", routed));
    return routed;
  }

  onEvent(listener: (message: TachyonMessage) => void): () => void {
    this.events.on("event", listener);
    return () => this.events.off("event", listener);
  }

  emitEvent(message: TachyonMessage): void {
    this.events.emit("event", message);
  }

  private async deployLora(message: TachyonMessage): Promise<TachyonMessage> {
    const artifact = readArtifact(message.payload);
    await this.config.deployLora(artifact);

    const event: TachyonMessage = {
      type: "EVENT",
      action: "nebula.deployment.started",
      requestId: message.requestId,
      payload: {
        artifact,
        status: "deploying"
      }
    };
    queueMicrotask(() => this.events.emit("event", event));

    return {
      type: "tachyon.config.updated",
      action: "DEPLOY_LORA",
      requestId: message.requestId,
      payload: {
        artifact,
        accepted: true
      }
    };
  }

  private setFederationPaused(message: TachyonMessage): TachyonMessage {
    const paused = readPaused(message.payload);
    const event: TachyonMessage = {
      type: "EVENT",
      action: "nebula.federation.status",
      requestId: message.requestId,
      payload: { paused }
    };
    queueMicrotask(() => this.events.emit("event", event));

    return {
      type: "tachyon.config.updated",
      action: "federation.sync.setPaused",
      requestId: message.requestId,
      payload: {
        paused,
        accepted: true
      }
    };
  }

  private async listArtifactVariants(message: TachyonMessage): Promise<TachyonMessage> {
    const variants = await this.config.listArtifactVariants();
    const event: TachyonMessage = {
      type: "EVENT",
      action: "nebula.deployment.artifacts",
      requestId: message.requestId,
      payload: {
        hostVramGb: 8,
        variants
      }
    };
    queueMicrotask(() => this.events.emit("event", event));
    return event;
  }

  private async listCanaryMetrics(message: TachyonMessage): Promise<TachyonMessage> {
    const metrics = await this.config.listCanaryMetrics();
    const event: TachyonMessage = {
      type: "EVENT",
      action: "nebula.canary.metrics",
      requestId: message.requestId,
      payload: {
        metrics,
        status: metrics.some((metric) => metric.rollback) ? "rollback" : "healthy"
      }
    };
    queueMicrotask(() => this.events.emit("event", event));
    return event;
  }

  private async setMaxVariant(message: TachyonMessage): Promise<TachyonMessage> {
    const maxVariant = readMaxVariant(message.payload);
    await this.config.setMaxVariant(maxVariant);

    const event: TachyonMessage = {
      type: "EVENT",
      action: "nebula.deployment.variant_ceiling",
      requestId: message.requestId,
      payload: { maxVariant }
    };
    queueMicrotask(() => this.events.emit("event", event));

    return {
      type: "tachyon.config.updated",
      action: "deployment.variant.setMax",
      requestId: message.requestId,
      payload: {
        maxVariant,
        accepted: true
      }
    };
  }

  private async testPrivacySandbox(message: TachyonMessage): Promise<TachyonMessage> {
    const text = readText(message.payload);
    const result = await this.config.testPrivacySandbox(text);
    const event: TachyonMessage = {
      type: "EVENT",
      action: "nebula.privacy.sandbox.result",
      requestId: message.requestId,
      payload: result
    };
    queueMicrotask(() => this.events.emit("event", event));
    return event;
  }
}

function readArtifact(payload: unknown): string {
  if (
    typeof payload === "object" &&
    payload !== null &&
    "artifact" in payload &&
    typeof payload.artifact === "string" &&
    payload.artifact.length > 0
  ) {
    return payload.artifact;
  }

  throw new Error("DEPLOY_LORA requires payload.artifact");
}

function readPaused(payload: unknown): boolean {
  if (
    typeof payload === "object" &&
    payload !== null &&
    "paused" in payload &&
    typeof payload.paused === "boolean"
  ) {
    return payload.paused;
  }

  throw new Error("federation.sync.setPaused requires payload.paused");
}

function readMaxVariant(payload: unknown): string {
  if (
    typeof payload === "object" &&
    payload !== null &&
    "maxVariant" in payload &&
    typeof payload.maxVariant === "string" &&
    payload.maxVariant.length > 0
  ) {
    return payload.maxVariant;
  }

  throw new Error("deployment.variant.setMax requires payload.maxVariant");
}

function readText(payload: unknown): string {
  if (
    typeof payload === "object" &&
    payload !== null &&
    "text" in payload &&
    typeof payload.text === "string"
  ) {
    return payload.text;
  }

  throw new Error("privacy.sandbox.test requires payload.text");
}

function runPrivacySandbox(text: string): PrivacySandboxResult {
  const rules: Array<{ name: string; token: string; regex: RegExp }> = [
    { name: "email", token: "<EMAIL>", regex: /\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b/gi },
    { name: "bearer_token", token: "<BEARER_TOKEN>", regex: /\bBearer\s+[A-Za-z0-9._~+/=-]{16,}\b/gi },
    { name: "jwt", token: "<JWT>", regex: /\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b/g },
    { name: "ipv4", token: "<IPV4>", regex: /\b(?:\d{1,3}\.){3}\d{1,3}\b/g },
    { name: "uuid", token: "<UUID>", regex: /\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/gi },
    { name: "credit_card", token: "<PAYMENT_CARD>", regex: /\b(?:\d[ -]*?){13,19}\b/g }
  ];
  let maskedText = text;
  const byRule: Record<string, number> = {};

  for (const rule of rules) {
    const matches = maskedText.match(rule.regex);
    const count = matches?.length ?? 0;
    if (count === 0) {
      continue;
    }
    byRule[rule.name] = (byRule[rule.name] ?? 0) + count;
    maskedText = maskedText.replace(rule.regex, rule.token);
  }

  return {
    maskedText,
    totalMasked: Object.values(byRule).reduce((sum, count) => sum + count, 0),
    byRule
  };
}
