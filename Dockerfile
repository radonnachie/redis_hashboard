# build blank project with dependencies as per manifest
FROM rust:slim as build
RUN USER=root cargo new --bin redis_hashboard
WORKDIR /redis_hashboard
COPY ./Cargo.* ./
RUN cargo build && rm src/*.rs

# rebuild with source
COPY ./src ./src
RUN cargo build --release

# final base, with the build artifact
FROM rust:slim
COPY --from=build /redis_hashboard/target/release/redis_hashboard .
COPY ./static ./static
CMD ["./redis_hashboard"]