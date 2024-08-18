import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { filterIncludesKey, parseParameters } from "./filter.ts";
import { PublicSSHKey } from "./load_config.ts";
import { PGPKey } from "./load_pgp.ts";
import { getPGPTarget, servePGPKey, servePGPKeyList } from "./serve_pgp.ts";
import { getContentType } from "./content-types.ts";
import { serveKeys } from "./serve-keys.ts";

/**
 * The dependencies required by the server.
 * We use this to make it easier to mock the dependencies in tests.
 */
export interface ServerDependencies {
  filterIncludesKey: typeof filterIncludesKey;
  parseParameters: typeof parseParameters;
  serveKeys: typeof serveKeys;
  getPGPTarget: typeof getPGPTarget;
  servePGPKey: typeof servePGPKey;
  servePGPKeyList: typeof servePGPKeyList;
  sshKeys: PublicSSHKey[];
  pgpKeys: PGPKey[];
}

/**
 * Start a simple http server that listens on the provided port and provides authorized keys based on query string filter
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

const validSSHKeyRoutes = [
  "/keys",
  "/keys/",
  "/authorized_keys",
  "/authorized_keys/",
];
const validPGPKeyRoutes = ["/pgp", "/pgp/"];

export function handleRequest(
  req: Request,
  dependencies: ServerDependencies,
  version: string,
) {
  // Extract content type
  const contentType = getContentType(req.headers);

  const { serveKeys, servePGPKeyList, getPGPTarget, servePGPKey } =
    dependencies;
  try {
    const url = new URL(req.url);

    /** If the url is /pgp return the list of loaded pgp keys. */
    if (validPGPKeyRoutes.includes(url.pathname)) {
      return servePGPKeyList(version, dependencies, contentType);
    }

    /**
     * If the url is /pgp/${some_key} then return the pgp key body.
     * If a file extension is included in the url, content disposition headers are set to indicate download is preferred.
     */
    const pgpKeyTarget = getPGPTarget(url.pathname);
    if (pgpKeyTarget) {
      return servePGPKey(pgpKeyTarget, version, dependencies, contentType);
    }

    // For each supported keys endpoint serve the keys
    if (validSSHKeyRoutes.includes(url.pathname)) {
      return serveKeys(url, version, dependencies, contentType);
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
