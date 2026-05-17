use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    if diagnostics
        .iter()
        .any(|line| line.to_ascii_lowercase().contains("error"))
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Runner;

    impl FoundryRunner for Runner {
        fn build_component(&mut self, _request: &BuildToolRequest) -> Result<Vec<String>> {
            Ok(vec!["finished release wasm32-wasip1".into()])
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
}
