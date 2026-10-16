# syntax=docker/dockerfile:1

# Used as a build environment to build the production image
FROM docker.io/rust:1.99.1-alpine3.24 AS build

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Cache mounts aren't available in subsequent images, so copy the built executable out of /build/target after building it
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --locked --release && \
    cp /build/target/release/linkding-discord-bot /

# Provides a stage containing only the executable so it can easily be extracted and published with a release
FROM scratch AS binary

COPY --from=build /linkding-discord-bot /

# The production container for running the bot
FROM gcr.io/distroless/static-debian13:nonroot AS runtime

# Image metadata
LABEL org.opencontainers.image.title="Linkding Discord Bot" \
    org.opencontainers.image.description="A Discord bot for managing bookmarks in a Linkding instance" \
    org.opencontainers.image.source="https://github.com/mcb2003/linkding-discord-bot" \
    org.opencontainers.image.documentation="https://github.com/mcb2003/linkding-discord-bot#readme" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.authors="Michael Connor Buchan <mikey@blindcomputing.org>"

COPY --from=build /linkding-discord-bot /

ENTRYPOINT [ "/linkding-discord-bot" ]
