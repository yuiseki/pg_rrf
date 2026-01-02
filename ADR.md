# ADR: RRF/Score Fusion as a PostgreSQL Extension via pgrx

Date: 2026-01-01
Status: Accepted

## Context
We want to turn hybrid-search ranking logic into a reusable tool rather than ad-hoc SQL. The TODO proposes providing RRF (Reciprocal Rank Fusion) functions such as `rrf(rank_a, rank_b, k)` and `rrf3(...)`, and potentially a 3-way fusion that includes distance decay (PostGIS), full-text rank, and vector rank. The goal is clean SQL, repeatable experiments, and easy adoption across studies.

We need a safe, maintainable way to provide these functions inside PostgreSQL. Options include SQL functions alone, PL/pgSQL, C extensions, or Rust via pgrx.

## Decision
Implement RRF and related score fusion functions as a PostgreSQL extension using Rust with pgrx.

## Rationale
- pgrx provides a stable Rust-based extension framework, reducing C-level complexity and memory safety risks.
- Functions are small, deterministic, and performance sensitive; a compiled extension is appropriate.
- Having a dedicated extension allows consistent SQL API across studies and avoids copy-paste of formulas.
- The extension boundary makes future enhancements (e.g., 3-way fusion with distance decay) straightforward without rewriting SQL across projects.

## Scope
Initial scope (MVP):
- `rrf(rank_a, rank_b, k)`
- `rrf3(rank_a, rank_b, rank_c, k)`
- Null-safe handling for missing ranks
- Basic tests for numerical correctness and boundary conditions

Out of scope (for now):
- Direct integration with PostGIS distance decay inside the extension
- In-SQL orchestration of candidate generation (handled by SQL queries)
- Advanced configuration or planner hooks

## Alternatives Considered
1) Pure SQL/PL/pgSQL functions
- Pros: simple setup, no compilation
- Cons: performance and reusability constraints; less robust for edge cases

2) C-based PostgreSQL extension
- Pros: high performance, standard approach
- Cons: higher development cost and safety risks compared to Rust

3) Application-side fusion
- Pros: no database extension required
- Cons: loses SQL composability and forces data extraction to the app layer

## Consequences
- Requires Rust toolchain and pgrx for building and installation.
- Extension functions can be shared across projects and serve as a stable API.
- Testing and CI will need to compile and load the extension in a PostgreSQL instance.

## Notes
- Future work can add distance-decay fusion and weighting variants once basic RRF is stable.
- Consider a stable SQL API early to avoid breaking downstream experiments.
