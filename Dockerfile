FROM rust:1.83-slim AS builder

WORKDIR /app


RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*


COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src


COPY src ./src

COPY migrations ./migrations

RUN touch src/main.rs src/lib.rs && \
    cargo build --release


FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*


COPY --from=builder /app/target/release/url-shortener .

COPY migrations ./migrations


RUN mkdir -p /app/data


RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app
    
USER appuser

EXPOSE 3000

CMD ["./url-shortener"]