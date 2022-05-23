FROM rust:1.59 AS x-builder

RUN USER=root cargo new --bin carp

WORKDIR /carp

RUN USER=root cargo new --lib entity
RUN USER=root cargo new --lib migration
RUN USER=root cargo new --bin reparse
RUN USER=root cargo new --bin rollback
RUN USER=root cargo new --lib tasks
RUN USER=root cargo new --lib task-docgen
RUN USER=root cargo new --bin plan-visualizer

COPY ./indexer/Cargo.toml ./Cargo.toml
COPY ./indexer/Cargo.lock ./Cargo.lock

COPY ./indexer/entity/Cargo.toml ./entity/Cargo.toml
COPY ./indexer/migration/Cargo.toml ./migration/Cargo.toml
COPY ./indexer/reparse/Cargo.toml ./reparse/Cargo.toml
COPY ./indexer/rollback/Cargo.toml ./rollback/Cargo.toml
COPY ./indexer/tasks/Cargo.toml ./tasks/Cargo.toml
COPY ./indexer/task-docgen/Cargo.toml ./task-docgen/Cargo.toml
COPY ./indexer/plan-visualizer/Cargo.toml ./plan-visualizer/Cargo.toml

RUN cargo build --release
RUN cargo clean --release -p carp
RUN cargo clean --release -p entity
RUN cargo clean --release -p migration
RUN cargo clean --release -p reparse
RUN cargo clean --release -p rollback
RUN cargo clean --release -p tasks
RUN cargo clean --release -p task-docgen
RUN cargo clean --release -p plan-visualizer

RUN rm -rf ./src
RUN rm -rf ./entity
RUN rm -rf ./migration
RUN rm -rf ./reparse
RUN rm -rf ./rollback
RUN rm -rf ./tasks
RUN rm -rf ./plan-visualizer

COPY ./indexer/entity ./entity
COPY ./indexer/migration ./migration
COPY ./indexer/reparse ./reparse
COPY ./indexer/rollback ./rollback
COPY ./indexer/tasks ./tasks
COPY ./indexer/src ./src
COPY ./indexer/plan-visualizer ./plan-visualizer

RUN cargo build --release \
    -p carp -p migration -p reparse -p rollback -p tasks -p plan-visualizer

WORKDIR /ops

RUN cp /carp/target/release/carp .
RUN cp /carp/target/release/migration .
RUN cp /carp/target/release/reparse .
RUN cp /carp/target/release/rollback .
RUN cp /carp/target/release/plan-visualizer .

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
