FROM rust:1.95-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/readsmith /usr/local/bin/readsmith
COPY --from=builder /app/static ./static
COPY --from=builder /app/templates ./templates
RUN mkdir -p /data
EXPOSE 8080
CMD ["readsmith"]
