FROM ubuntu:noble

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:/usr/local/rustup$PATH

RUN apt-get update && apt-get install -y --no-install-recommends --reinstall\
    ca-certificates \
    curl \
    gcc \
    libsqlite3-dev \
    libssl-dev \
    pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y -v
RUN rustup toolchain install nightly --allow-downgrade --profile default

RUN cargo install cargo-audit

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src/
COPY tests tests/
COPY scripts scripts/
COPY .cargo .cargo/
