FROM rust:slim-bullseye AS build
WORKDIR /app
RUN apt-get update
RUN apt-get install -y build-essential clang lld libssl-dev pkg-config
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM node:19-alpine AS node-builder
WORKDIR /app
COPY web/package.json .
RUN npm i
COPY web .
ENV NODE_ENV production
RUN npm run build


FROM debian:bullseye-slim
WORKDIR /app
COPY --from=build /app/target/release/lshort lshort
COPY --from=node-builder /app/dist web/dist
COPY config config 
ENV RUST_LOG trace
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./lshort"]