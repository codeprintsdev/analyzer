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

COPY --from=builder /analyzer/target/release/codeprints-analyzer /usr/local/bin/codeprints-analyzer
WORKDIR /repo
ENTRYPOINT [ "codeprints-analyzer" ]
# The standard command parses the commits of a repository
CMD ["run"]
