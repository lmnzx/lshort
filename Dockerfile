FROM rust:slim-bullseye AS build

WORKDIR /app
RUN apt-get update
RUN apt-get install -y build-essential clang lld libssl-dev pkg-config
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build /app/target/release/lshort lshort
ENV RUST_LOG trace
ENTRYPOINT ["./lshort"]