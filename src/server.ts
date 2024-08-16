import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { filterIncludesKey, parseParameters } from "./filter.ts";
import { PublicSSHKey } from "./load_config.ts";

/**
 * The dependencies required by the server.
 * We use this to make it easier to mock the dependencies in tests.
 */
export interface ServerDependencies {
  filterIncludesKey: typeof filterIncludesKey;
  parseParameters: typeof parseParameters;
  keys: PublicSSHKey[];
}

/**
 * Start a simple http server listening on the provided port that listens on
 * the provided port and provides authorized keys based on query string filter
 * parameters.
 * @param port The port to listen on.
 */
export default function start(
  port: number,
  dependencies: ServerDependencies,
  version: string,
) {
  console.log(`Server listening at :${port}`);
  Deno.serve({
    port,
    handler: (req) => handleRequest(req, dependencies, version),
  });
}

const validKeysRoutes = [
  "/keys",
  "/keys/",
  "/authorized_keys",
  "/authorized_keys/",
];

export function handleRequest(
  req: Request,
  dependencies: ServerDependencies,
  version: string,
) {
  try {
    const url = new URL(req.url);
    // For each supported keys endpoint serve the keys
    if (validKeysRoutes.includes(url.pathname)) {
      return serveKeys(url, version, dependencies);
    }

    // If the url is not recognized, return a 404.
    return new Response(undefined, {
      status: STATUS_CODE.NotFound,
      statusText: STATUS_TEXT[STATUS_CODE.NotFound],
    });
  } catch (err) {
    console.error(err);
    return new Response(undefined, {
      status: STATUS_CODE.InternalServerError,
      statusText: STATUS_TEXT[STATUS_CODE.InternalServerError],
    });
  }
}

function serveKeys(
  url: URL,
  version: string,
  dependencies: ServerDependencies,
) {
  const { filterIncludesKey, parseParameters, keys } = dependencies;

  /** Parse query params into filters object and filter all public keys. */
  const filter = parseParameters(url);
  const filteredKeys = keys.filter((key) => filterIncludesKey(filter, key));

  /** Format the public keys in a suitable way for an authorized_keys file. */
  const responseData = filteredKeys
    .map((key) => `${key.key} ${key.user}@${key.name}`)
    .join("\n");

  /** Everything worked! We're good to return the keys and OK. */
  return new Response(responseData, {
    status: STATUS_CODE.OK,
    statusText: STATUS_TEXT[STATUS_CODE.OK],
    headers: {
      "X-Keys-Version": version,
    },
  });
}
