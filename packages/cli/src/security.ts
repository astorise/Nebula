import fs from "node:fs";
import type { TLSSocket } from "node:tls";
import type { TlsFileConfig } from "./config";

export function loadTlsOptions(config: TlsFileConfig) {
  return {
    key: fs.readFileSync(config.keyPath),
    cert: fs.readFileSync(config.certPath),
    ca: fs.readFileSync(config.caPath),
    requestCert: true,
    rejectUnauthorized: true
  };
}

export function isAuthorizedClient(socket: TLSSocket): boolean {
  return socket.authorized === true;
}
