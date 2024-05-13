FROM rust:1.73 AS x-builder

WORKDIR /app

COPY ./Cargo.* ./
COPY ./indexer ./indexer

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
COPY --from=x-builder /ops ${APP}
WORKDIR ${APP}
#USER nonroot
ENTRYPOINT ["/bin/sh", "-c" , "./migration up && ./carp"]
