# Proposal: Secure NAT Traversal via Astorise Wormhole

## Context
In Change 007 (Deterministic Document Ingestion), we introduced a local WebDAV server within the Nebula CLI to expose user documents to the training forge. However, this design contains a critical networking flaw: WebAssembly FaaS components running on remote Tachyon Edge nodes cannot route standard HTTP requests to a developer's local machine hidden behind a NAT router or corporate firewall.

To solve this ingress routing failure without requiring users to configure port forwarding, Nebula must integrate **Astorise Wormhole** (Repository: `https://github.com/astorise/Wormhole`) to establish a secure, outbound-only reverse tunnel.

## Objectives
1. **CLI Tunnel Client**: Embed the Wormhole client logic into the Nebula Node.js CLI. When the WebDAV server starts, the CLI automatically establishes a persistent reverse tunnel to the Tachyon mesh.
2. **Virtual Mesh Addressing**: Tachyon will assign a dynamic, tenant-isolated internal DNS alias (e.g., `webdav.tenant-{id}.wormhole.internal`) to the active tunnel.
3. **FaaS Realignment**: Update the `nebula-doc-parser` to target this virtual Wormhole address via `wasi:http` rather than attempting to route to a hardcoded local IP.
