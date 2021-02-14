FROM rust:latest as builder

WORKDIR /analyzer
COPY . ./
RUN cargo build --release

# Our production image starts here, which uses 
# the files from the builder image above.
FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y git \
    && rm -rf /var/lib/apt/lists/* 

COPY --from=builder /analyzer/target/release/analyzer /usr/local/bin/analyzer
WORKDIR /repo
ENTRYPOINT [ "analyzer" ]
