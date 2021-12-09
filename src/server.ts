import { serve, Status, STATUS_TEXT } from "../deps.ts";
import keys, { PublicKey } from "./public_keys.ts";

interface Filter {
  /** If included only keys with ALL the tags provided will be returned. */
  allOf?: string[];
  /** If included only keys with AT LEAST ONE of the tags provided will be returned. */
  oneOf?: string[];
  /** If included only keys with NON of the tags provided will be returned. */
  noneOf?: string[];
}

function filterIncludesKey(filter: Filter, key: PublicKey) {
  if (
    filter.allOf &&
    filter.allOf.find((needle) => !key.tags.find((tag) => tag === needle))
  ) {
    return false;
  }

  if (
    filter.oneOf &&
    !key.tags.find((needle) => filter.oneOf?.includes(needle))
  ) {
    return false;
  }

  if (
    filter.noneOf &&
    key.tags.find((needle) => filter.noneOf?.includes(needle))
  ) {
    return false;
  }

  return true;
}

export default function start(port: number) {
  console.log(`Server listening at on port ${port}`);
  serve(
    (req: Request) => {
      const url = new URL(req.url);
      const params = new URLSearchParams(url.search);
      if (url.pathname !== "/api") {
        return new Response(undefined, {
          status: Status.NotFound,
          statusText: STATUS_TEXT.get(Status.NotFound),
        });
      }
      const filter: Filter = {};
      if (params.get("allOf") !== null) {
        filter.allOf = params.getAll("allOf");
      }
      if (params.get("oneOf") !== null) {
        filter.oneOf = params.getAll("oneOf");
      }
      if (params.get("noneOf") !== null) {
        filter.noneOf = params.getAll("noneOf");
      }
      const filteredKeys = keys.filter((key) => filterIncludesKey(filter, key));
      const responseData = filteredKeys.map((key) => `${key.key} ${key.name}`).join("\n");
      return new Response(responseData, {
        status: Status.OK,
        statusText: STATUS_TEXT.get(Status.OK),
      });
    },
    { addr: `:${port}` }
  );
}
