use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const CARGO_COMPONENT_BUILD_ARGS: &[&str] =
    &["component", "build", "--release", "--message-format=json"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildToolRequest {
    pub src_lib_rs: String,
    pub cargo_toml: String,
    pub artifact_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildToolResponse {
    pub artifact_ref: Option<String>,
    pub diagnostics: Vec<String>,
    pub success: bool,
}

pub trait FoundryRunner {
    fn build_component(&mut self, request: &BuildToolRequest) -> Result<Vec<String>>;
    fn push_artifact(&mut self, artifact_ref: &str) -> Result<String>;
}

pub fn build_tool(
    runner: &mut impl FoundryRunner,
    request: BuildToolRequest,
) -> Result<BuildToolResponse> {
    let diagnostics = runner.build_component(&request)?;
    if cargo_diagnostics_have_error(&diagnostics) {
        return Ok(BuildToolResponse {
            artifact_ref: None,
            diagnostics,
            success: false,
        });
    }

    let artifact_ref = runner.push_artifact(&request.artifact_ref)?;
    Ok(BuildToolResponse {
        artifact_ref: Some(artifact_ref),
        diagnostics,
        success: true,
    })
}

pub fn cargo_diagnostics_have_error(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            return false;
        };
        value
            .get("message")
            .and_then(|message| message.get("level"))
            .and_then(serde_json::Value::as_str)
            == Some("error")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Runner;

    impl FoundryRunner for Runner {
        fn build_component(&mut self, _request: &BuildToolRequest) -> Result<Vec<String>> {
            Ok(vec![serde_json::json!({
                "reason": "compiler-message",
                "message": { "level": "warning", "message": "contains word error in warning text" }
            })
            .to_string()])
        }

        fn push_artifact(&mut self, artifact_ref: &str) -> Result<String> {
            Ok(artifact_ref.into())
        }
    }

    #[test]
    fn builds_and_pushes_wasm_artifact() {
        let response = build_tool(
            &mut Runner,
            BuildToolRequest {
                src_lib_rs: "pub fn run() {}".into(),
                cargo_toml: "[package]\nname=\"tool\"".into(),
                artifact_ref: "oci://localhost:5000/tools/lookup:v1".into(),
            },
        )
        .unwrap();

        assert!(response.success);
        assert_eq!(
            response.artifact_ref.unwrap(),
            "oci://localhost:5000/tools/lookup:v1"
        );
    }

    #[test]
    fn detects_only_structured_cargo_errors() {
        assert!(!cargo_diagnostics_have_error(&["plain text error".into()]));
        assert!(cargo_diagnostics_have_error(&[
            serde_json::json!({ "message": { "level": "error" } }).to_string()
        ]));
    }
}
