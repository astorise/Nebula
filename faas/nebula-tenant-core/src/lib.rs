use anyhow::Result;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

pub const TENANT_DATASET_PREFIX: &str = "/mnt/forge/tenants";
pub const TENANT_ROW_QUOTA: usize = 50_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TenantError {
    Empty,
    Unregistered(String),
    QuotaExceeded { tenant_id: Uuid, rows: usize },
}

impl std::fmt::Display for TenantError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(formatter, "tenant id is empty"),
            Self::Unregistered(raw) => write!(formatter, "tenant id is not registered: {raw}"),
            Self::QuotaExceeded { tenant_id, rows } => {
                write!(
                    formatter,
                    "tenant {tenant_id} exceeded row quota with {rows} rows"
                )
            }
        }
    }
}

impl std::error::Error for TenantError {}

pub trait TenantRegistry {
    fn lookup_tenant_uuid(&self, raw_id: &str) -> Result<Option<Uuid>>;
    fn tenant_row_count(&self, tenant_id: Uuid) -> Result<usize>;
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(
    export,
    export_to = "../../../packages/extension/src/types/generated.ts"
)]
pub struct TenantSummary {
    #[ts(type = "string")]
    pub tenant_id: Uuid,
    pub rows: usize,
    pub quota: usize,
}

pub fn resolve_tenant(raw_id: &str, registry: &impl TenantRegistry) -> Result<Uuid, TenantError> {
    let raw_id = raw_id.trim();
    if raw_id.is_empty() {
        return Err(TenantError::Empty);
    }

    registry
        .lookup_tenant_uuid(raw_id)
        .map_err(|_| TenantError::Unregistered(raw_id.into()))?
        .ok_or_else(|| TenantError::Unregistered(raw_id.into()))
}

pub fn enforce_quota(tenant_id: Uuid, registry: &impl TenantRegistry) -> Result<(), TenantError> {
    let rows = registry
        .tenant_row_count(tenant_id)
        .map_err(|_| TenantError::QuotaExceeded {
            tenant_id,
            rows: TENANT_ROW_QUOTA,
        })?;
    if rows >= TENANT_ROW_QUOTA {
        return Err(TenantError::QuotaExceeded { tenant_id, rows });
    }
    Ok(())
}

pub fn tenant_dataset_path(tenant_id: Uuid, file_name: &str) -> String {
    format!("{TENANT_DATASET_PREFIX}/{tenant_id}/{file_name}")
}

pub fn deterministic_test_tenant(raw_id: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("nebula://tenant/{raw_id}").as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Registry;

    impl TenantRegistry for Registry {
        fn lookup_tenant_uuid(&self, raw_id: &str) -> Result<Option<Uuid>> {
            Ok((raw_id == "acme").then(|| deterministic_test_tenant(raw_id)))
        }

        fn tenant_row_count(&self, _tenant_id: Uuid) -> Result<usize> {
            Ok(42)
        }
    }

    #[test]
    fn resolves_only_registered_tenants() {
        assert!(resolve_tenant("acme", &Registry).is_ok());
        assert!(matches!(
            resolve_tenant("acme/prod", &Registry),
            Err(TenantError::Unregistered(_))
        ));
    }
}
