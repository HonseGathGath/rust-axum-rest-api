FROM rust:latest as builder
WORKDIR /app
COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-axum-rest-api ./app

CMD ["./app"]
