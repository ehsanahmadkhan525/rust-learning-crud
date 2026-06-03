# ============ STAGE 1: build ============
# Start from an image that already has the Rust compiler + cargo.
FROM rust:1 AS builder

WORKDIR /app

# Use sqlx's offline query cache so we DON'T need a live database while building.
ENV SQLX_OFFLINE=true

# Copy the dependency manifests first. (Docker caches each step; copying these
# separately means deps only re-download when Cargo.toml changes, not on every
# source edit.)
COPY Cargo.toml Cargo.lock ./

# Copy the source code and the committed sqlx offline cache.
COPY src ./src
COPY .sqlx ./.sqlx

# Build an optimized "release" binary (smaller + faster than the debug build).
RUN cargo build --release

# ============ STAGE 2: runtime ============
# A small Debian image — no Rust compiler, just enough to RUN the binary.
FROM debian:bookworm-slim

WORKDIR /app

# Copy ONLY the compiled binary out of the build stage. Everything else
# (the compiler, source, dependencies) is left behind, keeping this image small.
COPY --from=builder /app/target/release/rust-learning-crud .

# Document that the app listens on port 4000.
EXPOSE 4000

# The command that runs when the container starts.
CMD ["./rust-learning-crud"]
