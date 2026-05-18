import { createRequire } from "node:module";
import type { TachyonRouter } from "./tachyon";

const requireFromHere = createRequire(__filename);

export interface WormholeTunnel {
  host: string;
  close(): Promise<void> | void;
}

export interface WormholeTunnelOptions {
  localHost: string;
  localPort: number;
  router: TachyonRouter;
  tenantId?: string;
  sessionToken?: string;
}

type WormholeClientModule = {
  connect?: (options: Record<string, unknown>) => Promise<WormholeTunnel> | WormholeTunnel;
  createTunnel?: (options: Record<string, unknown>) => Promise<WormholeTunnel> | WormholeTunnel;
  startTunnel?: (options: Record<string, unknown>) => Promise<WormholeTunnel> | WormholeTunnel;
};

export async function startWormholeTunnel(options: WormholeTunnelOptions): Promise<WormholeTunnel> {
  const wormhole = loadWormholeClient();
  const connect = wormhole.connect ?? wormhole.createTunnel ?? wormhole.startTunnel;
  if (!connect) {
    throw new Error("wormhole-tunnel does not expose connect/createTunnel/startTunnel");
  }

  const tunnel = await connect({
    localHost: options.localHost,
    localPort: options.localPort,
    tenantId: options.tenantId,
    sessionToken: options.sessionToken
  });

  publishTunnelStatus(options.router, "connected", tunnel.host);
  return {
    host: tunnel.host,
    close: async () => {
      await tunnel.close();
      publishTunnelStatus(options.router, "disconnected", tunnel.host);
    }
  };
}

export function fallbackTunnelHost(tenantId = "default"): string {
  return `https://webdav.tenant-${tenantId}.wormhole.internal`;
}

export function publishTunnelStatus(router: TachyonRouter, status: "connected" | "disconnected" | "error", host?: string): void {
  router.emitEvent({
    type: "EVENT",
    payload: {
      topic: "nebula.wormhole.status",
      payload: { status, host }
    }
  });
}

function loadWormholeClient(): WormholeClientModule {
  return requireFromHere("wormhole-tunnel") as WormholeClientModule;
}
