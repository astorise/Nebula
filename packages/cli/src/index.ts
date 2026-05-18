#!/usr/bin/env node
import https from "node:https";
import { loadConfig } from "./config";
import { loadTlsOptions } from "./security";
import { StubTachyonRouter } from "./tachyon";
import { fallbackTunnelHost, publishTunnelStatus, startWormholeTunnel, type WormholeTunnel } from "./tunnel";
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

  let tunnel: WormholeTunnel | undefined;
  let tunnelHost = fallbackTunnelHost(process.env.NEBULA_TENANT_ID);
  try {
    tunnel = await startWormholeTunnel({
      localHost: config.host,
      localPort: config.port,
      router,
      tenantId: process.env.NEBULA_TENANT_ID,
      sessionToken: process.env.NEBULA_SESSION_TOKEN
    });
    tunnelHost = tunnel.host;
  } catch (error) {
    publishTunnelStatus(router, "error", tunnelHost);
    console.warn(error instanceof Error ? error.message : "Unable to start Wormhole tunnel");
  }

  const watcher = watchWebDavRoot(config.docsRoot, router, tunnelHost);
  server.on("close", () => {
    watcher.close();
    void tunnel?.close();
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
