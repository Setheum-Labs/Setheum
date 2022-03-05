FROM ubuntu:18.04
RUN apt-get update && apt-get install -y \
    build-essential clang git\
    curl

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"
RUN rustup update nightly
RUN rustup update stable
RUN rustup target add wasm32-unknown-unknown --toolchain nightly

WORKDIR /build
COPY . /build
RUN make release

FROM debian:buster

COPY --from=build /build/target/release/setheum-node /usr/local/bin
ENTRYPOINT ["setheum-node"]
