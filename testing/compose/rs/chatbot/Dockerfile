FROM rust:1.88-slim AS build
WORKDIR /repo
COPY Cargo.toml rust-toolchain.toml ./
COPY src ./src
COPY core ./core
COPY bindings/heeczer-rs ./bindings/heeczer-rs
COPY testing/compose/rs/chatbot ./testing/compose/rs/chatbot
COPY testing/tests/fixtures ./testing/tests/fixtures
RUN cargo build --manifest-path testing/compose/rs/chatbot/Cargo.toml --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /repo/testing/compose/rs/chatbot/target/release/heeczer-test-rs-chatbot /usr/local/bin/chatbot
COPY --from=build /repo/testing/tests/fixtures /fixtures
EXPOSE 8000
CMD ["/usr/local/bin/chatbot"]
