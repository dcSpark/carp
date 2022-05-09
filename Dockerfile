FROM rust:1.59 AS x-builder

RUN USER=root cargo new --bin carp

WORKDIR /carp

COPY ./entity ./entity
COPY ./migration ./migration

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release

RUN rm -rf ./src/*.rs
RUN rm ./target/release/carp

COPY ./genesis ./genesis
COPY ./src ./src

# trick to rebuild
RUN echo 'fn main() {}' > ./build.rs

#ARG GIT_BRANCH
#ARG GIT_COMMIT

RUN cargo build --release -p carp -p migration

# trick for "COPY --chown" on distroless
WORKDIR /ops
RUN cp /carp/target/release/carp .
RUN cp /carp/target/release/migration .

############################################################

# FROM gcr.io/distroless/cc:nonroot AS carp
FROM debian:stable-slim AS carp
ENV TZ=Etc/UTC
ARG APP=/app
# COPY --chown=nonroot:nonroot --from=x-builder /ops ${APP}
COPY --from=x-builder /ops ${APP}
WORKDIR ${APP}
#USER nonroot
ENTRYPOINT ["./carp"]
