FROM rust:1.73 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY ./indexer .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY ./indexer ./

RUN cargo build --release -p carp -p migration

WORKDIR /ops

RUN cp /app/target/release/carp .
RUN cp /app/target/release/migration .

COPY ./indexer/genesis ./genesis
COPY ./indexer/execution_plans ./execution_plans

############################################################

FROM debian:stable-slim AS carp
ENV TZ=Etc/UTC
ARG APP=/app
COPY --from=builder /ops ${APP}
WORKDIR ${APP}
#USER nonroot
ENTRYPOINT ["/bin/sh", "-c" , "./migration up && ./carp"]
