use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstitutionRule {
    pub id: String,
    pub instruction: String,
    #[serde(default)]
    pub forbidden_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DpoJudgementRequest {
    pub prompt: String,
    pub tier1_answer: String,
    pub tier3_answer: String,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferencePair {
    pub prompt: String,
    pub chosen: String,
    pub rejected: String,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

pub trait ConstitutionStore {
    fn load_rules(&self, tenant_id: Option<&str>) -> Result<Vec<ConstitutionRule>>;
}

pub trait PreferenceSink {
    fn write_preference(&mut self, pair: PreferencePair) -> Result<()>;
}

pub fn build_teacher_prompt(prompt: &str, rules: &[ConstitutionRule]) -> String {
    let constitution = rules
        .iter()
        .map(|rule| format!("- {}: {}", rule.id, rule.instruction))
        .collect::<Vec<_>>()
        .join("\n");
    format!("Constitution:\n{constitution}\n\nUser prompt:\n{prompt}\n\nReturn the safest corrected answer.")
}

pub fn judge_and_forward(
    store: &impl ConstitutionStore,
    sink: &mut impl PreferenceSink,
    request: DpoJudgementRequest,
) -> Result<PreferencePair> {
    let rules = store.load_rules(request.tenant_id.as_deref())?;
    for rule in &rules {
        for forbidden in &rule.forbidden_terms {
            anyhow::ensure!(
                !request.tier3_answer.contains(forbidden),
                "chosen answer violates constitution rule {}",
                rule.id
            );
        }
    }

    let pair = PreferencePair {
        prompt: build_teacher_prompt(&request.prompt, &rules),
        chosen: request.tier3_answer,
        rejected: request.tier1_answer,
        tenant_id: request.tenant_id,
    };
    sink.write_preference(pair.clone())?;
    Ok(pair)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Store;
    struct Sink(Vec<PreferencePair>);

    impl ConstitutionStore for Store {
        fn load_rules(&self, _tenant_id: Option<&str>) -> Result<Vec<ConstitutionRule>> {
            Ok(vec![ConstitutionRule {
                id: "rust-safety".into(),
                instruction: "Do not use unwrap in generated Rust.".into(),
                forbidden_terms: vec!["unwrap()".into()],
            }])
        }
    }

    impl PreferenceSink for Sink {
        fn write_preference(&mut self, pair: PreferencePair) -> Result<()> {
            self.0.push(pair);
            Ok(())
        }
    }

    #[test]
    fn forwards_constitutional_preference_pair() {
        let mut sink = Sink(Vec::new());
        let pair = judge_and_forward(
            &Store,
            &mut sink,
            DpoJudgementRequest {
                prompt: "read a file".into(),
                tier1_answer: "unwrap()".into(),
                tier3_answer: "std::fs::read_to_string(path)?".into(),
                tenant_id: Some("acme".into()),
            },
        )
        .unwrap();

        assert!(pair.prompt.contains("Do not use unwrap"));
        assert_eq!(sink.0.len(), 1);
    }
}
