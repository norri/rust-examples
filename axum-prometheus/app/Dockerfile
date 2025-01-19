FROM rust:1.83 AS dev

WORKDIR /my_app
RUN cargo install cargo-watch
COPY . .

CMD ["cargo", "watch", "-x", "run"]

FROM rust:1.83-alpine3.21 AS builder

RUN apk add musl-dev
WORKDIR /my_app
COPY . .

RUN cargo build --release

FROM alpine:3.21
COPY --from=builder /my_app/target/release/axum-postgres /
CMD ["./axum-postgres"]
