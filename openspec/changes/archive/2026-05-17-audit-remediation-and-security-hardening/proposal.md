# Proposal: Security Hardening and Audit Remediation Sprint

## Context
A comprehensive architectural audit of the `main` branch (commits up to `df72e2d`) revealed severe security vulnerabilities and technical debt in the FaaS crates generated during the previous sprints. Critical flaws include cosmetic tenant isolation (leading to cross-tenant data corruption), trivial bypasses in the Constitutional AI judge, naive Markdown parsing for executable code synthesis, and unsafe CSP policies in the VS Code extension.

## Objectives
Execute a strict stabilization sprint focusing purely on security and robustness:
1. **Tenant Core Extraction**: Create a single source of truth for tenant validation (`nebula-tenant-core`) utilizing strict UUID mapping to prevent string collision attacks.
2. **Robust Pattern Matching**: Replace naive `String::contains()` checks in the DPO judge with strict, word-boundary-aware `regex::RegexSet` configurations.
3. **Deterministic Compilation & Extraction**: Refactor the tool synthesis pipeline to use structured extraction (Base64/JSON or Tree-sitter) and parse `cargo` diagnostic JSONs instead of brittle string matching.
4. **Extension Security**: Harden the VS Code Webview CSP, eliminating `unsafe-inline` vulnerabilities.
