import { serve, Status, STATUS_TEXT } from "../deps.ts";
import { filterIncludesKey, parseParameters } from "./filter.ts";
import keys from "./public_keys.ts";

/**
 * Start a simple http server listening on the provided port that listens on `/api` on
 * the provided port and provides authorized keys based on query string filter
 * parameters.
 * @param port The port to listen on.
 */
export default function start(port: number) {
  console.log(`Server listening at :${port}/api`);
  serve(
    (req: Request) => {
      try {
        const url = new URL(req.url);

        /** Any url that is not `/api` we can simply return a 404. */
        if (url.pathname !== "/api") {
          return new Response(undefined, {
            status: Status.NotFound,
            statusText: STATUS_TEXT.get(Status.NotFound),
          });
        }

        /** Parse query params into filters object and filter all public keys. */
        const filter = parseParameters(url);
        const filteredKeys = keys.filter((key) =>
          filterIncludesKey(filter, key)
        );

        /** Format the public keys in a suitable way for an authorized_keys file. */
        const responseData = filteredKeys
          .map((key) => `${key.key} ${key.user}@${key.name}`)
          .join("\n");

        /** Everything worked! We're good to return the keys and OK. */
        return new Response(responseData, {
          status: Status.OK,
          statusText: STATUS_TEXT.get(Status.OK),
        });
      } catch (err) {
        console.error(err);
        return new Response(undefined, {
          status: Status.InternalServerError,
          statusText: STATUS_TEXT.get(Status.InternalServerError),
        });
      }
    },
    { addr: `:${port}` }
  );
}
