import start from "./src/server.ts";
import keys from "./src/public_keys.ts";
import { filterIncludesKey, parseParameters } from "./src/filter.ts";
import { parseEnvironmentVariables } from "./src/environment.ts";

const environment = parseEnvironmentVariables(Deno.env.toObject());

start(environment.PORT, {
  filterIncludesKey,
  parseParameters,
  keys,
});
