use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INPUT_TOPIC: &str = "pulsar.telemetry.inference_triplets";
pub const OUTPUT_TOPIC: &str = "nebula.tenant.routed_triplets";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantRouterConfig {
    pub strict: bool,
    pub default_tenant: String,
}

impl Default for TenantRouterConfig {
    fn default() -> Self {
        Self {
            strict: false,
            default_tenant: "default".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryTriplet {
    pub prompt: String,
    pub answer: String,
    #[serde(default)]
    pub context: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantRoutedTriplet {
    pub tenant_id: String,
    pub payload: TelemetryTriplet,
}

pub fn route_tenant_triplet(
    triplet: TelemetryTriplet,
    config: &TenantRouterConfig,
) -> Result<Option<TenantRoutedTriplet>> {
    let tenant = triplet
        .context
        .get("x-tenant-id")
        .filter(|value| !value.trim().is_empty())
        .cloned();

    if tenant.is_none() && config.strict {
        return Ok(None);
    }

    Ok(Some(TenantRoutedTriplet {
        tenant_id: sanitize_tenant_id(tenant.as_deref().unwrap_or(&config.default_tenant)),
        payload: triplet,
    }))
}

fn sanitize_tenant_id(tenant_id: &str) -> String {
    tenant_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_explicit_tenant() {
        let mut context = BTreeMap::new();
        context.insert("x-tenant-id".into(), "acme/prod".into());
        let routed = route_tenant_triplet(
            TelemetryTriplet {
                prompt: "p".into(),
                answer: "a".into(),
                context,
            },
            &TenantRouterConfig::default(),
        )
        .unwrap()
        .unwrap();

        assert_eq!(routed.tenant_id, "acme_prod");
    }

    #[test]
    fn drops_missing_tenant_in_strict_mode() {
        let routed = route_tenant_triplet(
            TelemetryTriplet {
                prompt: "p".into(),
                answer: "a".into(),
                context: BTreeMap::new(),
            },
            &TenantRouterConfig {
                strict: true,
                default_tenant: "default".into(),
            },
        )
        .unwrap();

        assert!(routed.is_none());
    }
}
