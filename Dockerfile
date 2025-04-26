# Fixed compiler version for reproductible builds
FROM rust:1.86.0 AS build

# setup a non-root user to use for safer compilation
ARG user=nonroot
RUN useradd --home-dir /build $user
USER $user
WORKDIR /build

# add the source "bill of materials" (SBOM) to the target binary
RUN cargo install cargo-auditable

# use only the relevant parts of the project
COPY --chown=$user src src
COPY --chown=$user Cargo.* .

# build as release and install
RUN cargo auditable install --path .


# Use a minimal image and run with a unprivileged user
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /build/target/release/dedicated-server-availability-watcher /usr/local/bin/dedicated-server-availability-watcher
ENTRYPOINT ["dedicated-server-availability-watcher"]
