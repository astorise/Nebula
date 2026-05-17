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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratedToolPayload {
    pub files: Vec<GeneratedToolFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratedToolFile {
    pub name: String,
    pub content: String,
}

pub trait Tier3Model {
    fn generate_tool(&self, prompt: &str) -> Result<String>;
}

pub fn build_tool_prompt(event: &MissingToolEvent) -> String {
    format!(
        "Generate a Rust Wasm component for capability '{}'. Return strict JSON only using schema {{\"files\":[{{\"name\":\"src/lib.rs\",\"content\":\"...\"}},{{\"name\":\"Cargo.toml\",\"content\":\"...\"}}]}}.\nWIT context:\n{}\nUser need:\n{}",
        event.capability, event.wit_context, event.prompt
    )
}

pub fn architect_tool(model: &impl Tier3Model, event: &MissingToolEvent) -> Result<GeneratedTool> {
    let response = model.generate_tool(&build_tool_prompt(event))?;
    extract_generated_tool(&event.capability, &response)
}

pub fn extract_generated_tool(name: &str, response: &str) -> Result<GeneratedTool> {
    let payload = serde_json::from_str::<GeneratedToolPayload>(response)?;
    let lib_rs = file_content(&payload, "src/lib.rs")?;
    let cargo_toml = file_content(&payload, "Cargo.toml")?;
    Ok(GeneratedTool {
        name: name.into(),
        lib_rs,
        cargo_toml,
    })
}

fn file_content(payload: &GeneratedToolPayload, name: &str) -> Result<String> {
    payload
        .files
        .iter()
        .find(|file| file.name == name)
        .map(|file| file.content.clone())
        .ok_or_else(|| anyhow::anyhow!("missing generated file {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Model;

    impl Tier3Model for Model {
        fn generate_tool(&self, _prompt: &str) -> Result<String> {
            Ok(serde_json::json!({
                "files": [
                    { "name": "src/lib.rs", "content": "pub fn run() {}" },
                    { "name": "Cargo.toml", "content": "[package]\nname=\"tool\"" }
                ]
            })
            .to_string())
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
