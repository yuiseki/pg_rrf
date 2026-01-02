# pg_rrf

A PostgreSQL extension that provides Reciprocal Rank Fusion (RRF) functions for hybrid search score fusion.

## Features
- `rrf(rank_a, rank_b, k)`
- `rrf3(rank_a, rank_b, rank_c, k)`
- NULL-safe: missing ranks are ignored
- `rank <= 0` is ignored
- `k <= 0` raises an error

## Requirements
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
