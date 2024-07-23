import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { filterIncludesKey, parseParameters } from "./filter.ts";
import keys from "./public_keys.ts";
import pgp_key from "./pgp_key.ts";

/**
 * The dependencies required by the server.
 * We use this to make it easier to mock the dependencies in tests.
 */
export interface ServerDependencies {
  filterIncludesKey: typeof filterIncludesKey;
  parseParameters: typeof parseParameters;
  keys: typeof keys;
}

/**
 * Start a simple http server listening on the provided port that listens on `/api` on
 * the provided port and provides authorized keys based on query string filter
 * parameters.
 * @param port The port to listen on.
 */
export default function start(port: number, dependencies: ServerDependencies) {
  console.log(`Server listening at :${port}/api`);
  Deno.serve({
    port,
    handler: (req) => handleRequest(req, dependencies),
  });
}

export function handleRequest(req: Request, dependencies: ServerDependencies) {
  const { filterIncludesKey, parseParameters, keys } = dependencies;
  try {
    const url = new URL(req.url);

    /** If the url is /pgp return static public pgp key. */
    if (url.pathname === "/pgp") {
      return new Response(pgp_key, {
        status: STATUS_CODE.OK,
        statusText: STATUS_TEXT[STATUS_CODE.OK],
      });
    }

    /** If the url is /pgp/daniel_emery.pub.asc return the keys with headers indicating download is preferred. */
    if (url.pathname === "/pgp/daniel_emery.pub.asc") {
      return new Response(pgp_key, {
        status: STATUS_CODE.OK,
        statusText: STATUS_TEXT[STATUS_CODE.OK],
        headers: {
          "Content-Disposition": "attachment; filename=daniel_emery.pub.asc",
        },
      });
    }

    /** Any other url that is not `/api` we can simply return a 404. */
    if (url.pathname !== "/api") {
      return new Response(undefined, {
        status: STATUS_CODE.NotFound,
        statusText: STATUS_TEXT[STATUS_CODE.NotFound],
      });
    }

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
    });
  } catch (err) {
    console.error(err);
    return new Response(undefined, {
      status: STATUS_CODE.InternalServerError,
      statusText: STATUS_TEXT[STATUS_CODE.InternalServerError],
    });
  }
}
