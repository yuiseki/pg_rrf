# SPEC.md — pg_rrf (Reciprocal Rank Fusion) Extension

Status: Draft  
Target: v0.0.2 (SRF), v0.0.3 (rrfN)  
Last updated: 2026-01-03

## 1. Overview

`pg_rrf` is a PostgreSQL extension providing Reciprocal Rank Fusion (RRF) utilities for hybrid search.
Current v0.0.1 exposes scalar score functions (`rrf`, `rrf3`). This spec expands the extension in two steps:

- **v0.0.2**: Add an SRF (set-returning function) to *fuse* ranked ID lists and return `(id, score, ...)` rows.
- **v0.0.3**: Add a generalized scalar function (**rrfN**) to avoid proliferating `rrf4`, `rrf5`, ... as channels increase.

The guiding principle is: **make fusion easy without forcing users into FULL OUTER JOIN / COALESCE / row_number() boilerplate.**

---

## 2. Definitions

- **Channel**: One retrieval method producing a ranked list (e.g., lexical, vector, geo, recency).
- **Rank**: 1-based position in a channel list. Smaller is better. Missing rank means “not present in that channel.”
- **k**: Positive constant controlling smoothing in `1 / (k + rank)`.

### 2.1 Canonical scoring formula

For a document/item `d` with ranks in channels `r_i(d)`:

```

score(d) = Σ_i 1 / (k + r_i(d))

````

Rules used in this extension:
- Missing rank (`NULL`) is ignored.
- Non-positive rank (`<= 0`) is ignored.
- `k <= 0` raises an ERROR.

---

## 3. Current behavior (v0.0.1 baseline)

v0.0.1 provides:

- `rrf(rank_a, rank_b, k) -> double precision`
- `rrf3(rank_a, rank_b, rank_c, k) -> double precision`

Semantics:
- NULL-safe (missing ranks ignored)
- `rank <= 0` ignored
- `k <= 0` ERROR

This baseline must remain backwards compatible.

---

## 4. Design principles

1. **Backward compatibility**: existing SQL calling `rrf`/`rrf3` continues to work.
2. **Deterministic math**: for identical ranks and k, results are stable.
3. **Short SQL**: provide primitives that reduce user-side SQL complexity.
4. **Separation of concerns**:
   - scalar functions compute a score from ranks
   - SRF functions fuse ranked lists into a result set; joining to the base table remains the caller’s job.
5. **Performance**: O(total input size) time for fusion, with predictable memory use.

---

## 5. Roadmap & versions

### v0.0.2 — SRF for “list fusion” (2-channel MVP)

#### 5.1 Goals
- Provide a minimal SRF that fuses two ranked ID lists.
- Eliminate the caller’s need to write:
  - FULL OUTER JOIN between two ranked subqueries
  - COALESCE rank handling
  - custom score expressions

#### 5.2 Non-goals (explicitly out of scope for v0.0.2)
- N-channel SRF (3+ channels)
- Weighted RRF
- Passing result subqueries/relations into the extension (security + complexity)
- Polymorphic ID types (uuid/text) unless easy to add without complexity

#### 5.3 Public API

##### Function: `rrf_fuse`
Fuses two ranked lists and returns a row set.

**Signature (MVP):**
```sql
rrf_fuse(
  ids_a bigint[],
  ids_b bigint[],
  k int DEFAULT 60
) RETURNS TABLE (
  id bigint,
  score double precision,
  rank_a int,
  rank_b int
);
````

**Inputs:**

* `ids_a`: ranked IDs from channel A (best-first). `ids_a[1]` = rank 1.
* `ids_b`: ranked IDs from channel B (best-first).
* `k`: smoothing constant. Must be > 0.

**Output columns:**

* `id`: fused ID (union of both lists)
* `score`: `1/(k+rank_a) + 1/(k+rank_b)` with ignored/missing rank rules
* `rank_a`, `rank_b`: computed ranks (1-based). If missing, output `NULL`.

**Behavior rules:**

* If `ids_a` is `NULL`, treat as empty.
* If `ids_b` is `NULL`, treat as empty.
* Duplicate IDs inside a list:

  * Use the **best (minimum) rank** for that channel.
* Rank validity:

  * Rank is derived from position. Therefore it is always `>= 1` if present.
  * (No special-casing needed beyond missing ranks.)
* Ordering:

  * SRF does **not guarantee output order**. Callers should `ORDER BY score DESC`.

**Examples:**

```sql
WITH fused AS (
  SELECT * FROM rrf_fuse(
    ARRAY(SELECT id FROM docs ORDER BY bm25_score DESC LIMIT 100),
    ARRAY(SELECT id FROM docs ORDER BY embedding <=> :qvec LIMIT 100),
    60
  )
)
SELECT d.*, fused.score
FROM fused
JOIN docs d USING (id)
ORDER BY fused.score DESC
LIMIT 20;
```

##### Naming note

PostgreSQL folds unquoted identifiers to lower-case. This function name is already lower-case.

#### 5.4 Implementation notes (non-normative)

* Build maps:

  * `HashMap<id, rank_a>`
  * `HashMap<id, rank_b>`
* Union IDs:

  * iterate over both maps’ keys (or build one combined map).
* Compute score using the same internal scoring helper used by `rrf`/`rrf3` (see §7.2).

Return via a set-returning mechanism (`RETURNS TABLE`) suitable for Rust/pgrx.

