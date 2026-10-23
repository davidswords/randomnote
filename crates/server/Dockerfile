# The randomtask server. Built from the crates directory, since the server
# uses the client's crypto crate for email normalisation:
#   docker build -f crates/server/Dockerfile crates
FROM rust:1.97-slim-bookworm AS build
WORKDIR /src
COPY client/crypto client/crypto
COPY server server
RUN cargo build --release --locked --manifest-path server/Cargo.toml

FROM debian:bookworm-slim
RUN useradd --system randomtask && mkdir /data && chown randomtask /data
COPY --from=build /src/server/target/release/randomtask-server /usr/local/bin/
USER randomtask
ENV RANDOMTASK_ADDRESS=0.0.0.0:8080 \
    RANDOMTASK_DATABASE=/data/randomtask-server.db
VOLUME /data
EXPOSE 8080
CMD ["randomtask-server"]
