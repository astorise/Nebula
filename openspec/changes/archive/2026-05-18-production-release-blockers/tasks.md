# Implementation Tasks

- [x] `task-1`: Implement `fs.realpath` in the Node.js CLI WebDAV server to neutralize symlink path traversal.
- [x] `task-2`: Install `vitest` and write runtime tests proving WebDAV traversal prevention and mTLS rejection.
- [x] `task-3`: Remove the Wormhole stub and wire the real `@tachyon-mesh/wormhole` client, removing silent fallbacks.
- [x] `task-4`: Update the Redis LUA script in `reconcile_tokens` to prevent underflow (`math.max(0, val)`).
- [x] `task-5`: Implement `testcontainers-rs` in the Redis adapter and write a 10-thread concurrent atomicity test.
- [x] `task-6`: Secure `nebula-doc-parser` against SSRF by explicitly verifying the `.wormhole.internal` suffix.
- [x] `task-7`: Expand the `websocket.test.ts` suite to explicitly cover JSON payload parsing and structural validation.
- [x] `task-8`: Create a `KNOWN_LIMITATIONS.md` file at the root of the workspace detailing the aggressive behavior of the document parser heuristics.
