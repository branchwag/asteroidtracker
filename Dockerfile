# syntax=docker/dockerfile:1.7

# ---- Build stage ----
FROM rust:1.90-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        clang \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --locked

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY style ./style
COPY public ./public

RUN cargo leptos build --release


# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin app

WORKDIR /app

COPY --from=builder --chown=app:app /app/target/release/asteroidtracker /app/asteroidtracker
COPY --from=builder --chown=app:app /app/target/site /app/site

ENV LEPTOS_SITE_ROOT=/app/site \
    LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    NASA_API_KEY=DEMO_KEY

USER app
EXPOSE 3000

CMD ["/app/asteroidtracker"]
