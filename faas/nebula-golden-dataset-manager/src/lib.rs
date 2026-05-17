use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const GOLDEN_VECTOR_NAMESPACE: &str = "golden_dataset";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateRow {
    pub prompt: String,
    pub answer: String,
    pub tenant_id: String,
    pub days_in_production: u32,
    pub rollback_count: u32,
    pub drift_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoldenRow {
    pub prompt: String,
    pub answer: String,
    pub tenant_id: String,
    pub locked: bool,
    pub vector_namespace: String,
}

pub trait GoldenStore {
    fn append(&mut self, row: &GoldenRow) -> Result<()>;
    fn vectorize(&mut self, namespace: &str, row: &GoldenRow) -> Result<()>;
}

pub fn promote_if_stable(
    store: &mut impl GoldenStore,
    candidate: CandidateRow,
) -> Result<Option<GoldenRow>> {
    if candidate.days_in_production <= 7
        || candidate.rollback_count > 0
        || candidate.drift_count > 0
    {
        return Ok(None);
    }

    let row = GoldenRow {
        prompt: candidate.prompt,
        answer: candidate.answer,
        tenant_id: candidate.tenant_id,
        locked: false,
        vector_namespace: GOLDEN_VECTOR_NAMESPACE.into(),
    };
    store.append(&row)?;
    store.vectorize(GOLDEN_VECTOR_NAMESPACE, &row)?;
    Ok(Some(row))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Store {
        rows: usize,
        vectors: usize,
    }

    impl GoldenStore for Store {
        fn append(&mut self, _row: &GoldenRow) -> Result<()> {
            self.rows += 1;
            Ok(())
        }

        fn vectorize(&mut self, _namespace: &str, _row: &GoldenRow) -> Result<()> {
            self.vectors += 1;
            Ok(())
        }
    }

    #[test]
    fn promotes_rows_after_stable_production_window() {
        let mut store = Store::default();
        let promoted = promote_if_stable(
            &mut store,
            CandidateRow {
                prompt: "p".into(),
                answer: "a".into(),
                tenant_id: "acme".into(),
                days_in_production: 8,
                rollback_count: 0,
                drift_count: 0,
            },
        )
        .unwrap();

        assert!(promoted.is_some());
        assert_eq!(store.rows, 1);
        assert_eq!(store.vectors, 1);
    }
}
