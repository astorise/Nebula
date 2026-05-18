import type { Wormhole } from "@tachyon-mesh/wormhole";
import type { TachyonRouter } from "./tachyon";

export interface WormholeTunnel {
  host: string;
  close(): Promise<void> | void;
}

export interface WormholeTunnelOptions {
  localPort: number;
  router: TachyonRouter;
  relay: string;
  sni?: string;
}

export async function startWormholeTunnel(options: WormholeTunnelOptions): Promise<WormholeTunnel> {
  const WormholeClient = await loadWormholeClient();
  const tunnel: Wormhole = await WormholeClient.create({
    relay: options.relay,
    sni: options.sni,
    targets: [
      {
        protocol: "tcp",
        publicPort: 443,
        localPort: options.localPort
      }
    ]
  });
  const host = ingressUrlFromEndpoint(tunnel.endpoint, options.relay);

  publishTunnelStatus(options.router, "connected", host);
  return {
    host,
    close: async () => {
      await tunnel.close();
      publishTunnelStatus(options.router, "disconnected", host);
    }
  };
}

async function loadWormholeClient(): Promise<typeof import("@tachyon-mesh/wormhole").Wormhole> {
  const dynamicImport = new Function("specifier", "return import(specifier)") as <T>(specifier: string) => Promise<T>;
  const module = await dynamicImport<typeof import("@tachyon-mesh/wormhole")>("@tachyon-mesh/wormhole");
  return module.Wormhole;
}

export function ingressUrlFromEndpoint(endpoint: string, relay: string): string {
  const source = endpoint.startsWith("wormhole://") ? endpoint.replace("wormhole://", "https://") : endpoint;
  const parsed = new URL(source.includes("://") ? source : `https://${source}`);
  const relayHost = parseHost(relay);
  if (!parsed.hostname.endsWith(".wormhole.internal") && relayHost.endsWith(".wormhole.internal")) {
    return `https://${relayHost}`;
  }
  return `https://${parsed.hostname}`;
}

function parseHost(value: string): string {
  return new URL(value.includes("://") ? value : `https://${value}`).hostname;
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
