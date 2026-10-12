# syntax=docker/dockerfile:1

# ─── Builder ──────────────────────────────────────────────────────────────────────
# No sqlx query!/query_as! macros are used anywhere in this codebase (all queries are
# runtime-checked via query()/query_as::<_, T>()), so the build needs no DATABASE_URL
# and no .sqlx cache — a plain `cargo build` is sufficient.
FROM rust:1.89-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency compilation separately from source changes: copy only the
# manifests first, build a throwaway main.rs, then overwrite with real sources below.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/main.rs \
    && for name in calibrate_rasionalisasi import_institutions import_prodi_additions import_ptn_catalog seed; do \
        echo "fn main() {}" > "src/bin/${name}.rs"; \
    done
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin edvionptn-backend

# Now bring in the real source and rebuild — only this layer re-runs on code changes.
COPY src ./src
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    touch src/main.rs \
    && cargo build --release --bin edvionptn-backend \
    && cp target/release/edvionptn-backend /app/edvionptn-backend

# ─── Runtime ──────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin app

WORKDIR /app

# `sqlx::migrate!("./migrations")` embeds migration file contents at COMPILE TIME, so the
# running binary does not need this directory on disk — but keeping it here too makes the
# image self-documenting and lets scripts/sqlx-cli run against it if ever needed in-place.
COPY --from=builder /app/edvionptn-backend ./edvionptn-backend
COPY --from=builder /app/migrations ./migrations

USER app
EXPOSE 8080

ENTRYPOINT ["./edvionptn-backend"]
