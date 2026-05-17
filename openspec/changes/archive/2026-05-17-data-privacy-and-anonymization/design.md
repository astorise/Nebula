# Design: Data Privacy and Anonymization

The anonymizer runs before telemetry reaches the existing gateway. It consumes `nebula.telemetry.raw_inferences`, applies deterministic regex-based compliance rules to the prompt and all responses, then republishes the sanitized triplet to `pulsar.telemetry.inference_triplets`.

Rules are represented as data so the default dictionary and future custom rules share the same engine. The Rust crate uses the `regex` crate and keeps audit counts per rule. Runtime integrations for event publishing and custom metrics remain trait-backed.

The telemetry gateway already accepts `pulsar.telemetry.inference_triplets`; tests are extended to reject the new raw topic directly so downstream components cannot accidentally bypass anonymization.

The VS Code dashboard adds a compact privacy panel to the existing webview. The CLI exposes a local sandbox command through the WebSocket router so developers can validate masking behavior without sending raw text outside the local process.
