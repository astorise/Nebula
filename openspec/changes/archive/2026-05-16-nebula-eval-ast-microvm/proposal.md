# Proposal: AST Parser through SmolVM and gRPC (virtio-vsock)

## Context

Syntax evaluation for LLMs (hallucination detection) requires the native C `tree-sitter` engine. Because the WASI environment is incompatible with that native dependency, the `nebula-eval-ast` component is packaged as a Linux microVM (SmolVM) orchestrated by Tachyon.

## Objectives

To ensure near-zero overhead for memory transfers between the WebAssembly mesh and the microVM, the network architecture relies on a `virtio-vsock` channel multiplexed through gRPC.

1. **Binary Protocol (Protobuf)**: Remove expensive JSON serialization by defining a strict Protobuf contract for sending LLM responses and receiving divergence status.
2. **gRPC Server (Tonic/UDS)**: Implement the Rust binary inside the microVM with `tonic`, configured to listen on a Unix Domain Socket (UDS) mapped to the VSOCK device.
3. **OCI Packaging**: Package this binary and shared C libraries into a `rootfs.ext4` filesystem distributed through the artifact registry.
