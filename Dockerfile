FROM rust:1.59 AS x-builder

RUN USER=root cargo new --bin oura-postgres-sink

WORKDIR /oura-postgres-sink

COPY ./entity ./entity
COPY ./migration ./migration

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release

RUN rm -rf ./src/*.rs
RUN rm ./target/release/oura-postgres-sink

COPY ./genesis ./genesis
COPY ./src ./src

# trick to rebuild
RUN echo 'fn main() {}' > ./build.rs

#ARG GIT_BRANCH
#ARG GIT_COMMIT

RUN cargo build --release -p oura-postgres-sink -p migration

# trick for "COPY --chown" on distroless
WORKDIR /ops
RUN cp /oura-postgres-sink/target/release/oura-postgres-sink .
RUN cp /oura-postgres-sink/target/release/migration .

############################################################

# FROM gcr.io/distroless/cc:nonroot AS oura-postgres-sink
FROM debian:stable-slim AS oura-postgres-sink
ENV TZ=Etc/UTC
ARG APP=/app
# COPY --chown=nonroot:nonroot --from=x-builder /ops ${APP}
COPY --from=x-builder /ops ${APP}
WORKDIR ${APP}
#USER nonroot
ENTRYPOINT ["./oura-postgres-sink"]
