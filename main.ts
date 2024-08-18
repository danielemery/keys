import start from "./src/server.ts";
import { filterIncludesKey, parseParameters } from "./src/filter.ts";
import { parseEnvironmentVariables } from "./src/environment.ts";
import { Sentry } from "./deps.ts";
import loadConfig from "./src/load_config.ts";
import loadPGPKeys from "./src/load_pgp.ts";
import { getPGPTarget, servePGPKey, servePGPKeyList } from "./src/serve_pgp.ts";
import { serveKeys } from "./src/serve-keys.ts";

const environment = parseEnvironmentVariables(Deno.env.toObject());

if (environment.SENTRY_DSN) {
  Sentry.init({
    dsn: environment.SENTRY_DSN,
    environment: environment.DOPPLER_ENVIRONMENT,
    tracesSampleRate: 1.0,
    release: environment.KEYS_VERSION,
  });
}

const { "ssh-keys": sshKeys } = await loadConfig(environment.CONFIG_PATH);
const pgpKeys = await loadPGPKeys(environment.PGP_KEYS_PATH);

start(
  environment.PORT,
  {
    filterIncludesKey,
    parseParameters,
    serveKeys,
    getPGPTarget,
    servePGPKey,
    servePGPKeyList,
    sshKeys,
    pgpKeys,
  },
  environment.KEYS_VERSION,
);
