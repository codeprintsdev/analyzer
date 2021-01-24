FROM rust:latest as builder

RUN USER=root cargo new --bin client
WORKDIR /client

# Just copy the Cargo.toml and trigger a build so 
# that we compile our dependencies only.
# This way we avoid layer cache invalidation
# if our dependencies haven't changed,
# resulting in faster builds.
COPY Cargo.toml Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code and run the build again.
# This should only compile the client itself as the
# dependencies were already built above.
ADD . ./
RUN rm ./target/release/deps/client*
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
