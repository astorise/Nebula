use serde::{Deserialize, Serialize};

pub const MISSING_TOOL_TOPIC: &str = "nebula.telemetry.missing_tool";
pub const APPROVAL_REQUIRED_TOPIC: &str = "nebula.foundry.approval_required";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SynthesisMode {
    Disabled,
    HumanInLoop,
    FullyAutonomous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSynthesisConfig {
    pub mode: SynthesisMode,
    pub max_retries: u8,
    pub allowed_wit_imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissingToolSignal {
    pub capability: String,
    pub wit_imports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    EmitMissingTool,
    ApprovalRequired,
    AutoBuild,
    Reject(String),
}

pub fn evaluate_gate(config: &ToolSynthesisConfig, signal: &MissingToolSignal) -> GateDecision {
    if let Some(import) = signal
        .wit_imports
        .iter()
        .find(|import| !config.allowed_wit_imports.contains(import))
    {
        return GateDecision::Reject(format!("WIT import not allowed: {import}"));
    }

    match config.mode {
        SynthesisMode::Disabled => GateDecision::EmitMissingTool,
        SynthesisMode::HumanInLoop => GateDecision::ApprovalRequired,
        SynthesisMode::FullyAutonomous => GateDecision::AutoBuild,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // spec: tool-gap-analyzer
    fn requires_approval_in_human_loop_mode() {
        let decision = evaluate_gate(
            &ToolSynthesisConfig {
                mode: SynthesisMode::HumanInLoop,
                max_retries: 2,
                allowed_wit_imports: vec!["wasi:clocks".into()],
            },
            &MissingToolSignal {
                capability: "time".into(),
                wit_imports: vec!["wasi:clocks".into()],
            },
        );

        assert_eq!(decision, GateDecision::ApprovalRequired);
    }
}
