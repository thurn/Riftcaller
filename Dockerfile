FROM rust:1.67-bullseye

ARG SDVERSION
ENV SDVERSION=$SDVERSION
ENV PORT $PORT

# Arguments are required
RUN test -n "$SDVERSION"

WORKDIR /usr/src/riftcaller
COPY src src
COPY tests tests
COPY justfile justfile
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*
RUN cargo install --path src/riftcaller

CMD riftcaller firestore ${SDVERSION} stackdriver

EXPOSE $PORT
