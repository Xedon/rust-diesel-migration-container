FROM rust:1-slim as builder
RUN apt update \
    && apt install -y libpq-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./

RUN echo 'fn main() {}' > dummy.rs
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY src/ src/

RUN cargo build --release

FROM debian:buster-slim

RUN apt update \
    && apt-get install wget gnupg2 -y \
    && sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt buster-pgdg main" > /etc/apt/sources.list.d/pgdg.list' \
    && wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - \
    && apt-get autoremove wget gnupg2 -y \
    && apt update \
    && apt-get install -y postgresql-client-12 libpq-dev netcat-openbsd \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /migrator
# Copy diesel_cli from previous build
COPY --from=builder /build/target/release/app app
#Allows to run with lower privileges
RUN chmod ugo+rx app

ENTRYPOINT [ "/migrator/app" ]