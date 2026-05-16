import type { Server as HttpsServer } from "node:https";
import type { TLSSocket } from "node:tls";
import { WebSocketServer, type WebSocket } from "ws";
import { isAuthorizedClient } from "./security";
import type { TachyonMessage, TachyonRouter } from "./tachyon";

export function attachWebSocketBridge(server: HttpsServer, router: TachyonRouter): WebSocketServer {
  const wss = new WebSocketServer({ server, path: "/ws" });

  wss.on("connection", (socket, request) => {
    const tlsSocket = request.socket as TLSSocket;
    if (!isAuthorizedClient(tlsSocket)) {
      socket.close(1008, "mTLS client certificate required");
      return;
    }

    const unsubscribe = router.onEvent((message) => sendJson(socket, message));
    socket.on("close", unsubscribe);
    socket.on("message", async (raw) => {
      try {
        const message = JSON.parse(raw.toString()) as TachyonMessage;
        const response = await router.route(message);
        sendJson(socket, response);
      } catch (error) {
        sendJson(socket, {
          type: "nebula.error",
          payload: {
            message: error instanceof Error ? error.message : "Invalid message"
          }
        });
      }
    });
  });

  return wss;
}

function sendJson(socket: WebSocket, message: TachyonMessage): void {
  if (socket.readyState === socket.OPEN) {
    socket.send(JSON.stringify(message));
  }
}
