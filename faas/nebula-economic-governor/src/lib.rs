use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const FORWARD_TOPIC: &str = "nebula.teacher.arbitration.request";
pub const BUDGET_EXHAUSTED_TOPIC: &str = "nebula.finops.budget_exhausted";
pub const TOKEN_RESERVED_TOPIC: &str = "nebula.finops.token_reserved";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArbitrationRequest {
    pub tenant_id: String,
    pub prompt: String,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantBudget {
    pub tenant_id: String,
    pub daily_token_limit: u64,
    pub daily_tokens_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsageEvent {
    pub tenant_id: String,
    pub estimated_tokens: u64,
    pub exact_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BudgetDecision {
    Forward {
        topic: &'static str,
        reserved_tokens: u64,
    },
    Exhausted {
        topic: &'static str,
        remaining_tokens: u64,
    },
}

pub trait BudgetStore {
    fn load_budget(&self, tenant_id: &str) -> Result<TenantBudget>;
    fn reserve_tokens(&mut self, tenant_id: &str, tokens: u64) -> Result<()>;
    fn reconcile_tokens(&mut self, tenant_id: &str, estimated: u64, exact: u64) -> Result<()>;
}

pub fn evaluate_budget(
    store: &mut impl BudgetStore,
    request: &ArbitrationRequest,
) -> Result<BudgetDecision> {
    let budget = store.load_budget(&request.tenant_id)?;
    let estimated = estimate_tokens(&request.prompt);
    let remaining = budget
        .daily_token_limit
        .saturating_sub(budget.daily_tokens_used);
    if estimated > remaining {
        return Ok(BudgetDecision::Exhausted {
            topic: BUDGET_EXHAUSTED_TOPIC,
            remaining_tokens: remaining,
        });
    }

    store.reserve_tokens(&request.tenant_id, estimated)?;
    Ok(BudgetDecision::Forward {
        topic: FORWARD_TOPIC,
        reserved_tokens: estimated,
    })
}

pub fn reconcile_exact_usage(store: &mut impl BudgetStore, event: &TokenUsageEvent) -> Result<()> {
    if let Some(exact) = event.exact_tokens {
        store.reconcile_tokens(&event.tenant_id, event.estimated_tokens, exact)?;
    }
    Ok(())
}

pub fn estimate_tokens(text: &str) -> u64 {
    let words = text.split_whitespace().count() as u64;
    let chars = text.chars().count() as u64;
    words.max(chars.div_ceil(4)).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Store {
        budget: TenantBudget,
        reserved: u64,
        reconciled: Option<u64>,
    }

    impl BudgetStore for Store {
        fn load_budget(&self, _tenant_id: &str) -> Result<TenantBudget> {
            Ok(self.budget.clone())
        }

        fn reserve_tokens(&mut self, _tenant_id: &str, tokens: u64) -> Result<()> {
            self.reserved += tokens;
            Ok(())
        }

        fn reconcile_tokens(
            &mut self,
            _tenant_id: &str,
            _estimated: u64,
            exact: u64,
        ) -> Result<()> {
            self.reconciled = Some(exact);
            Ok(())
        }
    }

    #[test]
    fn reserves_when_under_budget() {
        let mut store = Store {
            budget: TenantBudget {
                tenant_id: "acme".into(),
                daily_token_limit: 100,
                daily_tokens_used: 0,
            },
            reserved: 0,
            reconciled: None,
        };

        let decision = evaluate_budget(
            &mut store,
            &ArbitrationRequest {
                tenant_id: "acme".into(),
                prompt: "small request".into(),
                context: serde_json::json!({}),
            },
        )
        .unwrap();

        assert!(matches!(decision, BudgetDecision::Forward { .. }));
        assert!(store.reserved > 0);
    }

    #[test]
    fn drops_when_budget_exhausted() {
        let mut store = Store {
            budget: TenantBudget {
                tenant_id: "acme".into(),
                daily_token_limit: 1,
                daily_tokens_used: 1,
            },
            reserved: 0,
            reconciled: None,
        };

        let decision = evaluate_budget(
            &mut store,
            &ArbitrationRequest {
                tenant_id: "acme".into(),
                prompt: "expensive request".into(),
                context: serde_json::json!({}),
            },
        )
        .unwrap();

        assert!(matches!(decision, BudgetDecision::Exhausted { .. }));
    }
}
