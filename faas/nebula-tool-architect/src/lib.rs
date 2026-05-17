use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const INPUT_TOPIC: &str = "nebula.eval.missing_tool_detected";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissingToolEvent {
    pub capability: String,
    pub prompt: String,
    pub wit_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratedTool {
    pub name: String,
    pub lib_rs: String,
    pub cargo_toml: String,
}

pub trait Tier3Model {
    fn generate_tool(&self, prompt: &str) -> Result<String>;
}

pub fn build_tool_prompt(event: &MissingToolEvent) -> String {
    format!(
        "Generate a Rust Wasm component for capability '{}'.\nWIT context:\n{}\nUser need:\n{}",
        event.capability, event.wit_context, event.prompt
    )
}

pub fn architect_tool(model: &impl Tier3Model, event: &MissingToolEvent) -> Result<GeneratedTool> {
    let response = model.generate_tool(&build_tool_prompt(event))?;
    extract_generated_tool(&event.capability, &response)
}

pub fn extract_generated_tool(name: &str, response: &str) -> Result<GeneratedTool> {
    let lib_rs = extract_block(response, "src/lib.rs")?;
    let cargo_toml = extract_block(response, "Cargo.toml")?;
    Ok(GeneratedTool {
        name: name.into(),
        lib_rs,
        cargo_toml,
    })
}

fn extract_block(response: &str, marker: &str) -> Result<String> {
    let start = response
        .find(marker)
        .ok_or_else(|| anyhow::anyhow!("missing {marker} block"))?;
    let rest = &response[start + marker.len()..];
    let fence = rest
        .find("```")
        .ok_or_else(|| anyhow::anyhow!("missing opening fence for {marker}"))?;
    let after_fence = &rest[fence + 3..];
    let newline = after_fence.find('\n').unwrap_or(0);
    let body = &after_fence[newline..];
    let end = body
        .find("```")
        .ok_or_else(|| anyhow::anyhow!("missing closing fence for {marker}"))?;
    Ok(body[..end].trim().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Model;

    impl Tier3Model for Model {
        fn generate_tool(&self, _prompt: &str) -> Result<String> {
            Ok("src/lib.rs\n```rust\npub fn run() {}\n```\nCargo.toml\n```toml\n[package]\nname=\"tool\"\n```".into())
        }
    }

    #[test]
    fn extracts_generated_source_and_manifest() {
        let tool = architect_tool(
            &Model,
            &MissingToolEvent {
                capability: "lookup".into(),
                prompt: "need lookup".into(),
                wit_context: "world config-ai".into(),
            },
        )
        .unwrap();

        assert!(tool.lib_rs.contains("pub fn run"));
        assert!(tool.cargo_toml.contains("[package]"));
    }
}
