#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const RAW_TOPIC: &str = "nebula.telemetry.raw_inferences";
pub const SANITIZED_TOPIC: &str = "pulsar.telemetry.inference_triplets";
pub const PRIVACY_METRIC: &str = "nebula.privacy.entities_masked";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceTriplet {
    pub prompt: String,
    pub task_type: String,
    pub responses: [String; 3],
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskRule {
    pub name: String,
    pub token: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskAudit {
    pub total_masked: usize,
    pub by_rule: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SanitizedTriplet {
    pub triplet: InferenceTriplet,
    pub audit: MaskAudit,
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, event: &InferenceTriplet) -> Result<()>;
}

pub trait PrivacyMetrics {
    fn push_mask_counts(&mut self, metric: &str, audit: &MaskAudit) -> Result<()>;
}

pub fn default_rules() -> Vec<MaskRule> {
    vec![
        MaskRule {
            name: "email".into(),
            token: "<EMAIL>".into(),
            pattern: r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b".into(),
        },
        MaskRule {
            name: "bearer_token".into(),
            token: "<BEARER_TOKEN>".into(),
            pattern: r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]{16,}\b".into(),
        },
        MaskRule {
            name: "jwt".into(),
            token: "<JWT>".into(),
            pattern: r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b".into(),
        },
        MaskRule {
            name: "ipv4".into(),
            token: "<IPV4>".into(),
            pattern: r"\b(?:\d{1,3}\.){3}\d{1,3}\b".into(),
        },
        MaskRule {
            name: "uuid".into(),
            token: "<UUID>".into(),
            pattern:
                r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b"
                    .into(),
        },
        MaskRule {
            name: "ipv6".into(),
            token: "<IPV6>".into(),
            pattern: r"\b(?:[0-9a-fA-F]{1,4}:){2,7}[0-9a-fA-F]{1,4}\b".into(),
        },
        MaskRule {
            name: "credit_card".into(),
            token: "<CREDIT_CARD>".into(),
            pattern: r"\b(?:\d[ -]*?){13,19}\b".into(),
        },
    ]
}

pub fn handle_raw_triplet(
    bus: &mut impl EventBus,
    metrics: &mut impl PrivacyMetrics,
    topic: &str,
    raw_payload: &[u8],
    rules: &[MaskRule],
) -> Result<SanitizedTriplet> {
    if topic != RAW_TOPIC {
        return Err(anyhow!("unsupported topic: {topic}"));
    }

    let triplet: InferenceTriplet = serde_json::from_slice(raw_payload)?;
    let sanitized = sanitize_triplet(triplet, rules)?;
    bus.publish(SANITIZED_TOPIC, &sanitized.triplet)?;
    metrics.push_mask_counts(PRIVACY_METRIC, &sanitized.audit)?;
    Ok(sanitized)
}

pub fn sanitize_triplet(triplet: InferenceTriplet, rules: &[MaskRule]) -> Result<SanitizedTriplet> {
    let compiled = compile_rules(rules)?;
    let mut audit = MaskAudit {
        total_masked: 0,
        by_rule: BTreeMap::new(),
    };

    let prompt = mask_text(&triplet.prompt, &compiled, &mut audit);
    let responses = triplet
        .responses
        .map(|response| mask_text(&response, &compiled, &mut audit));

    Ok(SanitizedTriplet {
        triplet: InferenceTriplet {
            prompt,
            task_type: triplet.task_type,
            responses,
            context: triplet.context,
        },
        audit,
    })
}

pub fn sandbox_text(text: &str, rules: &[MaskRule]) -> Result<(String, MaskAudit)> {
    let compiled = compile_rules(rules)?;
    let mut audit = MaskAudit {
        total_masked: 0,
        by_rule: BTreeMap::new(),
    };
    let masked = mask_text(text, &compiled, &mut audit);
    Ok((masked, audit))
}

fn compile_rules(rules: &[MaskRule]) -> Result<Vec<(String, String, Regex)>> {
    rules
        .iter()
        .map(|rule| {
            Ok((
                rule.name.clone(),
                rule.token.clone(),
                Regex::new(&rule.pattern)?,
            ))
        })
        .collect()
}

fn mask_text(text: &str, rules: &[(String, String, Regex)], audit: &mut MaskAudit) -> String {
    let mut masked = text.to_string();
    for (name, token, regex) in rules {
        if name == "credit_card" {
            masked = mask_credit_cards(&masked, token, regex, audit);
            continue;
        }

        let count = regex.find_iter(&masked).count();
        if count == 0 {
            continue;
        }

        audit.total_masked += count;
        *audit.by_rule.entry(name.clone()).or_insert(0) += count;
        masked = regex.replace_all(&masked, token.as_str()).into_owned();
    }
    masked
}

fn mask_credit_cards(text: &str, token: &str, regex: &Regex, audit: &mut MaskAudit) -> String {
    let mut masked = String::with_capacity(text.len());
    let mut last_end = 0;
    let mut count = 0;

    for candidate in regex.find_iter(text) {
        let digits: String = candidate
            .as_str()
            .chars()
            .filter(|character| character.is_ascii_digit())
            .collect();

        if luhn_check(&digits) {
            masked.push_str(&text[last_end..candidate.start()]);
            masked.push_str(token);
            last_end = candidate.end();
            count += 1;
        }
    }

    if count == 0 {
        return text.to_string();
    }

    masked.push_str(&text[last_end..]);
    audit.total_masked += count;
    *audit.by_rule.entry("credit_card".into()).or_insert(0) += count;
    masked
}

fn luhn_check(digits: &str) -> bool {
    if !(13..=19).contains(&digits.len()) || !digits.chars().all(|digit| digit.is_ascii_digit()) {
        return false;
    }

    let sum: u32 = digits
        .bytes()
        .rev()
        .enumerate()
        .map(|(index, byte)| {
            let mut value = u32::from(byte - b'0');
            if index % 2 == 1 {
                value *= 2;
                if value > 9 {
                    value -= 9;
                }
            }
            value
        })
        .sum();

    sum.is_multiple_of(10)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Bus(Vec<InferenceTriplet>);
    struct Metrics(Vec<MaskAudit>);

    impl EventBus for Bus {
        fn publish(&mut self, _topic: &str, event: &InferenceTriplet) -> Result<()> {
            self.0.push(event.clone());
            Ok(())
        }
    }

    impl PrivacyMetrics for Metrics {
        fn push_mask_counts(&mut self, _metric: &str, audit: &MaskAudit) -> Result<()> {
            self.0.push(audit.clone());
            Ok(())
        }
    }

    #[test]
    // spec: data-anonymizer
    fn masks_default_sensitive_entities() {
        let triplet = InferenceTriplet {
            prompt: "Email john@example.com using Bearer abcdefghijklmnop".into(),
            task_type: "reasoning".into(),
            responses: [
                "Card 4111 1111 1111 1111".into(),
                "Host 192.168.0.1".into(),
                "Trace 550e8400-e29b-41d4-a716-446655440000".into(),
            ],
            context: serde_json::json!({}),
        };

        let sanitized = sanitize_triplet(triplet, &default_rules()).unwrap();

        assert!(sanitized.triplet.prompt.contains("<EMAIL>"));
        assert!(sanitized.triplet.prompt.contains("<BEARER_TOKEN>"));
        assert_eq!(sanitized.audit.total_masked, 5);
    }

    #[test]
    // spec: data-anonymizer
    fn forwards_sanitized_payload_and_metrics() {
        let raw = serde_json::to_vec(&InferenceTriplet {
            prompt: "john@example.com".into(),
            task_type: "reasoning".into(),
            responses: ["ok".into(), "ok".into(), "ok".into()],
            context: serde_json::json!({}),
        })
        .unwrap();
        let mut bus = Bus(Vec::new());
        let mut metrics = Metrics(Vec::new());

        let sanitized =
            handle_raw_triplet(&mut bus, &mut metrics, RAW_TOPIC, &raw, &default_rules()).unwrap();

        assert_eq!(bus.0[0].prompt, "<EMAIL>");
        assert_eq!(metrics.0[0], sanitized.audit);
    }

    #[test]
    // spec: data-anonymizer
    fn masks_only_luhn_valid_credit_cards() {
        let (masked, audit) = sandbox_text(
            "valid 4111 1111 1111 1111 invalid 4111 1111 1111 1112",
            &default_rules(),
        )
        .unwrap();

        assert!(masked.contains("valid <CREDIT_CARD>"));
        assert!(masked.contains("invalid 4111 1111 1111 1112"));
        assert_eq!(audit.by_rule.get("credit_card"), Some(&1));
    }

    #[test]
    // spec: data-anonymizer
    fn masks_uuid_and_ipv6_before_numeric_card_candidates() {
        let (masked, audit) = sandbox_text(
            "trace 550e8400-e29b-41d4-a716-446655440000 host 2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            &default_rules(),
        )
        .unwrap();

        assert!(masked.contains("<UUID>"));
        assert!(masked.contains("<IPV6>"));
        assert!(!masked.contains("<CREDIT_CARD>"));
        assert_eq!(audit.by_rule.get("credit_card"), None);
    }
}
