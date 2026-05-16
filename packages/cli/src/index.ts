#!/usr/bin/env node
import https from "node:https";
import { loadConfig } from "./config";
import { loadTlsOptions } from "./security";
import { StubTachyonRouter } from "./tachyon";
import { handleWebDav } from "./webdav";
import { attachWebSocketBridge } from "./websocket";

export async function startNebulaCli(): Promise<https.Server> {
  const config = loadConfig();
  const tlsOptions = loadTlsOptions(config.tls);
  const router = new StubTachyonRouter();

  const server = https.createServer(tlsOptions, (req, res) => {
    void handleWebDav(req, res, { docsRoot: config.docsRoot });
  });

  attachWebSocketBridge(server, router);

  await new Promise<void>((resolve) => {
    server.listen(config.port, config.host, resolve);
  });

  console.log(`Nebula CLI listening on https://${config.host}:${config.port}`);
  return server;
}

if (require.main === module) {
  startNebulaCli().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
