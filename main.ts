import start from "./src/server.ts";
import keys from "./src/public_keys.ts";
import { filterIncludesKey, parseParameters } from "./src/filter.ts";

start(8000, {
  filterIncludesKey,
  parseParameters,
  keys,
});
