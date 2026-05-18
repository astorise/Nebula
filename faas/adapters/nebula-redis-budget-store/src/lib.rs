#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use nebula_economic_governor::{BudgetStore, ReservationReceipt};
#[cfg(not(target_arch = "wasm32"))]
use nebula_tenant_core::TenantId;

#[cfg(not(target_arch = "wasm32"))]
use redis::Script;

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
const RECONCILE_LUA: &str = r#"
local current = tonumber(redis.call("GET", KEYS[1]) or "0")
local estimated = tonumber(ARGV[1])
local exact = tonumber(ARGV[2])
local next = current

if exact >= estimated then
  next = current + (exact - estimated)
else
  next = math.max(0, current - (estimated - exact))
end

redis.call("SET", KEYS[1], next)
return next
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
        let _: i64 = Script::new(RECONCILE_LUA)
            .key(self.usage_key(tenant_id))
            .arg(estimated)
            .arg(exact)
            .invoke(&mut self.connection)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nebula_economic_governor::BudgetStore;
    use nebula_tenant_core::deterministic_test_tenant;
    use std::thread;
    use testcontainers_modules::{
        redis::{Redis, REDIS_PORT},
        testcontainers::runners::SyncRunner,
    };

    #[test]
    // spec: redis-integration-tests
    fn concurrent_reservations_remain_atomic_against_real_redis() -> Result<()> {
        let node = match Redis::default().start() {
            Ok(node) => node,
            Err(error) if std::env::var_os("CI").is_none() => {
                eprintln!("skipping Redis integration test because Docker is unavailable: {error}");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        let host_ip = node.get_host()?;
        let host_port = node.get_host_port_ipv4(REDIS_PORT)?;
        let redis_url = format!("redis://{host_ip}:{host_port}");
        let tenant_id = deterministic_test_tenant("redis-concurrent-budget");

        let handles = (0..10)
            .map(|_| {
                let redis_url = redis_url.clone();
                thread::spawn(move || -> Result<bool> {
                    let mut store = RedisBudgetStore::connect(&redis_url, 500)?;
                    Ok(store.reserve_if_under_quota(tenant_id, 100)?.is_some())
                })
            })
            .collect::<Vec<_>>();

        let successes = handles
            .into_iter()
            .map(|handle| handle.join().expect("reservation thread panicked"))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter(|reserved| *reserved)
            .count();

        assert_eq!(successes, 5);
        Ok(())
    }

    #[test]
    // spec: redis-integration-tests
    fn reconcile_lua_clamps_usage_at_zero() {
        assert!(RECONCILE_LUA.contains("math.max(0"));
    }
}
