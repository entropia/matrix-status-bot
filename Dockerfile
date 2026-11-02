# Make sure RUST_VERSION matches the Rust version
ARG RUST_VERSION=1.93.1
ARG APP_NAME=matrix-status-bot

################################################################################
# Create a stage for building the application.
################################################################################

FROM docker.io/rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git

# Build the application.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/var/cache/cargo \
    CARGO_HOME=/var/cache/cargo cargo build --locked --profile release-lto && \
    cp ./target/release-lto/$APP_NAME /bin/server

################################################################################
# Create a new stage for running the application that contains the minimal
# We use dhi.io/static for the final stage because itâs a minimal Docker Hardened Image runtime (basically âjust # enough OS to run the binaryâ), which helps keep the image small and with a lower attack surface compared to a # # full Alpine/Debian runtime.
################################################################################

FROM docker.io/library/alpine:3.18 AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

# Drop privileges for runtime.
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/

# What the container should run when it is started.
CMD ["/bin/server"]