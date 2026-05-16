import fs from "node:fs";
import * as vscode from "vscode";
import WebSocket from "ws";

export interface NebulaConnectionConfig {
  websocketUrl: string;
  keyPath: string;
  certPath: string;
  caPath: string;
}

export function readConnectionConfig(): NebulaConnectionConfig {
  const config = vscode.workspace.getConfiguration("nebula");

  return {
    websocketUrl: config.get<string>("websocketUrl", "wss://127.0.0.1:7443/ws"),
    keyPath: config.get<string>("tls.keyPath", ""),
    certPath: config.get<string>("tls.certPath", ""),
    caPath: config.get<string>("tls.caPath", "")
  };
}

export function createNebulaSocket(config: NebulaConnectionConfig): WebSocket {
  if (!config.keyPath || !config.certPath || !config.caPath) {
    throw new Error("Nebula mTLS settings are incomplete");
  }

  return new WebSocket(config.websocketUrl, {
    key: fs.readFileSync(config.keyPath),
    cert: fs.readFileSync(config.certPath),
    ca: fs.readFileSync(config.caPath),
    rejectUnauthorized: true
  });
}
