import start from "./src/server.ts";
import {
  filterIncludesKey,
  parseParameters,
} from "./src/routes/keys/filter.ts";
import { parseEnvironmentVariables } from "./src/environment/environment.ts";
import { Sentry } from "./deps.ts";
import loadConfig from "./src/config/load_config.ts";
import {
  getPGPTarget,
  servePGPKey,
  servePGPKeyList,
} from "./src/routes/pgp/serve_pgp.ts";
import { serveKeys } from "./src/routes/keys/serve-keys.ts";
import { serveHome } from "./src/routes/serve-home.ts";

const environment = parseEnvironmentVariables(Deno.env.toObject());

if (environment.SENTRY_DSN) {
  Sentry.init({
    dsn: environment.SENTRY_DSN,
    environment: environment.DOPPLER_ENVIRONMENT,
    tracesSampleRate: 1.0,
    release: environment.KEYS_VERSION,
  });
}

const { "ssh-keys": sshKeys, "pgp-keys": pgpKeys } = await loadConfig(
  environment.CONFIG_PATH,
);

start(
  environment.PORT,
  {
    filterIncludesKey,
    parseParameters,
    serveHome,
    serveKeys,
    getPGPTarget,
    servePGPKey,
    servePGPKeyList,
    sshKeys,
    pgpKeys,
    instanceName: environment.INSTANCE_NAME,
  },
  environment.KEYS_VERSION,
);
