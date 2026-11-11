# syntax=docker/dockerfile:1

# Two stages. The source is TypeScript and the image runs the compiled `dist/`, so the
# first stage installs the full tree (the compiler is a devDependency) and builds, and
# the second carries only production dependencies, `dist/` and `content/`. Neither the
# compiler nor the TypeScript source reaches the runtime image.
FROM node:22-alpine AS build

WORKDIR /srv

# Installed from the lockfile only. `npm ci` fails rather than silently resolving
# something the lockfile does not contain, which is the point of building an image from
# a versioned tree. Lifecycle scripts are disabled, so `prepare` does not run here: the
# build is the explicit next step instead of a side effect of installing.
COPY package.json package-lock.json tsconfig.json ./
RUN npm ci --ignore-scripts

COPY src ./src
RUN npm run build

FROM node:22-alpine

# Lifecycle scripts are disabled here as well: running them by default is supply-chain
# risk with nothing to offset it, and `prepare` would need the compiler this stage does
# not install. `--omit=dev` keeps the compiler and the test-only tree out of the image;
# `.dockerignore` already excludes the test files themselves, so this image cannot run
# its own suite.
WORKDIR /srv

COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts --omit=dev

# `content/` is the product. It is copied, not generated, and it is the only thing in
# this image that a change to the set is expected to alter. `dist/` is the server,
# compiled in the first stage.
COPY --from=build /srv/dist ./dist
COPY content ./content

# Not root. The process reads files and answers JSON-RPC, and nothing else.
USER node

# 3000 is `dist/index.js`'s default, and `PORT` overrides it at runtime. The
# interface is `HOST`, which also defaults to 0.0.0.0 - every IPv4 interface.
#
# EXPOSE documents the port; it does not publish it. `docker run -p 3000:3000 …`
# is what makes the port reachable from outside the container, and forgetting
# it produces a container that is running, healthy, and connectable from
# nowhere. See wiki/environments/docker.md.
EXPOSE 3000

# HTTP is the default here, so a container that is run rather than spawned is
# reachable:
#
#   docker run --rm -p 3000:3000 -e MCP_AUTH_TOKEN lxagents-security:2.0.4
#
# HTTP refuses to start without MCP_AUTH_TOKEN (at least 32 characters): the container
# exits 1 with a line saying why, and does not run open. The token is passed at run time and
# is deliberately not an ENV line here, because a value baked into the image ships to
# everyone who pulls it. stdio never reads it.
#
# stdio is one environment variable away, and the image serves both from the
# same entry point, because they differ only in which transport is selected:
#
#   docker run --rm -i -e MCP_TRANSPORT=stdio lxagents-security:2.0.4
#
# For stdio, `-i` is not optional. Without it Docker does not attach stdin, the
# server sees closed input, and it exits at once - which reads as a broken image
# rather than a missing flag. There is nothing to publish in that form.
#
# Set MCP_ALLOWED_HOSTS whenever the container is reachable from anywhere but
# this machine. The Host allow-list is off unless it is set, and the server says
# so on startup. See wiki/environments/env.md.
ENV MCP_TRANSPORT=http
ENTRYPOINT ["node", "dist/index.js"]
