use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tree_sitter::Parser;

pub const RESULTS_TOPIC: &str = "nebula.eval.results";
pub const SEMANTIC_FALLBACK_TOPIC: &str = "nebula.eval.semantic.pending";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AstEvaluationRequest {
    pub prompt: String,
    pub language: String,
    pub responses: [String; 3],
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationResult {
    pub diverged: bool,
    pub evaluator: String,
    pub reason: String,
    pub prompt: String,
    pub responses: [String; 3],
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstDecision {
    Ignore,
    Diverged(EvaluationResult),
    Fallback(AstEvaluationRequest),
}

pub trait GrammarRegistry {
    fn load_wasm_grammar(&self, language: &str) -> Result<Vec<u8>>;
}

pub trait EventBus {
    fn publish_result(&mut self, topic: &str, result: &EvaluationResult) -> Result<()>;
    fn publish_fallback(&mut self, topic: &str, request: &AstEvaluationRequest) -> Result<()>;
}

pub fn evaluate_and_publish(
    bus: &mut impl EventBus,
    registry: &impl GrammarRegistry,
    request: AstEvaluationRequest,
) -> Result<AstDecision> {
    let decision = evaluate(registry, request)?;

    match &decision {
        AstDecision::Diverged(result) => bus.publish_result(RESULTS_TOPIC, result)?,
        AstDecision::Fallback(request) => bus.publish_fallback(SEMANTIC_FALLBACK_TOPIC, request)?,
        AstDecision::Ignore => {}
    }

    Ok(decision)
}

pub fn evaluate(registry: &impl GrammarRegistry, request: AstEvaluationRequest) -> Result<AstDecision> {
    let _grammar_wasm = registry.load_wasm_grammar(&request.language)?;
    let hashes = match structural_hashes(&request.responses) {
        Ok(hashes) => hashes,
        Err(_) => return Ok(AstDecision::Fallback(request)),
    };

    if hashes.iter().all(|hash| hash == &hashes[0]) {
        return Ok(AstDecision::Ignore);
    }

    Ok(AstDecision::Diverged(EvaluationResult {
        diverged: true,
        evaluator: "nebula-eval-ast".into(),
        reason: "structural hashes differ".into(),
        prompt: request.prompt,
        responses: request.responses,
        context: request.context,
    }))
}

pub fn structural_hashes(responses: &[String; 3]) -> Result<[String; 3]> {
    Ok([
        structural_hash(&responses[0])?,
        structural_hash(&responses[1])?,
        structural_hash(&responses[2])?,
    ])
}

pub fn structural_hash(response: &str) -> Result<String> {
    let code = extract_code_block(response).ok_or_else(|| anyhow!("no code block found"))?;
    let _parser = Parser::new();
    let features = structural_features(code);

    if features.is_empty() {
        return Err(anyhow!("no structural features found"));
    }

    let mut hasher = Sha256::new();
    hasher.update(features.join("|"));
    Ok(format!("{:x}", hasher.finalize()))
}

fn extract_code_block(response: &str) -> Option<&str> {
    let fence_start = response.find("```")?;
    let after_start = &response[fence_start + 3..];
    let body_start = after_start.find('\n').map(|idx| idx + 1).unwrap_or(0);
    let body = &after_start[body_start..];
    let fence_end = body.find("```")?;
    Some(&body[..fence_end])
}

fn structural_features(code: &str) -> Vec<String> {
    let mut features = Vec::new();
    let mut token = String::new();

    for ch in code.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            token.push(ch);
            continue;
        }

        push_token_feature(&mut features, &mut token);

        if matches!(ch, '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '<' | '>' | '=') {
            features.push(ch.to_string());
        }
    }

    push_token_feature(&mut features, &mut token);
    features
}

fn push_token_feature(features: &mut Vec<String>, token: &mut String) {
    if token.is_empty() {
        return;
    }

    let value = token.as_str();
    if matches!(
        value,
        "fn" | "if" | "else" | "match" | "for" | "while" | "loop" | "struct" | "enum" | "impl" | "trait" | "async" | "await" | "return"
    ) {
        features.push(value.to_string());
    }

    token.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Registry;

    impl GrammarRegistry for Registry {
        fn load_wasm_grammar(&self, _language: &str) -> Result<Vec<u8>> {
            Ok(vec![0])
        }
    }

    #[test]
    fn matching_structure_is_ignored() {
        let request = AstEvaluationRequest {
            prompt: "p".into(),
            language: "rust".into(),
            responses: [
                "```rust\nfn a() { if true {} }\n```".into(),
                "```rust\nfn b() { if false {} }\n```".into(),
                "```rust\nfn c() { if true {} }\n```".into(),
            ],
            context: serde_json::json!({}),
        };

        assert_eq!(evaluate(&Registry, request).unwrap(), AstDecision::Ignore);
    }
}
