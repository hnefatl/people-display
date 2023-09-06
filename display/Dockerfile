FROM rust:1-bookworm as chef
WORKDIR /app
# Use cargo chef to cache dependencies. Invalidated once any in Cargo.toml change,
# but good for code-only iterations.
RUN cargo install cargo-chef

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
# Cache apt packages: https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
# TODO: can maybe remove libssl-dev here and below, now that we're statically linking?
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  apt update && apt-get --no-install-recommends install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    libsdl2-dev \
    libsdl2-image-dev \
    cmake \
    gcc \
    perl

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --bin display --recipe-path recipe.json
COPY . .
RUN cargo build --locked --release --bin display

FROM rust:1-slim-bookworm as runtime
WORKDIR /app
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  apt update && apt-get --no-install-recommends install -y \
    libssl-dev \
    libsdl2-2.0-0 \
    libsdl2-image-2.0-0

COPY --from=builder /app/target/release/display /app/display
ENTRYPOINT ["/app/display"]
