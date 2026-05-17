# Design: Drift Detection and Automated Retraining

The drift monitor is a Rust FaaS crate with pure sliding-window logic. Runtime storage and event publication are trait-backed so the implementation can use `tachyon:store/kv` and the event bus in production while remaining testable in CI.

Each inference metric is grouped by topic and evaluated over a bounded time window. A drift event is emitted only when the topic has enough samples and the uncertain ratio exceeds the configured threshold. The event carries the topic, confidence score, threshold, sample count, and uncertain count.

The curriculum generator consumes drift events by converting them into focused curriculum requests. Drift-triggered prompts carry a dedicated correlation header so dataset and retraining artifacts can be traced back to the detected drift.

The VS Code dashboard keeps recent drift alerts and current per-topic metrics in state. It renders a compact drift panel in the existing dashboard surface without introducing a separate webview component build path.
