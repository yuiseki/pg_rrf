ARG PG_MAJOR=17
ARG PGVECTOR_VERSION=0.8.7
ARG DEBIAN_SUITE=trixie

FROM pgvector/pgvector:${PGVECTOR_VERSION}-pg${PG_MAJOR}-${DEBIAN_SUITE} AS builder

ARG PG_MAJOR
ENV PG_CONFIG=/usr/lib/postgresql/${PG_MAJOR}/bin/pg_config \
    DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/cargo \
    RUSTUP_HOME=/rustup
ENV PATH="$CARGO_HOME/bin:$PATH"

RUN set -eux; \
  apt-get update; \
  apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    clang libclang-dev \
    llvm \
    pkg-config \
    libssl-dev \
    sudo \
    postgresql-server-dev-${PG_MAJOR}; \
  rm -rf /var/lib/apt/lists/*

RUN set -eux; \
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable; \
  rustc --version; \
  cargo --version

WORKDIR /workspace
COPY Cargo.toml Cargo.lock* ./
COPY pg_rrf.control ./

RUN set -eux; \
  cargo install --locked cargo-pgrx --version 0.20.1; \
  cargo pgrx init "--pg${PG_MAJOR}" "$PG_CONFIG" --no-run

COPY src ./src

RUN set -eux; \
  cargo pgrx install --release --pg-config "$PG_CONFIG"

FROM pgvector/pgvector:${PGVECTOR_VERSION}-pg${PG_MAJOR}-${DEBIAN_SUITE}
ARG PG_MAJOR

COPY --from=builder /usr/lib/postgresql/${PG_MAJOR}/lib/pg_rrf.so /usr/lib/postgresql/${PG_MAJOR}/lib/
COPY --from=builder /usr/share/postgresql/${PG_MAJOR}/extension/pg_rrf* /usr/share/postgresql/${PG_MAJOR}/extension/
