FROM rust:1.67-bullseye

ARG SDVERSION
ENV SDVERSION=$SDVERSION

# Arguments are required
RUN test -n "$SDVERSION"

WORKDIR /usr/src/spelldawn
COPY src src
COPY tests tests
COPY justfile justfile
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*
RUN cargo install --path src/spelldawn

CMD spelldawn firestore ${SDVERSION}

EXPOSE 80
