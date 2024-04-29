FROM rust AS build

WORKDIR /build

COPY . .

# Use Docker buildkit to cache our build directory for quicker recompilations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release \
    && mkdir /output \
    && cp target/release/libulid.so /output

FROM mariadb

ENV MARIADB_ALLOW_EMPTY_ROOT_PASSWORD=1
ENV MARIADB_DATABASE=foo

COPY --from=build /output/* /usr/lib/mysql/plugin/
