FROM rust:1.68 AS builder
COPY . .
RUN cargo build

FROM debian:buster-slim
COPY --from=builder ./target/debug/url-shortener ./target/release/url-shortener
CMD ["/target/release/url-shortener", "--port", "3000"]
