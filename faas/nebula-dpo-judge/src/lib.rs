use anyhow::Result;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    fn append_audit_hash(&mut self, _hash: &str) -> Result<()> {
        Ok(())
    }
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
    let pattern_set = compile_forbidden_patterns(&rules)?;
    if pattern_set.is_match(&request.tier3_answer) {
        anyhow::bail!("chosen answer violates constitution");
    }

    let pair = PreferencePair {
        prompt: build_teacher_prompt(&request.prompt, &rules),
        chosen: request.tier3_answer,
        rejected: request.tier1_answer,
        tenant_id: request.tenant_id,
    };
    let audit_hash = preference_audit_hash(&pair);
    sink.write_preference(pair.clone())?;
    sink.append_audit_hash(&audit_hash)?;
    Ok(pair)
}

pub fn compile_forbidden_patterns(rules: &[ConstitutionRule]) -> Result<RegexSet> {
    let patterns = rules
        .iter()
        .flat_map(|rule| rule.forbidden_terms.iter())
        .map(|term| {
            let identifier = term
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .collect::<String>();
            let needle = if identifier.is_empty() {
                term.as_str()
            } else {
                identifier.as_str()
            };
            format!(r"(?i)\b{}\b", regex::escape(needle))
        })
        .collect::<Vec<_>>();
    Ok(RegexSet::new(patterns)?)
}

pub fn preference_audit_hash(pair: &PreferencePair) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pair.prompt.as_bytes());
    hasher.update(b"\0");
    hasher.update(pair.chosen.as_bytes());
    hasher.update(b"\0");
    hasher.update(pair.rejected.as_bytes());
    format!("{:x}", hasher.finalize())
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

        fn append_audit_hash(&mut self, hash: &str) -> Result<()> {
            assert_eq!(hash.len(), 64);
            Ok(())
        }
    }

    #[test]
    // spec: dpo-judge
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

    #[test]
    // spec: dpo-judge
    fn rejects_case_variants_without_matching_identifiers() {
        let rules = vec![ConstitutionRule {
            id: "rust-safety".into(),
            instruction: "Do not use unwrap.".into(),
            forbidden_terms: vec!["unwrap".into()],
        }];
        let set = compile_forbidden_patterns(&rules).unwrap();

        assert!(set.is_match("value.Unwrap()"));
        assert!(!set.is_match("unwrap_internal_macro"));
    }
}
