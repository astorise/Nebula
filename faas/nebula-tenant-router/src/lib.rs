use anyhow::Result;
use nebula_tenant_core::{deterministic_test_tenant, resolve_tenant, TenantRegistry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const INPUT_TOPIC: &str = "pulsar.telemetry.inference_triplets";
pub const OUTPUT_TOPIC: &str = "nebula.tenant.routed_triplets";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantRouterConfig {
    pub require_registered_tenant: bool,
}

impl Default for TenantRouterConfig {
    fn default() -> Self {
        Self {
            require_registered_tenant: true,
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
    pub tenant_id: Uuid,
    pub payload: TelemetryTriplet,
}

pub fn route_tenant_triplet(
    triplet: TelemetryTriplet,
    config: &TenantRouterConfig,
) -> Result<Option<TenantRoutedTriplet>> {
    route_tenant_triplet_with_registry(triplet, config, &StaticRegistry)
}

pub fn route_tenant_triplet_with_registry(
    triplet: TelemetryTriplet,
    config: &TenantRouterConfig,
    registry: &impl TenantRegistry,
) -> Result<Option<TenantRoutedTriplet>> {
    let tenant = triplet
        .context
        .get("x-tenant-id")
        .filter(|value| !value.trim().is_empty())
        .cloned();

    if tenant.is_none() || config.require_registered_tenant {
        let Some(raw_tenant) = tenant else {
            return Ok(None);
        };

        let tenant_id = match resolve_tenant(&raw_tenant, registry) {
            Ok(tenant_id) => tenant_id,
            Err(_) => return Ok(None),
        };
        return Ok(Some(TenantRoutedTriplet {
            tenant_id,
            payload: triplet,
        }));
    }

    Ok(None)
}

struct StaticRegistry;

impl TenantRegistry for StaticRegistry {
    fn lookup_tenant_uuid(&self, raw_id: &str) -> Result<Option<Uuid>> {
        Ok((raw_id == "default" || raw_id == "acme").then(|| deterministic_test_tenant(raw_id)))
    }

    fn tenant_row_count(&self, _tenant_id: Uuid) -> Result<usize> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nebula_tenant_core::TenantRegistry;

    struct Registry;

    impl TenantRegistry for Registry {
        fn lookup_tenant_uuid(&self, raw_id: &str) -> Result<Option<Uuid>> {
            Ok((raw_id == "acme").then(|| deterministic_test_tenant(raw_id)))
        }

        fn tenant_row_count(&self, _tenant_id: Uuid) -> Result<usize> {
            Ok(0)
        }
    }

    #[test]
    fn routes_registered_tenant() {
        let mut context = BTreeMap::new();
        context.insert("x-tenant-id".into(), "acme".into());
        let routed = route_tenant_triplet_with_registry(
            TelemetryTriplet {
                prompt: "p".into(),
                answer: "a".into(),
                context,
            },
            &TenantRouterConfig::default(),
            &Registry,
        )
        .unwrap()
        .unwrap();

        assert_eq!(routed.tenant_id, deterministic_test_tenant("acme"));
    }

    #[test]
    fn drops_unregistered_tenant() {
        let mut context = BTreeMap::new();
        context.insert("x-tenant-id".into(), "acme/prod".into());
        let routed = route_tenant_triplet_with_registry(
            TelemetryTriplet {
                prompt: "p".into(),
                answer: "a".into(),
                context,
            },
            &TenantRouterConfig::default(),
            &Registry,
        )
        .unwrap();

        assert!(routed.is_none());
    }
}
