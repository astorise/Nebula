#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use nebula_economic_governor::{BudgetStore, ReservationReceipt};
#[cfg(not(target_arch = "wasm32"))]
use nebula_tenant_core::TenantId;

#[cfg(not(target_arch = "wasm32"))]
use redis::{Commands, Script};

#[cfg(not(target_arch = "wasm32"))]
const RESERVE_LUA: &str = r#"
local current = tonumber(redis.call("GET", KEYS[1]) or "0")
local limit = tonumber(ARGV[1])
local estimated = tonumber(ARGV[2])
local next = current + estimated

if next <= limit then
  redis.call("SET", KEYS[1], next)
  return next
end

return -1
"#;

#[cfg(not(target_arch = "wasm32"))]
pub struct RedisBudgetStore {
    connection: redis::Connection,
    daily_token_limit: u64,
    key_prefix: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl RedisBudgetStore {
    pub fn connect(redis_url: &str, daily_token_limit: u64) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self::from_connection(
            client.get_connection()?,
            daily_token_limit,
        ))
    }

    pub fn from_connection(connection: redis::Connection, daily_token_limit: u64) -> Self {
        Self {
            connection,
            daily_token_limit,
            key_prefix: "nebula:finops:budget".into(),
        }
    }

    pub fn with_key_prefix(mut self, key_prefix: impl Into<String>) -> Self {
        self.key_prefix = key_prefix.into();
        self
    }

    fn usage_key(&self, tenant_id: TenantId) -> String {
        format!("{}:{tenant_id}:daily_tokens_used", self.key_prefix)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BudgetStore for RedisBudgetStore {
    fn reserve_if_under_quota(
        &mut self,
        tenant_id: TenantId,
        estimated_tokens: u64,
    ) -> Result<Option<ReservationReceipt>> {
        let used_after_reservation: i64 = Script::new(RESERVE_LUA)
            .key(self.usage_key(tenant_id))
            .arg(self.daily_token_limit)
            .arg(estimated_tokens)
            .invoke(&mut self.connection)?;

        if used_after_reservation < 0 {
            return Ok(None);
        }

        Ok(Some(ReservationReceipt {
            tenant_id,
            reserved_tokens: estimated_tokens,
            used_after_reservation: used_after_reservation as u64,
        }))
    }

    fn reconcile_tokens(&mut self, tenant_id: TenantId, estimated: u64, exact: u64) -> Result<()> {
        let key = self.usage_key(tenant_id);
        if exact >= estimated {
            let delta = exact - estimated;
            if delta > 0 {
                let _: i64 = self.connection.incr(key, delta)?;
            }
            return Ok(());
        }

        let delta = estimated - exact;
        let _: i64 = self.connection.decr(key, delta)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RESERVE_LUA;

    #[test]
    // spec: concrete-adapters
    fn reservation_lua_uses_single_atomic_eval_script() {
        assert!(RESERVE_LUA.contains("redis.call(\"GET\""));
        assert!(RESERVE_LUA.contains("redis.call(\"SET\""));
        assert!(RESERVE_LUA.contains("next <= limit"));
        assert!(RESERVE_LUA.contains("return -1"));
    }
}
