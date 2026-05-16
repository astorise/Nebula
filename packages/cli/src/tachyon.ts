import { EventEmitter } from "node:events";

export interface TachyonMessage {
  type: string;
  payload?: unknown;
  requestId?: string;
}

export interface TachyonRouter {
  route(message: TachyonMessage): Promise<TachyonMessage>;
  onEvent(listener: (message: TachyonMessage) => void): () => void;
  emitEvent(message: TachyonMessage): void;
}

export class StubTachyonRouter implements TachyonRouter {
  private readonly events = new EventEmitter();

  async route(message: TachyonMessage): Promise<TachyonMessage> {
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
}
