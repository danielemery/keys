FROM denoland/deno:2.7.5

WORKDIR /app

# Prefer not to run as root.
USER deno

# Cache the dependencies as a layer (re-run only when deno.json or deno.lock change).
COPY deno.json deno.lock ./
RUN deno install --frozen

# These steps will be re-run upon each file change in your working directory:
ADD . .
# Compile the main app so that it doesn't need to be compiled each startup/entry.
RUN deno cache main.ts

CMD ["run", "--allow-net", "--allow-env", "--allow-read", "main.ts"]
