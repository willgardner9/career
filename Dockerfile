FROM lukemathwalker/cargo-chef:0.1.78-rust-1.99.1-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies only. This layer is cached until recipe.json changes.
RUN cargo chef cook --release --workspace --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration
COPY src ./src
RUN cargo build --release --workspace

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/career /usr/local/bin/career
COPY --from=builder /app/target/release/migration /usr/local/bin/migration
RUN useradd --system --no-create-home --shell /usr/sbin/nologin app
USER app
EXPOSE 3000
CMD ["career"]
