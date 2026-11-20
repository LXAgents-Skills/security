# syntax=docker/dockerfile:1

# Two stages. The first compiles the server with Cargo; `content/` is embedded in the binary
# at compile time, so the second stage carries one file and nothing else: no compiler, no
# source, no package manager state.
FROM rust:1-slim-bookworm AS build

WORKDIR /srv

# Built from the lockfile only. `--locked` fails rather than silently resolving something
# `Cargo.lock` does not contain, which is the point of building an image from a versioned tree.
# `rust-toolchain.toml` is deliberately not copied: the image's toolchain is the build's, and
# the file would make rustup fetch developer components (clippy, rustfmt) the build never uses.
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY content ./content
RUN cargo build --release --locked --bin lxagents-security-server

FROM debian:bookworm-slim

# Not root. The process answers JSON-RPC from text compiled into it, and nothing else.
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin lxagents
USER 10001

COPY --from=build /srv/target/release/lxagents-security-server /usr/local/bin/lxagents-security-server

# 3000 is the server's default, and `PORT` overrides it at runtime. The interface is `HOST`,
# which also defaults to 0.0.0.0 - every IPv4 interface.
#
# EXPOSE documents the port; it does not publish it. `docker run -p 3000:3000 …` is what makes
# the port reachable from outside the container, and forgetting it produces a container that
# is running, healthy, and connectable from nowhere. See wiki/environments/docker.md.
EXPOSE 3000

# HTTP is the default here, so a container that is run rather than spawned is reachable:
#
#   docker run --rm -p 3000:3000 -e MCP_AUTH_TOKEN lxagents-security:2.0.5
#
# HTTP refuses to start without MCP_AUTH_TOKEN (at least 32 characters): the container exits 1
# with a line saying why, and does not run open. The token is passed at run time and is
# deliberately not an ENV line here, because a value baked into the image ships to everyone
# who pulls it. stdio never reads it.
#
# stdio is one environment variable away, from the same entry point:
#
#   docker run --rm -i -e MCP_TRANSPORT=stdio lxagents-security:2.0.5
#
# For stdio, `-i` is not optional. Without it Docker does not attach stdin, the server sees
# closed input, and it exits at once - which reads as a broken image rather than a missing
# flag.
#
# Set MCP_ALLOWED_HOSTS whenever the container is reachable from anywhere but this machine.
# The Host allow-list is off unless it is set, and the server says so on startup. See
# wiki/environments/env.md.
ENV MCP_TRANSPORT=http
ENTRYPOINT ["/usr/local/bin/lxagents-security-server"]
