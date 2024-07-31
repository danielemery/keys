# Use the official Deno image as the source to copy the Deno executable from.
FROM denoland/deno:bin-1.45.4 AS deno

# Use the official TypeScript Node image as a base image.
# We don't actually use it for much since deno takes care of things, but
# it provides a base set of tools for a dev container.
FROM mcr.microsoft.com/devcontainers/typescript-node:20

# Copy the deno executable into the container.
COPY --from=deno /deno /usr/local/bin/deno
