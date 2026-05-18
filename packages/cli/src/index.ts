#!/usr/bin/env node
import https from "node:https";
import { loadConfig } from "./config";
import { loadTlsOptions } from "./security";
import { StubTachyonRouter } from "./tachyon";
import { startWormholeTunnel, type WormholeTunnel } from "./tunnel";
import { handleWebDav, watchWebDavRoot } from "./webdav";
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

  const relay = process.env.NEBULA_WORMHOLE_RELAY;
  if (!relay) {
    throw new Error("NEBULA_WORMHOLE_RELAY is required to start the Wormhole tunnel");
  }

  const tunnel: WormholeTunnel = await startWormholeTunnel({
    localPort: config.port,
    router,
    relay,
    sni: process.env.NEBULA_WORMHOLE_SNI
  });

  const watcher = watchWebDavRoot(config.docsRoot, router, tunnel.host);
  server.on("close", () => {
    watcher.close();
    void tunnel.close();
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
