FROM rust:1-slim as builder
RUN apt update \
    && apt install -y libpq-dev

WORKDIR /build

RUN cargo init --name dummy

COPY Cargo.toml Cargo.lock .

RUN cargo build --release

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

WORKDIR /migration
# Copy diesel_cli from previous build
COPY --from=builder /build/target/release/postgres-sql-migration-base /migrator/app
#Allows to run with lower privileges
RUN chmod ugo+rx /migrator/app

ENTRYPOINT [ "/migrator/app" ]