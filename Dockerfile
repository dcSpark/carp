FROM rust:1.61 AS x-builder

WORKDIR /indexer

COPY ./indexer ./

RUN cargo build --release -p carp -p migration

WORKDIR /ops

RUN cp /indexer/target/release/carp .
RUN cp /indexer/target/release/migration .

COPY ./indexer/genesis ./genesis
COPY ./indexer/execution_plans ./execution_plans

############################################################

FROM debian:stable-slim AS carp
ENV TZ=Etc/UTC
ARG APP=/app
COPY --from=x-builder /ops ${APP}
WORKDIR ${APP}
#USER nonroot
ENTRYPOINT ["./carp"]