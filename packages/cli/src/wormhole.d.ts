declare module "@tachyon-mesh/wormhole" {
  export interface WormholeTarget {
    protocol: "tcp" | "udp";
    publicPort: number;
    localPort: number;
  }

  export interface WormholeOptions {
    relay: string;
    sni?: string;
    targets: WormholeTarget[];
    auth?: { cert: string; key: string };
    ca?: string;
    unsecure?: boolean;
  }

  export class Wormhole {
    readonly endpoint: string;
    static create(options: WormholeOptions): Promise<Wormhole>;
    close(): void;
  }
}
