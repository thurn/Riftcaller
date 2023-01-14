FROM rust:1.65-bullseye

WORKDIR /usr/src/spelldawn
COPY src src
COPY justfile justfile
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*
RUN cargo install --path src/spelldawn

CMD ["spelldawn"]

EXPOSE 50052
