use pgrx::prelude::*;
use std::collections::{HashMap, HashSet};

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

#[pg_extern]
fn rrf_fuse(
    ids_a: Option<Vec<Option<i64>>>,
    ids_b: Option<Vec<Option<i64>>>,
    k: default!(i64, 60),
) -> TableIterator<
    'static,
    (
        name!(id, i64),
        name!(score, f64),
        name!(rank_a, Option<i32>),
        name!(rank_b, Option<i32>),
    ),
> {
    let mut ranks_a = HashMap::<i64, i32>::new();
    if let Some(ids) = ids_a {
        for (idx, id) in ids.into_iter().enumerate() {
            if let Some(id) = id {
                let rank = (idx + 1) as i32;
                ranks_a
                    .entry(id)
                    .and_modify(|r| {
                        if rank < *r {
                            *r = rank;
                        }
                    })
                    .or_insert(rank);
            }
        }
    }

    let mut ranks_b = HashMap::<i64, i32>::new();
    if let Some(ids) = ids_b {
        for (idx, id) in ids.into_iter().enumerate() {
            if let Some(id) = id {
                let rank = (idx + 1) as i32;
                ranks_b
                    .entry(id)
                    .and_modify(|r| {
                        if rank < *r {
                            *r = rank;
                        }
                    })
                    .or_insert(rank);
            }
        }
    }

    let mut ids = HashSet::<i64>::new();
    ids.extend(ranks_a.keys().copied());
    ids.extend(ranks_b.keys().copied());

    let mut rows = Vec::with_capacity(ids.len());
    for id in ids.into_iter() {
        let rank_a = ranks_a.get(&id).copied();
        let rank_b = ranks_b.get(&id).copied();
        let score = rrf_sum(
            &[
                rank_a.map(|r| r as i64),
                rank_b.map(|r| r as i64),
            ],
            k,
        )
        .unwrap_or(0.0);
        rows.push((id, score, rank_a, rank_b));
    }

    TableIterator::new(rows.into_iter())
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    fn rows_to_map(
        rows: Vec<(i64, f64, Option<i32>, Option<i32>)>,
    ) -> HashMap<i64, (f64, Option<i32>, Option<i32>)> {
        rows.into_iter()
            .map(|(id, score, rank_a, rank_b)| (id, (score, rank_a, rank_b)))
            .collect()
    }

    #[pg_test]
    fn test_rrf_fuse_overlap() {
        let rows: Vec<(i64, f64, Option<i32>, Option<i32>)> =
            rrf_fuse(
                Some(vec![Some(10), Some(20), Some(30)]),
                Some(vec![Some(20), Some(40)]),
                60,
            )
            .collect();

        let map = rows_to_map(rows);
        assert_eq!(map.len(), 4);

        let (score, rank_a, rank_b) = map.get(&20).unwrap();
        assert_eq!(*rank_a, Some(2));
        assert_eq!(*rank_b, Some(1));

        let expected = rrf(Some(2), Some(1), 60).unwrap();
        assert!((*score - expected).abs() < 1e-12);
    }

    #[pg_test]
    fn test_rrf_fuse_disjoint_and_null_list() {
        let rows: Vec<(i64, f64, Option<i32>, Option<i32>)> =
            rrf_fuse(None, Some(vec![Some(1), Some(2)]), 60).collect();
        let map = rows_to_map(rows);
        assert_eq!(map.len(), 2);
        let (_, rank_a, rank_b) = map.get(&1).unwrap();
        assert_eq!(*rank_a, None);
        assert_eq!(*rank_b, Some(1));
    }

    #[pg_test]
    fn test_rrf_fuse_duplicates_take_best_rank() {
        let rows: Vec<(i64, f64, Option<i32>, Option<i32>)> =
            rrf_fuse(
                Some(vec![Some(10), Some(20), Some(10)]),
                Some(vec![Some(10)]),
                60,
            )
            .collect();
        let map = rows_to_map(rows);
        let (_, rank_a, rank_b) = map.get(&10).unwrap();
        assert_eq!(*rank_a, Some(1));
        assert_eq!(*rank_b, Some(1));
    }

    #[pg_test]
    #[should_panic(expected = "rrf k must be positive")]
    fn test_rrf_fuse_invalid_k() {
        let _: Vec<(i64, f64, Option<i32>, Option<i32>)> =
            rrf_fuse(Some(vec![Some(1)]), None, 0).collect();
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
