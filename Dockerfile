FROM pgvector/pgvector:0.8.1-pg17-trixie AS builder

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/cargo \
    RUSTUP_HOME=/rustup
ENV PATH="$CARGO_HOME/bin:$PATH"

RUN set -eux; \
  apt-get update; \
  apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    clang \
    llvm \
    pkg-config \
    libssl-dev \
    sudo \
    postgresql-server-dev-17; \
  rm -rf /var/lib/apt/lists/*

RUN set -eux; \
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable; \
  rustc --version; \
  cargo --version

WORKDIR /workspace
COPY Cargo.toml Cargo.lock* ./
COPY pg_rrf.control ./
COPY src ./src

RUN set -eux; \
  cargo install cargo-pgrx --version 0.16.1; \
  cargo pgrx init --pg17 /usr/lib/postgresql/17/bin/pg_config; \
  cargo pgrx install --release

FROM pgvector/pgvector:0.8.1-pg17-trixie

COPY --from=builder /usr/lib/postgresql/17/lib/pg_rrf.so /usr/lib/postgresql/17/lib/
COPY --from=builder /usr/share/postgresql/17/extension/pg_rrf* /usr/share/postgresql/17/extension/
