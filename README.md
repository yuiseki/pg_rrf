# pg_rrf

A PostgreSQL extension that provides Reciprocal Rank Fusion (RRF) functions for hybrid search score fusion.

## Status

- Version: v0.0.3
- Scope: SPEC.md v0.0.3

## Features
- `rrf(rank_a, rank_b, k)`
- `rrf3(rank_a, rank_b, rank_c, k)`
- `rrf_fuse(ids_a bigint[], ids_b bigint[], k int default 60)`
- `rrfn(ranks bigint[], k int)`
- NULL-safe: missing ranks are ignored
- `rank <= 0` is ignored
- `k <= 0` raises an error

## Requirements
- PostgreSQL 14–17
- Docker and Docker Compose v2

Local PostgreSQL or Rust toolchains are not required. The Makefile always runs in Docker.

## Quick Start

Build the extension and collect artifacts into `build/`:
```
make build
```

Run tests:
```
make test
```

## Package (Docker)

```
make package
```

Artifacts are collected under `build/pg17` by default. To target a different
PostgreSQL version:

```
PG_MAJOR=16 make package
```

## Using the Extension

Start the database container:
```
docker compose up -d db
```

Connect and enable the extension:
```
psql postgresql://postgres:postgres@localhost:5436/sandbox
CREATE EXTENSION pg_rrf;
```

Example queries:
```
SELECT rrf(1, 2, 60) AS rrf_12;
SELECT rrf3(1, 2, 3, 60) AS rrf_123;
SELECT rrfn(ARRAY[1, 2, 3], 60) AS rrfn_123;
SELECT * FROM rrf_fuse(ARRAY[10, 20, 30], ARRAY[20, 40], 60) ORDER BY score DESC;
```

Stop the database:
```
docker compose down
```

## Outputs

`make build` places these files under `build/`:
- `pg_rrf.so`
- `pg_rrf.control`
- `pg_rrf--<version>.sql`

## Notes
- The Docker database is exposed on `localhost:5436` by default to avoid conflicts.
- The extension name is `pg_rrf`. The repository name is expected to be `pg_rrf`.
