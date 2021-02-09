FROM rust:latest as builder

WORKDIR /client

# Just copy the Cargo.toml and trigger a build so 
# that we compile our dependencies only.
# This way we avoid layer cache invalidation
# if our dependencies haven't changed,
# resulting in faster builds.
COPY . ./
RUN cargo build --release

# Our production image starts here, which uses 
# the files from the builder image above.
FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y git \
    && rm -rf /var/lib/apt/lists/* 

COPY --from=builder /client/target/release/client /usr/local/bin/client
COPY entrypoint.sh /entrypoint.sh
RUN chmod a+x /entrypoint.sh
WORKDIR /repo
ENTRYPOINT [ "/entrypoint.sh" ]
