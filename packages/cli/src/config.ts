import path from "node:path";

export interface TlsFileConfig {
  keyPath: string;
  certPath: string;
  caPath: string;
}

export interface NebulaCliConfig {
  docsRoot: string;
  host: string;
  port: number;
  tls: TlsFileConfig;
}

export function loadConfig(env: NodeJS.ProcessEnv = process.env): NebulaCliConfig {
  const docsRoot = env.NEBULA_DOCS_ROOT ?? path.resolve(process.cwd(), "docs");
  const host = env.NEBULA_HOST ?? "127.0.0.1";
  const port = Number.parseInt(env.NEBULA_PORT ?? "7443", 10);

  if (!Number.isFinite(port) || port < 1 || port > 65535) {
    throw new Error("NEBULA_PORT must be a valid TCP port");
  }

  return {
    docsRoot: path.resolve(docsRoot),
    host,
    port,
    tls: {
      keyPath: required(env.NEBULA_TLS_KEY, "NEBULA_TLS_KEY"),
      certPath: required(env.NEBULA_TLS_CERT, "NEBULA_TLS_CERT"),
      caPath: required(env.NEBULA_TLS_CA, "NEBULA_TLS_CA")
    }
  };
}

function required(value: string | undefined, name: string): string {
  if (!value) {
    throw new Error(`${name} is required`);
  }

  return path.resolve(value);
}