#### 5.5 Tests (v0.0.2)

Add regression tests covering:

* Both lists non-empty, with overlap
* Disjoint lists
* One list empty/NULL
* Duplicate IDs in a list
* `k <= 0` errors
* Consistency check:

  * For a known pair of ranks, `score` matches scalar `rrf(rank_a, rank_b, k)`.

#### 5.6 Release checklist (v0.0.2)

* Bump extension version metadata to `0.0.2`
* Add `pg_rrf--0.0.2.sql` and upgrade script `pg_rrf--0.0.1--0.0.2.sql` if needed
* Update README:

  * Add SRF description + example
* Ensure `make build` and `make test` pass

---

### v0.0.3 — rrfN (generalized scalar score function)

#### 6.1 Goals

* Generalize `rrf`/`rrf3` into a single function that scales with channel count.
* Avoid `rrf4`, `rrf5`, ... as hybrid search grows.

#### 6.2 Public API

##### Function: `rrfn` (recommended SQL name)

Because PostgreSQL lower-cases identifiers, a mixed-case `rrfN` would become `rrfn` unless quoted.
Therefore we standardize on **`rrfn`** as the public SQL name.

**Primary signature:**

```sql
rrfn(
  ranks int[],
  k int
) RETURNS double precision;
```

**Semantics:**

* `k <= 0` ERROR
* For each `r` in `ranks`:

  * if `r IS NULL` → ignore
  * else if `r <= 0` → ignore
  * else add `1/(k + r)`
* Empty array or all ignored ranks → returns `0.0`

**Convenience wrapper (optional):**

```sql
rrfn(
  k int,
  VARIADIC ranks int[]
) RETURNS double precision;
```

This allows:

```sql
SELECT rrfn(60, rank_text, rank_vec, rank_geo);
```

If this wrapper adds complexity in Rust bindings, it can be deferred to v0.0.3.x.

##### Backwards compatibility

* Keep `rrf(rank_a, rank_b, k)` and `rrf3(rank_a, rank_b, rank_c, k)` as:

  * wrappers around `rrfn(ARRAY[rank_a, rank_b], k)`
  * wrappers around `rrfn(ARRAY[rank_a, rank_b, rank_c], k)`
* Their edge-case behavior must remain identical to v0.0.1.

#### 6.3 Tests (v0.0.3)

Add regression tests:

* `rrfn(ARRAY[1,2], 60)` equals `rrf(1,2,60)`
* `rrfn(ARRAY[1,2,3], 60)` equals `rrf3(1,2,3,60)`
* NULLs inside array: `rrfn(ARRAY[1,NULL,3], 60)` ignores NULL
* `<=0` inside array: ignored
* `k<=0` ERROR
* Empty array: returns `0.0`

#### 6.4 Release checklist (v0.0.3)

* Bump version to `0.0.3`
* Add `pg_rrf--0.0.3.sql` and upgrade script `pg_rrf--0.0.2--0.0.3.sql`
* README updates:

  * Introduce `rrfn` with examples
  * Mention `rrf` and `rrf3` remain supported

---

## 7. Recommended internal refactor (to support SRF → rrfN cleanly)

This section describes internal steps that make the roadmap low-risk.

### 7.1 Introduce an internal “scoring core” helper (v0.0.2)

Add a Rust helper function (not exposed to SQL yet) used by:

* `rrf`
* `rrf3`
* `rrf_fuse`

Example intent (pseudo):

* `fn rrf_score(ranks: impl Iterator<Item = Option<i32>>, k: i32) -> f64`

This reduces duplicate logic and ensures exact behavioral consistency.

### 7.2 Expose the helper as `rrfn` in v0.0.3

In v0.0.3, wire `rrfn` to the same helper.
Then rewrite `rrf` and `rrf3` to delegate to `rrfn`.

---

## 8. Optional task splitting / improvements

If you want even safer incremental delivery without changing the requested versions:

* **v0.0.2 (SRF) split into two PRs**

  1. “Core refactor + tests” (no new SQL API)
  2. “Add `rrf_fuse` SRF + docs”
     This keeps review small and makes regression failures easier to localize.

* **Add a small follow-up patch after v0.0.2**

  * v0.0.2.1: add `uuid[]` variant or polymorphic ID support if users request it
  * v0.0.2.2: add `rrf_fuse3` ONLY if demand appears (but prefer waiting for an eventual N-channel SRF design)

* **Keep SRF N-channel as a future v0.0.4**
  Once `rrfn` is in place, an N-channel SRF can be designed with less risk (e.g., `VARIADIC ids bigint[]` or `bigint[][]`), but it is intentionally out-of-scope here.

---

## 9. Compatibility & behavior guarantees

* `rrf` and `rrf3` behavior remains unchanged across v0.0.1 → v0.0.3.
* New APIs (`rrf_fuse`, `rrfn`) follow the same rank/k rules as v0.0.1.
* Caller must not rely on SRF output order unless explicitly ordered in SQL.

---

## 10. Changelog template (for future)

### v0.0.2

* Added: `rrf_fuse(ids_a bigint[], ids_b bigint[], k int default 60) returns table (...)`
* Internal: shared scoring helper introduced

### v0.0.3

* Added: `rrfn(ranks int[], k int) -> double precision` (generalized RRF)
* Kept: `rrf`, `rrf3` as compatibility wrappers
