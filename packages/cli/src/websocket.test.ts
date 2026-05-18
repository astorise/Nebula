import { describe, expect, it } from "vitest";
import { isAuthorizedWebSocketClient, parseTachyonMessage } from "./websocket";

describe("parseTachyonMessage", () => {
  it("parses valid JSON messages", () => {
    expect(parseTachyonMessage('{"type":"REQUEST","action":"tenant.list","payload":{},"requestId":"r1"}')).toEqual({
      type: "REQUEST",
      action: "tenant.list",
      payload: {},
      requestId: "r1"
    });
  });

  it("rejects malformed JSON payloads", () => {
    expect(() => parseTachyonMessage("{bad json")).toThrow();
  });

  it("rejects structurally invalid messages", () => {
    expect(() => parseTachyonMessage('{"type":42,"payload":{}}')).toThrow("Invalid Tachyon message structure");
    expect(() => parseTachyonMessage('{"type":"REQUEST","requestId":42,"payload":{}}')).toThrow(
      "Invalid Tachyon message structure"
    );
  });
});

describe("isAuthorizedWebSocketClient", () => {
  it("rejects clients without a valid mTLS certificate", () => {
    expect(isAuthorizedWebSocketClient({ authorized: false } as never)).toBe(false);
  });
});
