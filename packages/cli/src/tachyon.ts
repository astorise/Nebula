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

  async deployLora(artifact: string): Promise<void> {
    this.deployments.push(artifact);
  }

  async listArtifactVariants(): Promise<ArtifactVariant[]> {
    return this.variants;
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

    if (message.action === "deployment.variant.setMax") {
      return this.setMaxVariant(message);
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
