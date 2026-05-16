# Implementation Tasks

- [x] `task-1`: Create `proto/ast_evaluator.proto` and configure `tonic-build` in the `nebula-eval-ast` crate `build.rs`.
- [x] `task-2`: Implement the gRPC server with `tonic`, listening specifically on `UnixListener` (`/run/guest.sock`).
- [x] `task-3`: Implement the business logic that calls `tree-sitter` to generate the hash from `EvaluationRequest`.
- [x] `task-4`: Write `build-rootfs.sh`, generating the `ext4` image packaged with the gRPC binary and native dependencies.
- [x] `task-5`: Update `nebula-telemetry-gateway` to generate Protobuf binary payloads (`prost`) and send them through `wasi:http`.
