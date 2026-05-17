# Proposal: Autonomous Tool Synthesis and Wasm Compilation

## Context
Nebula successfully corrects hallucinations by injecting knowledge (SFT) and enforcing behavioral rules (DPO). However, many user queries fail not due to a lack of reasoning, but due to a lack of environmental capabilities (Tools/Function Calling). If a Pulsar agent needs to query an external API, read a database, or ping a network switch, and the tool does not exist, the agent will hallucinate. To achieve true autonomy, Nebula must be capable of recognizing missing tools, writing the source code for them, compiling them into WebAssembly components, and teaching the swarm how to use them.

## Objectives
Implement the "Tool Forge" pipeline:
1. **Tool Gap Detection (`nebula-tool-gap-analyzer`)**: Detect when a Tier 1 model hallucinates a tool call that is not in its allowed JSON schema.
2. **Code Synthesis (`nebula-tool-architect`)**: Prompt the Tier 3 Teacher (e.g., DeepSeek/Claude) to write the missing capability as a pure-Rust WebAssembly function adhering to Tachyon's Component Model constraints.
3. **Automated Compilation (`nebula-wasm-foundry`)**: Execute a secure MicroVM (SmolVM) containing a Rust toolchain to compile and verify the generated code safely.
4. **Swarm Injection**: Publish the new `.wasm` tool to the OCI registry and generate DPO training pairs to teach the Tier 1 model the exact JSON schema required to invoke its newly created tool.