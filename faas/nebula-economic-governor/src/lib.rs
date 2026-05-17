use anyhow::Result;
use nebula_tenant_core::TenantId;
use serde::{Deserialize, Serialize};

pub const FORWARD_TOPIC: &str = "nebula.teacher.arbitration.request";
pub const BUDGET_EXHAUSTED_TOPIC: &str = "nebula.finops.budget_exhausted";
pub const TOKEN_RESERVED_TOPIC: &str = "nebula.finops.token_reserved";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArbitrationRequest {
    pub tenant_id: TenantId,
    pub prompt: String,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantBudget {
    pub tenant_id: TenantId,
    pub daily_token_limit: u64,
    pub daily_tokens_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsageEvent {
    pub tenant_id: TenantId,
    pub estimated_tokens: u64,
    pub exact_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReservationReceipt {
    pub tenant_id: TenantId,
    pub reserved_tokens: u64,
    pub used_after_reservation: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BudgetDecision {
    Forward {
        topic: &'static str,
        receipt: ReservationReceipt,
    },
    Exhausted {
        topic: &'static str,
        remaining_tokens: u64,
    },
}

pub trait BudgetStore {
    fn reserve_if_under_quota(
        &mut self,
        tenant_id: TenantId,
        estimated_tokens: u64,
    ) -> Result<Option<ReservationReceipt>>;
    fn reconcile_tokens(&mut self, tenant_id: TenantId, estimated: u64, exact: u64) -> Result<()>;
}

pub fn evaluate_budget(
    store: &mut impl BudgetStore,
    request: &ArbitrationRequest,
) -> Result<BudgetDecision> {
    let estimated = estimate_tokens(&request.prompt);
    if let Some(receipt) = store.reserve_if_under_quota(request.tenant_id, estimated)? {
        return Ok(BudgetDecision::Forward {
            topic: FORWARD_TOPIC,
            receipt,
        });
    }

    Ok(BudgetDecision::Exhausted {
        topic: BUDGET_EXHAUSTED_TOPIC,
        remaining_tokens: 0,
    })
}

pub fn reconcile_exact_usage(store: &mut impl BudgetStore, event: &TokenUsageEvent) -> Result<()> {
    if let Some(exact) = event.exact_tokens {
        store.reconcile_tokens(event.tenant_id, event.estimated_tokens, exact)?;
    }
    Ok(())
}

pub fn estimate_tokens(text: &str) -> u64 {
    match tiktoken_rs::cl100k_base() {
        Ok(encoding) => encoding.encode_with_special_tokens(text).len().max(1) as u64,
        Err(_) => text.chars().count().div_ceil(3).max(1) as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use nebula_tenant_core::deterministic_test_tenant;

    struct Store {
        budget: TenantBudget,
        used: u64,
        reconciled: Option<u64>,
    }

    impl BudgetStore for Store {
        fn reserve_if_under_quota(
            &mut self,
            tenant_id: TenantId,
            estimated_tokens: u64,
        ) -> Result<Option<ReservationReceipt>> {
            let remaining = self
                .budget
                .daily_token_limit
                .saturating_sub(self.budget.daily_tokens_used + self.used);
            if estimated_tokens > remaining {
                return Ok(None);
            }
            self.used += estimated_tokens;
            Ok(Some(ReservationReceipt {
                tenant_id,
                reserved_tokens: estimated_tokens,
                used_after_reservation: self.budget.daily_tokens_used + self.used,
            }))
        }

        fn reconcile_tokens(
            &mut self,
            _tenant_id: TenantId,
            estimated: u64,
            exact: u64,
        ) -> Result<()> {
            if exact > estimated {
                self.used += exact - estimated;
            } else {
                self.used -= estimated - exact;
            }
            self.reconciled = Some(exact);
            Ok(())
        }
    }

    #[test]
    // spec: economic-governor
    fn reserves_when_under_budget() {
        let tenant_id = deterministic_test_tenant("acme");
        let mut store = Store {
            budget: TenantBudget {
                tenant_id,
                daily_token_limit: 100,
                daily_tokens_used: 0,
            },
            used: 0,
            reconciled: None,
        };

        let decision = evaluate_budget(
            &mut store,
            &ArbitrationRequest {
                tenant_id,
                prompt: "small request".into(),
                context: serde_json::json!({}),
            },
        )
        .unwrap();

        assert!(matches!(decision, BudgetDecision::Forward { .. }));
        assert!(store.used > 0);
    }

    #[test]
    // spec: economic-governor
    fn drops_when_budget_exhausted() {
        let tenant_id = deterministic_test_tenant("acme");
        let mut store = Store {
            budget: TenantBudget {
                tenant_id,
                daily_token_limit: 1,
                daily_tokens_used: 1,
            },
            used: 0,
            reconciled: None,
        };

        let decision = evaluate_budget(
            &mut store,
            &ArbitrationRequest {
                tenant_id,
                prompt: "expensive request".into(),
                context: serde_json::json!({}),
            },
        )
        .unwrap();

        assert!(matches!(decision, BudgetDecision::Exhausted { .. }));
    }

    #[test]
    // spec: economic-governor
    fn reconciliation_debits_and_credits_estimate_delta() {
        let tenant_id = deterministic_test_tenant("acme");
        let mut store = Store {
            budget: TenantBudget {
                tenant_id,
                daily_token_limit: 100,
                daily_tokens_used: 0,
            },
            used: 10,
            reconciled: None,
        };

        reconcile_exact_usage(
            &mut store,
            &TokenUsageEvent {
                tenant_id,
                estimated_tokens: 10,
                exact_tokens: Some(12),
            },
        )
        .unwrap();
        assert_eq!(store.used, 12);

        reconcile_exact_usage(
            &mut store,
            &TokenUsageEvent {
                tenant_id,
                estimated_tokens: 12,
                exact_tokens: Some(9),
            },
        )
        .unwrap();
        assert_eq!(store.used, 9);
    }
}
