FROM rust:1.68 AS builder
COPY . .
RUN cargo build

FROM debian:buster-slim
COPY --from=builder ./target/debug/url-shortner ./target/release/url-shortner
CMD ["/target/release/url-shortner", "--port", "3000"]
