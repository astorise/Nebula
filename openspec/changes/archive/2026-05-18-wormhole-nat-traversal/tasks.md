# Implementation Tasks

- [x] `task-1`: Add the Astorise Wormhole client dependency through an installable local wrapper that records the source Git repository URL (`git+https://github.com/astorise/Wormhole.git`) in `packages/cli` metadata.
- [x] `task-2`: Create `packages/cli/src/tunnel.ts` to manage the lifecycle of the reverse tunnel tied to the local WebDAV port using the cloned client logic.
- [x] `task-3`: Update the `nebula.fs.file_updated` event schema to strictly require the `tunnel_host` property.
- [x] `task-4`: Refactor `nebula-doc-parser` to construct its `wasi:http` GET requests using the dynamic `tunnel_host`.
- [x] `task-5`: Implement the Wormhole connection status indicator in the VS Code Webview dashboard.
