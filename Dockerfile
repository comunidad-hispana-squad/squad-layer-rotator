ARG RUST_VERSION=1.85.0


# Stage 1: Build environment
FROM rust:${RUST_VERSION}-slim-bullseye AS build

RUN apt-get update && \
    apt-get install -y pkgconf libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /squad-layer-rotator

COPY . .
RUN cargo install --path .


# Stage 2: Production environment
FROM debian:bullseye-slim AS prod

LABEL maintainer="Comunidad Hispana de Squad"
LABEL description="Production ready docker container for Squad Layer Rotator"

RUN apt-get update && \
    apt-get install -y \
    libssh2-1 \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN mkdir -p /app/layers

# Copy necessary files from the build stage
COPY --from=build /usr/local/cargo/bin/squad-layer-rotator /usr/local/bin/squad-layer-rotator

CMD ["squad-layer-rotator"]
