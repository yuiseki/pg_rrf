use pgrx::prelude::*;

::pgrx::pg_module_magic!(name, version);

fn rrf_sum(ranks: &[Option<i64>], k: i64) -> Option<f64> {
    if k <= 0 {
        error!("rrf k must be positive");
    }

    let kf = k as f64;
    let mut sum = 0.0f64;
    let mut used = 0usize;

    for rank in ranks.iter().flatten() {
        if *rank > 0 {
            sum += 1.0 / (kf + (*rank as f64));
            used += 1;
        }
    }

    if used == 0 {
        None
    } else {
        Some(sum)
    }
}

#[pg_extern]
fn rrf(rank_a: Option<i64>, rank_b: Option<i64>, k: i64) -> Option<f64> {
    rrf_sum(&[rank_a, rank_b], k)
}

#[pg_extern]
fn rrf3(
    rank_a: Option<i64>,
    rank_b: Option<i64>,
    rank_c: Option<i64>,
    k: i64,
) -> Option<f64> {
    rrf_sum(&[rank_a, rank_b, rank_c], k)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_rrf_basic() {
        let score = rrf(Some(1), Some(2), 60).unwrap();
        let expected = 1.0 / 61.0 + 1.0 / 62.0;
        assert!((score - expected).abs() < 1e-12);
    }

    #[pg_test]
    fn test_rrf_nulls() {
        let score = rrf(Some(1), None, 60).unwrap();
        let expected = 1.0 / 61.0;
        assert!((score - expected).abs() < 1e-12);

        let score = rrf(None, None, 60);
        assert!(score.is_none());
    }

    #[pg_test]
    fn test_rrf_ignores_non_positive_ranks() {
        let score = rrf(Some(0), Some(2), 60).unwrap();
        let expected = 1.0 / 62.0;
        assert!((score - expected).abs() < 1e-12);

        let score = rrf(Some(-1), None, 60);
        assert!(score.is_none());
    }

    #[pg_test]
    #[should_panic(expected = "rrf k must be positive")]
    fn test_rrf_invalid_k() {
        let _ = rrf(Some(1), Some(2), 0);
    }

    #[pg_test]
    fn test_rrf3_basic() {
        let score = rrf3(Some(1), Some(2), Some(3), 60).unwrap();
        let expected = 1.0 / 61.0 + 1.0 / 62.0 + 1.0 / 63.0;
        assert!((score - expected).abs() < 1e-12);
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
