# Design: AST Evaluator MicroVM

## Architecture

`nebula-eval-ast` becomes a native Linux binary executed inside a SmolVM microVM. The other FaaS components remain Wasm components.

The contract between the Tachyon mesh and the microVM is defined in `proto/ast_evaluator.proto`. The microVM binary uses `tonic` and listens on a Unix Domain Socket, defaulting to `/run/guest.sock`, which the host maps to the `virtio-vsock` transport.

## Flow

1. `nebula-telemetry-gateway` detects a code task.
2. It encodes `EvaluationRequest` as Protobuf and wraps it in an HTTP/2 gRPC frame.
3. The Tachyon core host routes the `wasi:http` request to the microVM socket.
4. The `nebula-eval-ast` server computes structural hashes and returns `EvaluationResponse`.
5. If parsing fails or the payload is invalid, `fallback_reason` allows the system to fall back to semantic evaluation.

## Packaging

`scripts/build-eval-ast-rootfs.sh` builds the release musl binary and creates a `rootfs.ext4` image with the binary as the startup process.

CI validates `nebula-eval-ast` natively and excludes this crate from Wasm clippy. The OCI script also ignores this crate because its artifact is a microVM rootfs, not a Wasm component.
