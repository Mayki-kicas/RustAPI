FROM rust:1.65 as builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY . .

RUN cargo build --release

FROM debian:buster-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/library_api .

EXPOSE 8080

CMD ["./library_api"]
