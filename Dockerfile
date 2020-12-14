FROM rust:slim as builder

WORKDIR ChaldeaDatabase
COPY _tmp.rs .
COPY Cargo.toml .
RUN sed -i 's#src/main.rs#_tmp.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#_tmp.rs#src/main.rs#' Cargo.toml
COPY . .
RUN cargo build --release

FROM debian:buster-slim as runner
RUN apt-get update \
 && apt-get install -y --no-install-recommends apt-transport-https ca-certificates \
 && apt-get clean \
 && apt-get autoremove \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /ChaldeaDatabase/target/release/chaldea_database /main
CMD ["/main"]
